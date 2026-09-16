use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::{BaseElement, Justification};
use crate::math::{Point3D, Vector3D};
use crate::units::Length;

/// A horizontal (or inclined) member spanning between two points.
///
/// The member geometry that the engineering model owns is its axis
/// (`start_point` .. `end_point`), its cross section reference, its material
/// reference and the level it is placed against. Its length is **derived** from
/// the axis — it is never stored, so it can never disagree with the points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralBeam {
    /// Identity, naming and placement of the element.
    pub base: BaseElement,
    /// Level the beam is placed against, together with `z_justification`.
    pub reference_level_id: Uuid,
    /// Start of the member axis.
    pub start_point: Point3D,
    /// End of the member axis.
    pub end_point: Point3D,
    /// Cross section of the beam.
    pub cross_section_id: Uuid,
    /// Material of the beam.
    pub material_id: Uuid,
    /// How the section is aligned with `reference_level_id`.
    pub z_justification: Justification,
}

impl StructuralBeam {
    /// Builds a beam with its section centred on the reference level.
    pub fn new(
        base: BaseElement,
        reference_level_id: Uuid,
        start_point: Point3D,
        end_point: Point3D,
        cross_section_id: Uuid,
        material_id: Uuid,
    ) -> Self {
        Self {
            base,
            reference_level_id,
            start_point,
            end_point,
            cross_section_id,
            material_id,
            z_justification: Justification::Center,
        }
    }

    /// Same beam with an explicit vertical justification.
    pub fn with_justification(mut self, justification: Justification) -> Self {
        self.z_justification = justification;
        self
    }

    /// Axis from start to end.
    pub fn axis(&self) -> Vector3D {
        self.start_point.offset_to(&self.end_point)
    }

    /// Derived length of the member.
    pub fn length(&self) -> Length {
        self.start_point.distance_to(&self.end_point)
    }

    /// Derived direction of the member, `None` when the axis is degenerate.
    pub fn direction(&self) -> Option<Vector3D> {
        self.axis().normalized()
    }

    /// `true` when the axis has no length, which is not a valid member.
    pub fn is_degenerate(&self) -> bool {
        self.axis().is_zero()
    }
}
