use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::{BaseElement, PlanBoundary};
use crate::units::Length;

/// A plate element: a thickness plus an outline in plan, placed on a level.
///
/// The outline is the [`PlanBoundary`] abstraction — an ordered ring of points.
/// The slab therefore has everything the model needs (identity, level, thickness,
/// material, extent in plan) without pulling a mesh or a solid into this step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralSlab {
    /// Identity, naming and placement of the element.
    pub base: BaseElement,
    /// Level the slab belongs to; its top face is the level elevation by default.
    pub level_id: Uuid,
    /// Thickness of the slab.
    pub thickness: Length,
    /// Material of the slab.
    pub material_id: Uuid,
    /// Closed outline of the slab in plan.
    pub boundary: PlanBoundary,
}

impl StructuralSlab {
    /// Builds a slab on `level_id`.
    pub fn new(
        base: BaseElement,
        level_id: Uuid,
        thickness: Length,
        material_id: Uuid,
        boundary: PlanBoundary,
    ) -> Self {
        Self {
            base,
            level_id,
            thickness,
            material_id,
            boundary,
        }
    }

    /// Derived area of the outline, by the shoelace formula on the plan projection.
    ///
    /// Derived on purpose: the area is not stored, so it cannot contradict the
    /// boundary.
    pub fn boundary_area(&self) -> crate::units::Area {
        let vertices = self.boundary.vertices();
        let mut twice_area = 0.0;
        for index in 0..vertices.len() {
            let current = vertices[index];
            let next = vertices[(index + 1) % vertices.len()];
            twice_area +=
                current.x.meters() * next.y.meters() - next.x.meters() * current.y.meters();
        }
        crate::units::Area::from_square_meters((twice_area / 2.0).abs())
    }
}
