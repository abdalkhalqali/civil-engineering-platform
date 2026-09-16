use serde::{Deserialize, Serialize};

use crate::math::{Point3D, Vector3D};
use crate::units::Angle;

/// Orientation of an element's local axes relative to the global axes.
///
/// This is an enum rather than a bare angle so that further orientations can be
/// added later (for example `Euler { .. }` or a quaternion for inclined members)
/// without breaking existing data.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Rotation3D {
    /// Local axes aligned with the global axes.
    #[default]
    Identity,
    /// Rotation about the global `+Z` axis (plan rotation), counter-clockwise
    /// positive, measured from the global `+X` axis.
    AroundZ(Angle),
}

impl Rotation3D {
    /// The neutral rotation.
    pub const IDENTITY: Self = Rotation3D::Identity;

    /// The plan rotation, `0 rad` when the rotation is the neutral one.
    pub const fn angle_about_z(self) -> Angle {
        match self {
            Rotation3D::Identity => Angle::ZERO,
            Rotation3D::AroundZ(angle) => angle,
        }
    }

    pub const fn is_identity(self) -> bool {
        matches!(self, Rotation3D::Identity)
    }
}

/// Where an element sits in the model and how it is oriented.
///
/// `Transform3D` is *placement*, not geometry: it carries the position of the
/// element's local origin ([`Point3D`], in metres) and its orientation
/// ([`Rotation3D`]). Element sizes stay where they belong — in the element's own
/// properties (thickness, profile dimensions, level references).
///
/// # Why there is no scale
///
/// A scale factor in a building model has no physical meaning and would create a
/// second source of truth for dimensions: a `400 mm` column scaled by `2` would
/// report a size that no property of the model states. Scaling is therefore
/// excluded by design; changing a size means changing the size property.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Transform3D {
    /// Position of the element's local origin in the global coordinate system.
    pub translation: Point3D,
    /// Orientation of the element's local axes.
    pub rotation: Rotation3D,
}

impl Transform3D {
    /// Placed at the global origin with no rotation.
    pub const IDENTITY: Self = Self {
        translation: Point3D::ORIGIN,
        rotation: Rotation3D::Identity,
    };

    /// No rotation, placed at `translation`.
    pub const fn at(translation: Point3D) -> Self {
        Self {
            translation,
            rotation: Rotation3D::Identity,
        }
    }

    /// Placed at `translation`, rotated about the global `+Z` axis.
    pub const fn at_with_plan_rotation(translation: Point3D, angle: Angle) -> Self {
        Self {
            translation,
            rotation: Rotation3D::AroundZ(angle),
        }
    }

    /// Rotated about the global `+Z` axis, placed at the global origin.
    pub const fn rotated_around_z(angle: Angle) -> Self {
        Self::at_with_plan_rotation(Point3D::ORIGIN, angle)
    }

    /// The element's position.
    pub const fn position(&self) -> Point3D {
        self.translation
    }

    /// The plan rotation about `+Z`.
    pub const fn plan_rotation(&self) -> Angle {
        self.rotation.angle_about_z()
    }

    /// The same placement, moved by an offset.
    pub fn translated_by(&self, offset: Vector3D) -> Self {
        Self {
            translation: self.translation.translated(offset),
            rotation: self.rotation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_transform_has_no_rotation() {
        assert!(Transform3D::IDENTITY.rotation.is_identity());
        assert_eq!(Transform3D::IDENTITY.plan_rotation().degrees(), 0.0);
    }

    #[test]
    fn plan_rotation_is_stored_in_radians() {
        let transform = Transform3D::rotated_around_z(Angle::from_degrees(90.0));

        assert!(transform.position().coordinates_meters() == (0.0, 0.0, 0.0));
        assert!((transform.plan_rotation().radians() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        assert!((transform.plan_rotation().degrees() - 90.0).abs() < 1e-12);
    }
}
