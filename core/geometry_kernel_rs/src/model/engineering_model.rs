//! The engineering model aggregate.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::Element;
use crate::error::{ModelError, ReferenceKind};
use crate::model::{CrossSection, Grid, IdMap, Level, Material, ModelSummary, ValidationMode};

/// Version of the serialized engineering model schema.
///
/// The model always carries the version it was written with, so a future loader can
/// recognise older data. A migration framework is intentionally *not* part of this
/// step — only the version marker that makes one possible.
pub const MODEL_SCHEMA_VERSION: u32 = 1;

/// The engineering model: the single source of truth of the platform.
///
/// The struct *is* the model — no database, no backend, no renderer. Every
/// collection is keyed by [`Uuid`] ([`IdMap`]), so `ID → entity` is a direct map
/// lookup and identity never depends on storage order.
///
/// # Mutations and the future command layer
///
/// Entities are added through the `add_*` methods, which
/// * reject an id that is already used anywhere in the model ([`ModelError::DuplicateId`]),
/// * reject an element whose references do not resolve while the model is in
///   [`ValidationMode::Strict`] ([`ModelError::MissingReference`]),
/// * and bump [`EngineeringModel::revision`].
///
/// That single funnel is what a future command layer
/// (`CreateColumnCommand → ElementCreated event → history`) will wrap: a command
/// applies a mutation here and records the revision it produced. The model itself
/// stays free of events and history for now.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineeringModel {
    /// Version of the model schema this data was written with.
    pub schema_version: u32,
    /// Stable identity of the project.
    pub project_id: Uuid,
    /// Number of accepted mutations, starting at `0` for a new model.
    pub revision: u64,
    /// Levels of the project.
    pub levels: IdMap<Level>,
    /// Grid lines of the project.
    pub grids: IdMap<Grid>,
    /// Materials of the project.
    pub materials: IdMap<Material>,
    /// Reusable cross sections of the project.
    pub cross_sections: IdMap<CrossSection>,
    /// Physical elements of the project.
    pub elements: IdMap<Element>,
    /// How the model treats references on mutation.
    ///
    /// This is an editing policy, not model data, so it is not serialized: a loaded
    /// model always starts strict.
    #[serde(skip, default)]
    pub validation_mode: ValidationMode,
}

impl Default for EngineeringModel {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineeringModel {
    /// An empty model with a fresh project id, the current schema version and
    /// revision `0`.
    pub fn new() -> Self {
        Self {
            schema_version: MODEL_SCHEMA_VERSION,
            project_id: Uuid::new_v4(),
            revision: 0,
            levels: IdMap::new(),
            grids: IdMap::new(),
            materials: IdMap::new(),
            cross_sections: IdMap::new(),
            elements: IdMap::new(),
            validation_mode: ValidationMode::default(),
        }
    }

    /// Same model with an explicit reference-checking policy.
    pub fn with_validation_mode(mut self, mode: ValidationMode) -> Self {
        self.validation_mode = mode;
        self
    }

    // ---------------------------------------------------------------- mutation

    /// Adds a level.
    pub fn add_level(&mut self, level: Level) -> Result<Uuid, ModelError> {
        let id = level.id;
        self.ensure_id_is_free(id)?;
        self.levels.insert(id, level);
        self.revision += 1;
        Ok(id)
    }

    /// Adds a grid line.
    pub fn add_grid(&mut self, grid: Grid) -> Result<Uuid, ModelError> {
        let id = grid.id;
        self.ensure_id_is_free(id)?;
        self.grids.insert(id, grid);
        self.revision += 1;
        Ok(id)
    }

    /// Adds a material.
    pub fn add_material(&mut self, material: Material) -> Result<Uuid, ModelError> {
        let id = material.id;
        self.ensure_id_is_free(id)?;
        self.materials.insert(id, material);
        self.revision += 1;
        Ok(id)
    }

    /// Adds a cross section.
    pub fn add_cross_section(&mut self, cross_section: CrossSection) -> Result<Uuid, ModelError> {
        let id = cross_section.id;
        self.ensure_id_is_free(id)?;
        self.cross_sections.insert(id, cross_section);
        self.revision += 1;
        Ok(id)
    }

    /// Adds a physical element.
    ///
    /// In [`ValidationMode::Strict`] the element is rejected when any of its
    /// references (`base_level_id`, `material_id`, `cross_section_id`, ...) does not
    /// exist in the model.
    pub fn add_element(&mut self, element: Element) -> Result<Uuid, ModelError> {
        let id = element.id();
        self.ensure_id_is_free(id)?;
        if self.validation_mode == ValidationMode::Strict {
            self.ensure_references_resolve(&element)?;
        }
        self.elements.insert(id, element);
        self.revision += 1;
        Ok(id)
    }

    // ------------------------------------------------------------------ lookup

    pub fn level(&self, id: Uuid) -> Option<&Level> {
        self.levels.get(&id)
    }

    pub fn grid(&self, id: Uuid) -> Option<&Grid> {
        self.grids.get(&id)
    }

    pub fn material(&self, id: Uuid) -> Option<&Material> {
        self.materials.get(&id)
    }

    pub fn cross_section(&self, id: Uuid) -> Option<&CrossSection> {
        self.cross_sections.get(&id)
    }

    pub fn element(&self, id: Uuid) -> Option<&Element> {
        self.elements.get(&id)
    }

    /// Number of physical elements in the model.
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    /// `true` when the model holds no entity at all.
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
            && self.grids.is_empty()
            && self.materials.is_empty()
            && self.cross_sections.is_empty()
            && self.elements.is_empty()
    }

    /// `true` when `id` is used by any entity of the model, whatever its kind.
    pub fn contains_id(&self, id: Uuid) -> bool {
        self.levels.contains_key(&id)
            || self.grids.contains_key(&id)
            || self.materials.contains_key(&id)
            || self.cross_sections.contains_key(&id)
            || self.elements.contains_key(&id)
    }

    // ---------------------------------------------------------- serialization

    /// Serializes the model to pretty JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Reads a model back from JSON.
    ///
    /// Deserialization trusts the input: a hand-edited file can therefore contain
    /// dangling references. Call [`EngineeringModel::validate`] to inspect what was
    /// loaded.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    // -------------------------------------------------------------- reporting

    /// A flat, FFI-friendly snapshot of the model.
    pub fn summary(&self) -> ModelSummary {
        let count = |len: usize| u32::try_from(len).unwrap_or(u32::MAX);
        ModelSummary {
            schema_version: self.schema_version,
            project_id: self.project_id.to_string(),
            revision: self.revision,
            levels: count(self.levels.len()),
            grids: count(self.grids.len()),
            materials: count(self.materials.len()),
            cross_sections: count(self.cross_sections.len()),
            elements: count(self.elements.len()),
        }
    }

    // ---------------------------------------------------------------- internal

    fn ensure_id_is_free(&self, id: Uuid) -> Result<(), ModelError> {
        if self.contains_id(id) {
            return Err(ModelError::DuplicateId { id });
        }
        Ok(())
    }

    fn ensure_references_resolve(&self, element: &Element) -> Result<(), ModelError> {
        let element_id = element.id();
        for (reference, missing_id) in element.referenced_ids() {
            let resolved = match reference {
                ReferenceKind::Level => self.levels.contains_key(&missing_id),
                ReferenceKind::Grid => self.grids.contains_key(&missing_id),
                ReferenceKind::Material => self.materials.contains_key(&missing_id),
                ReferenceKind::CrossSection => self.cross_sections.contains_key(&missing_id),
            };
            if !resolved {
                return Err(ModelError::MissingReference {
                    element_id,
                    reference,
                    missing_id,
                });
            }
        }
        Ok(())
    }
}
