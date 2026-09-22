//! Undo/redo over engineering state.
//!
//! History stores [`HistoryEntry`] values — a label plus the two directions of one
//! change ([`StateOp`] lists). It never stores a viewport, a mesh or a whole-file
//! copy of the model, so:
//!
//! * **undo** restores *model data*, not pixels;
//! * **redo** re-applies the recorded forward operations, so the restored model is
//!   identical to the original — including identities;
//! * camera movement never reaches this module, because a camera is not an
//!   engineering change.

use crate::commands::elements::StateOp;
use crate::error::ModelError;
use crate::model::EngineeringModel;

/// One reversible change of the model.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    /// Human readable description, shown in the status bar.
    pub label: String,
    /// Operations that apply the change.
    pub forward: Vec<StateOp>,
    /// Operations that take the change back.
    pub inverse: Vec<StateOp>,
}

impl HistoryEntry {
    pub fn new(label: impl Into<String>, forward: Vec<StateOp>, inverse: Vec<StateOp>) -> Self {
        Self {
            label: label.into(),
            forward,
            inverse,
        }
    }
}

/// Bounded undo/redo stack pair.
#[derive(Debug, Clone)]
pub struct CommandHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    limit: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(200)
    }
}

impl CommandHistory {
    /// A history that keeps at most `limit` reversible changes.
    pub fn new(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: limit.max(1),
        }
    }

    /// Records a change, discarding any redo branch.
    pub fn record(&mut self, entry: HistoryEntry) {
        self.undo.push(entry);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// Labels of the recorded changes, oldest first.
    pub fn labels(&self) -> Vec<String> {
        self.undo.iter().map(|entry| entry.label.clone()).collect()
    }

    /// Takes the model back by one change. Returns the label that was undone.
    pub fn undo(&mut self, model: &mut EngineeringModel) -> Result<Option<String>, ModelError> {
        let Some(entry) = self.undo.pop() else {
            return Ok(None);
        };
        for operation in entry.inverse.iter().rev() {
            operation.clone().apply(model)?;
        }
        let label = entry.label.clone();
        self.redo.push(entry);
        Ok(Some(label))
    }

    /// Re-applies the last undone change. Returns the label that was redone.
    pub fn redo(&mut self, model: &mut EngineeringModel) -> Result<Option<String>, ModelError> {
        let Some(entry) = self.redo.pop() else {
            return Ok(None);
        };
        for operation in entry.forward.iter() {
            operation.clone().apply(model)?;
        }
        let label = entry.label.clone();
        self.undo.push(entry);
        Ok(Some(label))
    }

    /// Forgets everything (used when a project is loaded).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}
