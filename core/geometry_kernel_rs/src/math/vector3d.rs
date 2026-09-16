use serde::{Deserialize, Serialize};

use crate::units::Length;

/// A displacement in the global model coordinate system: the offset between two
/// [`crate::math::Point3D`] positions.
///
/// Components are [`Length`]s, so `magnitude()` is a length and offsets can be
/// added to points ([`crate::math::Point3D::translated`]) without unit guessing.
///
/// Products of two displacements (dot/cross, i.e. areas and normals) belong to the
/// geometry kernel step and are deliberately not defined here: their results would
/// not be lengths, and this type refuses to pretend otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3D {
    pub x: Length,
    pub y: Length,
    pub z: Length,
}

impl Vector3D {
    /// The null displacement.
    pub const ZERO: Self = Self {
        x: Length::ZERO,
        y: Length::ZERO,
        z: Length::ZERO,
    };

    /// A unit vector along the global `X` axis (length 1 m).
    pub const X: Self = Self {
        x: Length::from_meters(1.0),
        y: Length::ZERO,
        z: Length::ZERO,
    };

    /// A unit vector along the global `Y` axis (length 1 m).
    pub const Y: Self = Self {
        x: Length::ZERO,
        y: Length::from_meters(1.0),
        z: Length::ZERO,
    };

    /// A unit vector along the global `Z` axis (length 1 m).
    pub const Z: Self = Self {
        x: Length::ZERO,
        y: Length::ZERO,
        z: Length::from_meters(1.0),
    };

    pub const fn new(x: Length, y: Length, z: Length) -> Self {
        Self { x, y, z }
    }

    /// Builds a vector from components already expressed in metres.
    pub const fn from_meters(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Length::from_meters(x),
            y: Length::from_meters(y),
            z: Length::from_meters(z),
        }
    }

    /// Builds a vector from components expressed in millimetres.
    pub fn from_millimeters(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Length::from_millimeters(x),
            y: Length::from_millimeters(y),
            z: Length::from_millimeters(z),
        }
    }

    /// Raw components in metres, in `(x, y, z)` order.
    pub const fn components_meters(&self) -> (f64, f64, f64) {
        (self.x.meters(), self.y.meters(), self.z.meters())
    }

    /// The length of the vector.
    pub fn magnitude(&self) -> Length {
        Length::from_meters(
            (self.x.meters().powi(2) + self.y.meters().powi(2) + self.z.meters().powi(2)).sqrt(),
        )
    }

    /// The same vector scaled by a dimensionless factor.
    pub fn scaled(&self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    /// The vector reduced to length 1 m, i.e. its direction.
    ///
    /// Returns `None` for a null vector, which has no direction.
    pub fn normalized(&self) -> Option<Self> {
        let magnitude = self.magnitude();
        if magnitude.is_zero() {
            return None;
        }
        Some(self.scaled(1.0 / magnitude.meters()))
    }

    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

impl Default for Vector3D {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Vector3D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Mul<f64> for Vector3D {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        self.scaled(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnitude_and_normalization() {
        let v = Vector3D::from_meters(3.0, 4.0, 0.0);

        assert_eq!(v.magnitude().meters(), 5.0);
        assert!((v.normalized().unwrap().magnitude().meters() - 1.0).abs() < 1e-15);
    }

    #[test]
    fn null_vector_has_no_direction() {
        assert!(Vector3D::ZERO.normalized().is_none());
        assert!(Vector3D::ZERO.is_zero());
    }
}
