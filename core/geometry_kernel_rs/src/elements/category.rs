use serde::{Deserialize, Serialize};

/// Classification of everything an engineering model can contain.
///
/// The two groups are deliberately distinguishable, because a physical building
/// element takes part in quantities, analysis and drawings, while a reference
/// element only organises the model:
///
/// * **physical**: `Column`, `Beam`, `Slab`, `Wall`, `Foundation`
/// * **reference**: `Level`, `Grid`
///
/// Future variants (`Stair`, `Opening`, `Roof`, `Pile`, `RetainingWall`, `Road`,
/// `Terrain`, `SurveyPoint`) are added here; every consumer that matches on this
/// enum is expected to keep working through [`ElementCategory::is_physical`] and
/// [`ElementCategory::is_reference`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementCategory {
    // Physical building elements.
    Column,
    Beam,
    Slab,
    Wall,
    Foundation,
    // Reference / organisational elements.
    Level,
    Grid,
}

impl ElementCategory {
    /// `true` for entities that represent real building fabric.
    pub const fn is_physical(self) -> bool {
        matches!(
            self,
            ElementCategory::Column
                | ElementCategory::Beam
                | ElementCategory::Slab
                | ElementCategory::Wall
                | ElementCategory::Foundation
        )
    }

    /// `true` for entities that only organise the model (levels, grids).
    pub const fn is_reference(self) -> bool {
        !self.is_physical()
    }

    /// Stable, human readable identifier used in messages and serialized metadata.
    pub const fn as_str(self) -> &'static str {
        match self {
            ElementCategory::Column => "column",
            ElementCategory::Beam => "beam",
            ElementCategory::Slab => "slab",
            ElementCategory::Wall => "wall",
            ElementCategory::Foundation => "foundation",
            ElementCategory::Level => "level",
            ElementCategory::Grid => "grid",
        }
    }
}

impl std::fmt::Display for ElementCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
