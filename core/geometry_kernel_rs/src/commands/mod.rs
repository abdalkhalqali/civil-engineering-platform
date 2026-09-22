//! The extension point for model mutations.
//!
//! A full command system — undo/redo stacks, an event journal, a serialized
//! command history — is **not** part of this step. What this module fixes is the
//! shape those commands will follow, so the model never has to be redesigned for
//! them:
//!
//! ```text
//! Command ──apply──► EngineeringModel ──revision──► Event / History (later)
//! ```
//!
//! Design rules that make that future step cheap:
//!
//! * a command is **data** and serializable, so it can be journalled later;
//! * a command applies itself **through the model's `add_*` methods**, so it can
//!   never bypass the invariant checks nor forget to bump the revision;
//! * a command returns the identity of what it created or changed, which is what an
//!   event log needs to record.
//!
//! Future commands (`CreateColumnCommand`, `CreateBeamCommand`, `MoveElementCommand`,
//! `DeleteElementCommand`, `ChangePropertyCommand`, ...) implement the same trait.

pub mod elements;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ModelError;
use crate::model::{EngineeringModel, Level};
use crate::units::Length;

/// A named mutation of an [`EngineeringModel`].
pub trait ModelCommand {
    /// Stable name of the command, for example `add_level`.
    fn name(&self) -> &'static str;

    /// Applies the command and returns the identity of the affected entity.
    fn apply(self, model: &mut EngineeringModel) -> Result<Uuid, ModelError>;
}

/// Adds a level to the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddLevelCommand {
    /// Name of the new level.
    pub name: String,
    /// Elevation of the new level, in internal metres.
    pub elevation: Length,
}

impl AddLevelCommand {
    /// Builds the command.
    pub fn new(name: impl Into<String>, elevation: Length) -> Self {
        Self {
            name: name.into(),
            elevation,
        }
    }
}

impl ModelCommand for AddLevelCommand {
    fn name(&self) -> &'static str {
        "add_level"
    }

    fn apply(self, model: &mut EngineeringModel) -> Result<Uuid, ModelError> {
        model.add_level(Level::new(self.name, self.elevation))
    }
}
