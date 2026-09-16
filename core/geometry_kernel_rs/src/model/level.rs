use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::units::Length;

/// A horizontal reference plane of the building, identified by an elevation.
///
/// The elevation is a [`Length`], i.e. internal metres, and its sign is meaningful:
/// `Z = 0` is the project datum, so a basement level is a level with a negative
/// elevation.
///
/// A *datum* / reference-system description is not part of the level yet; it will be
/// added as an explicit field when the project datum becomes configurable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Level {
    /// Stable identity of the level.
    pub id: Uuid,
    /// Human readable name, for example `Level 1`.
    pub name: String,
    /// Height of the level relative to the project datum (`Z = 0`), in metres.
    pub elevation: Length,
}

impl Level {
    /// Creates a level with a freshly generated identity.
    pub fn new(name: impl Into<String>, elevation: Length) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            elevation,
        }
    }

    /// Creates a level with an explicit identity.
    pub fn with_id(id: Uuid, name: impl Into<String>, elevation: Length) -> Self {
        Self {
            id,
            name: name.into(),
            elevation,
        }
    }

    /// Elevation in metres, for convenience in assertions and reports.
    pub const fn elevation_meters(&self) -> f64 {
        self.elevation.meters()
    }

    /// `true` when this level sits above `other`.
    pub fn is_above(&self, other: &Level) -> bool {
        self.elevation > other.elevation
    }
}
