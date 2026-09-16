use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::BaseElement;
use crate::math::Point3D;
use crate::units::Length;

/// A vertical plate element: an axis in plan, a thickness and two levels.
///
/// The wall is intentionally described only by data the model owns today:
/// base and top levels (both by id), the axis of its centre plane, its thickness
/// and its material.
///
/// Extension points kept open on purpose, none of them implemented yet:
/// * **openings** — they will reference the wall by id from a separate collection, or
///   be added as an `openings` field without changing identity or references;
/// * **reinforcement** — a later layer keyed by element id;
/// * **analytical representation** — a separate model derived from this wall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralWall {
    /// Identity, naming and placement of the element.
    pub base: BaseElement,
    /// Level at the bottom of the wall.
    pub base_level_id: Uuid,
    /// Level at the top of the wall.
    pub top_level_id: Uuid,
    /// Start of the wall centre plane axis.
    pub start_point: Point3D,
    /// End of the wall centre plane axis.
    pub end_point: Point3D,
    /// Thickness of the wall.
    pub thickness: Length,
    /// Material of the wall.
    pub material_id: Uuid,
}

impl StructuralWall {
    /// Builds a wall spanning `base_level_id` .. `top_level_id`.
    pub fn new(
        base: BaseElement,
        base_level_id: Uuid,
        top_level_id: Uuid,
        start_point: Point3D,
        end_point: Point3D,
        thickness: Length,
        material_id: Uuid,
    ) -> Self {
        Self {
            base,
            base_level_id,
            top_level_id,
            start_point,
            end_point,
            thickness,
            material_id,
        }
    }

    /// Derived length of the wall axis.
    pub fn length(&self) -> Length {
        self.start_point.distance_to(&self.end_point)
    }

    /// `true` when the axis has no length, which is not a valid wall.
    pub fn is_degenerate(&self) -> bool {
        self.start_point == self.end_point
    }
}
