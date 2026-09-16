use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::BaseElement;
use crate::math::Transform3D;
use crate::units::{Angle, Length};

/// A vertical load-bearing member, described by the levels it spans.
///
/// The column is defined by *references*, never by copies:
/// `base_level_id` / `top_level_id` point at levels of the model,
/// `cross_section_id` at a cross section, `material_id` at a material. Moving a
/// level therefore moves every column that references it, and a material value
/// exists exactly once in the model.
///
/// The two levels define the span; the offsets are signed adjustments applied to
/// the level elevations (positive up, negative down, for example for a column cast
/// into a foundation). The extent in metres is
/// `[base_elevation + base_offset, top_elevation + top_offset]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralColumn {
    /// Identity, naming and placement of the element.
    pub base: BaseElement,
    /// Level at the bottom of the column.
    pub base_level_id: Uuid,
    /// Level at the top of the column.
    pub top_level_id: Uuid,
    /// Signed adjustment of the bottom end relative to its level elevation.
    pub base_offset: Length,
    /// Signed adjustment of the top end relative to its level elevation.
    pub top_offset: Length,
    /// Cross section of the column.
    pub cross_section_id: Uuid,
    /// Material of the column.
    pub material_id: Uuid,
}

impl StructuralColumn {
    /// Builds a column that references `base_level_id` .. `top_level_id`, with no
    /// offsets and no plan rotation.
    pub fn new(
        base: BaseElement,
        base_level_id: Uuid,
        top_level_id: Uuid,
        cross_section_id: Uuid,
        material_id: Uuid,
    ) -> Self {
        Self {
            base,
            base_level_id,
            top_level_id,
            base_offset: Length::ZERO,
            top_offset: Length::ZERO,
            cross_section_id,
            material_id,
        }
    }

    /// Same column with the two end offsets applied.
    pub fn with_offsets(mut self, base_offset: Length, top_offset: Length) -> Self {
        self.base_offset = base_offset;
        self.top_offset = top_offset;
        self
    }

    /// Same column, rotated in plan.
    ///
    /// The rotation lives in [`BaseElement::transform`] — the single place where
    /// placement is stored — so there is no separate `rotation_angle` field that
    /// could contradict it.
    pub fn with_plan_rotation(mut self, angle: Angle) -> Self {
        self.base.transform =
            Transform3D::at_with_plan_rotation(self.base.transform.position(), angle);
        self
    }

    /// Elevation of the bottom end, given the elevation of its base level.
    pub fn effective_base_elevation(&self, base_level_elevation: Length) -> Length {
        base_level_elevation + self.base_offset
    }

    /// Elevation of the top end, given the elevation of its top level.
    pub fn effective_top_elevation(&self, top_level_elevation: Length) -> Length {
        top_level_elevation + self.top_offset
    }

    /// Bottom and top elevations, given the elevations of both referenced levels.
    pub fn extent(
        &self,
        base_level_elevation: Length,
        top_level_elevation: Length,
    ) -> (Length, Length) {
        (
            self.effective_base_elevation(base_level_elevation),
            self.effective_top_elevation(top_level_elevation),
        )
    }
}
