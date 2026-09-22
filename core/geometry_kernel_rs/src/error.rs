//! Errors produced when a mutation would break a model invariant.
//!
//! This type lives at the crate root because both the element layer (for example
//! building a boundary) and the model layer (adding elements) report it.
//!
//! Non-blocking problems that are *reported* instead of rejected while editing are
//! described by [`crate::model::ValidationReport`].

use std::fmt;

use uuid::Uuid;

/// The kind of model entity an element reference points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ReferenceKind {
    Level,
    Grid,
    Material,
    CrossSection,
}

impl ReferenceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            ReferenceKind::Level => "level",
            ReferenceKind::Grid => "grid",
            ReferenceKind::Material => "material",
            ReferenceKind::CrossSection => "cross_section",
        }
    }
}

impl fmt::Display for ReferenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rejected mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    /// The id is already used by another entity of the model.
    DuplicateId { id: Uuid },
    /// An element points at an entity that is not part of the model.
    MissingReference {
        element_id: Uuid,
        reference: ReferenceKind,
        missing_id: Uuid,
    },
    /// A boundary was built from an unusable set of points.
    InvalidBoundary { reason: String },
    /// The entity a command names is not part of the model.
    MissingEntity { id: Uuid },
    /// A command was asked to do something the element kind cannot do, for example
    /// changing the top level of a slab.
    UnsupportedElementKind { id: Uuid, reason: String },
    /// An extent (a height, an axis, a size) is not usable as engineering data.
    InvalidExtent { reason: String },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::DuplicateId { id } => {
                write!(f, "id {id} is already used by another entity of the model")
            }
            ModelError::MissingReference {
                element_id,
                reference,
                missing_id,
            } => write!(
                f,
                "element {element_id} references {reference} {missing_id}, which does not exist in the model"
            ),
            ModelError::InvalidBoundary { reason } => {
                write!(f, "invalid boundary: {reason}")
            }
            ModelError::MissingEntity { id } => {
                write!(f, "entity {id} does not exist in the model")
            }
            ModelError::UnsupportedElementKind { id, reason } => {
                write!(f, "element {id} cannot be changed that way: {reason}")
            }
            ModelError::InvalidExtent { reason } => {
                write!(f, "invalid extent: {reason}")
            }
        }
    }
}

impl std::error::Error for ModelError {}
