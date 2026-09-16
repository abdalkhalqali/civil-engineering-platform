use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::{BaseElement, PlanBoundary};
use crate::units::Length;

/// Shape family of a foundation element.
///
/// This is a classification only. Bearing capacity, settlement, soil interaction
/// and reinforcement design are explicitly **out of scope** for the model core and
/// will not appear as fields here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum FoundationType {
    /// Single footing under one column.
    #[default]
    Isolated,
    /// Continuous footing under a wall or a line of columns.
    Strip,
    /// Raft / mat foundation covering several supports.
    Mat,
    /// Footing that ties a group of piles.
    PileCap,
}

impl FoundationType {
    pub const fn as_str(self) -> &'static str {
        match self {
            FoundationType::Isolated => "isolated",
            FoundationType::Strip => "strip",
            FoundationType::Mat => "mat",
            FoundationType::PileCap => "pile_cap",
        }
    }
}

impl std::fmt::Display for FoundationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A foundation element: a thickness, a type and a footprint in plan.
///
/// `level_id` is the level the footing is placed against (typically the top of the
/// foundation), and `footprint` reuses the same [`PlanBoundary`] abstraction as
/// slabs instead of inventing a second boundary representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Foundation {
    /// Identity, naming and placement of the element.
    pub base: BaseElement,
    /// Reference level of the foundation.
    pub level_id: Uuid,
    /// Thickness of the foundation element.
    pub thickness: Length,
    /// Material of the foundation element.
    pub material_id: Uuid,
    /// Shape family of the foundation.
    pub foundation_type: FoundationType,
    /// Closed outline of the foundation in plan.
    pub footprint: PlanBoundary,
}

impl Foundation {
    /// Builds a foundation element.
    pub fn new(
        base: BaseElement,
        level_id: Uuid,
        thickness: Length,
        material_id: Uuid,
        foundation_type: FoundationType,
        footprint: PlanBoundary,
    ) -> Self {
        Self {
            base,
            level_id,
            thickness,
            material_id,
            foundation_type,
            footprint,
        }
    }
}
