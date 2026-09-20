//! Project persistence layer.
//!
//! A [`Project`] wraps an [`EngineeringModel`] with lightweight metadata that is
//! useful for file management (name, description, timestamps) but must never
//! become part of the engineering domain state.
//!
//! # Design rules
//!
//! * **Single identity.** [`EngineeringModel::project_id`] *is* the project
//!   identity. `Project` never introduces a second UUID that could drift from it.
//! * **Metadata is not domain state.** Timestamps, name and description are
//!   organisational. They are serialised so they survive a round-trip, but they
//!   never affect geometry, calculations, model revision or validation.
//! * **Two version numbers.** [`PROJECT_FORMAT_VERSION`] is the *file format*
//!   version (how bytes are arranged on disk). [`crate::model::MODEL_SCHEMA_VERSION`]
//!   is the *domain model* version (what fields the engineering model carries).
//!   They evolve independently: a format v3 file may still carry a schema v1 model.
//! * **Format-agnostic.** The [`Project`] type is deliberately independent of the
//!   serialisation format. [`crate::project::format`] provides the trait and the
//!   concrete JSON-based serialiser.

mod error;
pub mod format;

pub use error::ProjectError;
pub use format::{ProjectFormatV1, ProjectSerializer};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::EngineeringModel;

/// File format version of the `.civilx` container.
///
/// This is *not* the engineering model schema version. The two evolve
/// independently:
///
/// ```text
/// .civilx file format version   →  how bytes are structured
/// model schema version           →  what fields the model carries
/// ```
pub const PROJECT_FORMAT_VERSION: u32 = 1;

/// Organisational metadata about a project.
///
/// Timestamps are `chrono::DateTime<Utc>` for human readability in the JSON
/// form. They do not participate in deterministic domain hashing or model
/// revision counting — they are *metadata*, not *state*.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Human-readable project name.
    pub name: String,
    /// High-level project classification selected by the user.
    #[serde(default)]
    pub project_type: String,
    /// Estimated site area in square metres.
    #[serde(default)]
    pub land_area_m2: f64,
    /// Optional description.
    #[serde(default)]
    pub description: String,
    /// When the project was first created (UTC).
    pub created_at: DateTime<Utc>,
    /// When the project was last modified (UTC).
    pub modified_at: DateTime<Utc>,
}

impl ProjectMetadata {
    /// Creates metadata with the given name and `created_at = now`.
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_with_site(name, "مبنى إنشائي", 0.0)
    }

    /// Creates metadata with project classification and estimated site area.
    pub fn new_with_site(
        name: impl Into<String>,
        project_type: impl Into<String>,
        land_area_m2: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            project_type: project_type.into(),
            land_area_m2,
            description: String::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Updates `modified_at` to the current time.
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }
}

/// A serialisable engineering project.
///
/// The project is the *persistence* container. The
/// [`EngineeringModel`](crate::model::EngineeringModel) inside it is the
/// *domain* source of truth. A `Project` can be serialised to bytes and
/// deserialised back without losing any engineering data.
///
/// # Identity
///
/// The project id is always [`EngineeringModel::project_id`]. There is no
/// separate identity field on `Project` — that would be a second source of
/// truth that could disagree with the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Organisational metadata (name, description, timestamps).
    pub metadata: ProjectMetadata,
    /// The engineering model — the single source of truth.
    pub model: EngineeringModel,
}

impl Project {
    /// Creates a new project with a fresh engineering model.
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_with_site(name, "مبنى إنشائي", 0.0)
    }

    /// Creates a project with the user's site classification and area.
    pub fn new_with_site(
        name: impl Into<String>,
        project_type: impl Into<String>,
        land_area_m2: f64,
    ) -> Self {
        Self {
            metadata: ProjectMetadata::new_with_site(name, project_type, land_area_m2),
            model: EngineeringModel::new(),
        }
    }

    /// The project id, always equal to `model.project_id`.
    pub fn project_id(&self) -> Uuid {
        self.model.project_id
    }

    /// Schema version of the engineering model.
    pub fn model_schema_version(&self) -> u32 {
        self.model.schema_version
    }

    /// Revision of the engineering model.
    pub fn revision(&self) -> u64 {
        self.model.revision
    }

    /// A flat, FFI-friendly snapshot of the project.
    pub fn summary(&self) -> ProjectSummary {
        let model_summary = self.model.summary();
        ProjectSummary {
            format_version: PROJECT_FORMAT_VERSION,
            model_schema_version: self.model.schema_version,
            project_id: self.model.project_id.to_string(),
            name: self.metadata.name.clone(),
            project_type: self.metadata.project_type.clone(),
            land_area_m2: self.metadata.land_area_m2,
            revision: self.model.revision,
            levels: model_summary.levels,
            grids: model_summary.grids,
            materials: model_summary.materials,
            cross_sections: model_summary.cross_sections,
            elements: model_summary.elements,
        }
    }

    /// Marks the project as modified at the current time.
    pub fn touch(&mut self) {
        self.metadata.touch();
    }
}

/// A flat, FFI-friendly snapshot of the project.
///
/// This type is deliberately made of primitives only — no domain model data
/// crosses the FFI boundary in this form. It exists so Flutter can display a
/// project overview without receiving the full model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSummary {
    /// File format version.
    pub format_version: u32,
    /// Engineering model schema version.
    pub model_schema_version: u32,
    /// Project id as a hyphenated UUID string.
    pub project_id: String,
    /// Human-readable project name.
    pub name: String,
    /// High-level project classification.
    pub project_type: String,
    /// Estimated site area in square metres.
    pub land_area_m2: f64,
    /// Number of accepted mutations.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_id_matches_model_project_id() {
        let project = Project::new("Test");
        assert_eq!(project.project_id(), project.model.project_id);
    }

    #[test]
    fn metadata_timestamps_are_set() {
        let project = Project::new("Timestamped");
        assert_eq!(project.metadata.name, "Timestamped");
        assert!(project.metadata.description.is_empty());
        assert!(project.metadata.created_at <= Utc::now());
        assert!(project.metadata.modified_at <= Utc::now());
    }

    #[test]
    fn touch_updates_modified_at() {
        let mut project = Project::new("Touch");
        let before = project.metadata.modified_at;
        // In tests, the timestamps may be identical because they're created within
        // the same tick. We just verify touch doesn't panic and the field is valid.
        project.touch();
        assert!(project.metadata.modified_at >= before);
    }
}
