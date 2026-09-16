use serde::{Deserialize, Serialize};

/// A flat snapshot of what a model contains.
///
/// This type exists for the boundary: it is what the kernel is willing to hand to
/// Flutter in this step, while the model itself stays in Rust. It is deliberately
/// made of primitives only — counts, the schema version, the revision and the
/// project id as a string (Dart has no UUID type yet) — so no model data is
/// duplicated on the UI side and no engineering logic can drift into Flutter.
///
/// Full model transfer, DTOs and a bridge query/command layer belong to a later
/// step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSummary {
    /// Schema version of the model this summary was taken from.
    pub schema_version: u32,
    /// Project identity, as a hyphenated UUID string.
    pub project_id: String,
    /// Number of accepted mutations applied to the model.
    pub revision: u64,
    /// Number of levels.
    pub levels: u32,
    /// Number of grid lines.
    pub grids: u32,
    /// Number of materials.
    pub materials: u32,
    /// Number of cross sections.
    pub cross_sections: u32,
    /// Number of physical elements.
    pub elements: u32,
}

impl ModelSummary {
    /// Total number of entities in the model.
    pub fn total_entities(&self) -> u32 {
        self.levels
            .saturating_add(self.grids)
            .saturating_add(self.materials)
            .saturating_add(self.cross_sections)
            .saturating_add(self.elements)
    }

    /// `true` when the model holds nothing at all.
    pub fn is_empty(&self) -> bool {
        self.total_entities() == 0
    }
}
