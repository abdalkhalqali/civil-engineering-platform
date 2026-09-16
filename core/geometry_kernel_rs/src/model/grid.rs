use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::units::Length;

/// Direction of a grid line in plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GridDirection {
    /// The line runs parallel to the global `X` axis.
    AlongX,
    /// The line runs parallel to the global `Y` axis.
    AlongY,
}

impl GridDirection {
    pub const fn is_parallel_to_x(self) -> bool {
        matches!(self, GridDirection::AlongX)
    }

    pub const fn is_parallel_to_y(self) -> bool {
        matches!(self, GridDirection::AlongY)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            GridDirection::AlongX => "along_x",
            GridDirection::AlongY => "along_y",
        }
    }
}

impl std::fmt::Display for GridDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grid line of the model.
///
/// The model only stores the *data* of the grid system — its lines, their names and
/// their position. Drawing grids, extents and bubbles belongs to a later
/// visualisation step.
///
/// Names are free text so the usual conventions are expressible: letter lines
/// (`A`, `B`, `C`, typically running along `Y`) and numbered lines (`1`, `2`, `3`,
/// typically running along `X`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    /// Stable identity of the grid line.
    pub id: Uuid,
    /// Label of the line, for example `A` or `3`.
    pub name: String,
    /// Direction of the line in plan.
    pub direction: GridDirection,
    /// Signed coordinate of the line on the axis perpendicular to `direction`:
    /// the `Y` coordinate of an `AlongX` line, the `X` coordinate of an `AlongY`
    /// line. Signed, because a grid may extend into negative coordinates.
    pub offset: Length,
}

impl Grid {
    /// A line parallel to `X`, placed at the `Y` coordinate `offset`.
    pub fn along_x(name: impl Into<String>, offset: Length) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            direction: GridDirection::AlongX,
            offset,
        }
    }

    /// A line parallel to `Y`, placed at the `X` coordinate `offset`.
    pub fn along_y(name: impl Into<String>, offset: Length) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            direction: GridDirection::AlongY,
            offset,
        }
    }

    /// Same line with an explicit identity.
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    /// The coordinate the line occupies, in metres.
    pub const fn offset_meters(&self) -> f64 {
        self.offset.meters()
    }
}
