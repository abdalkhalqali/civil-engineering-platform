use serde::{Deserialize, Serialize};

use crate::math::Vector3D;
use crate::units::Length;

/// A position in the global model coordinate system (see [`crate::math`]).
///
/// Every component is a [`Length`], so a coordinate can never be an ambiguous
/// bare number: `Point3D::from_millimeters(0.0, 0.0, 3200.0)` is `z = 3.2 m`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    pub x: Length,
    pub y: Length,
    pub z: Length,
}

impl Point3D {
    /// The origin of the global coordinate system.
    pub const ORIGIN: Self = Self {
        x: Length::ZERO,
        y: Length::ZERO,
        z: Length::ZERO,
    };

    pub const fn new(x: Length, y: Length, z: Length) -> Self {
        Self { x, y, z }
    }

    /// Builds a point from coordinates already expressed in metres.
    pub const fn from_meters(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Length::from_meters(x),
            y: Length::from_meters(y),
            z: Length::from_meters(z),
        }
    }

    /// Builds a point from coordinates expressed in millimetres.
    pub fn from_millimeters(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Length::from_millimeters(x),
            y: Length::from_millimeters(y),
            z: Length::from_millimeters(z),
        }
    }

    /// Raw coordinates in metres, in `(x, y, z)` order.
    pub const fn coordinates_meters(&self) -> (f64, f64, f64) {
        (self.x.meters(), self.y.meters(), self.z.meters())
    }

    /// Straight distance to another point.
    pub fn distance_to(&self, other: &Self) -> Length {
        self.offset_to(other).magnitude()
    }

    /// Displacement from this point to `other`.
    pub fn offset_to(&self, other: &Self) -> Vector3D {
        Vector3D::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    /// Same point moved by an offset.
    pub fn translated(&self, offset: Vector3D) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
            z: self.z + offset.z,
        }
    }
}

impl Default for Point3D {
    fn default() -> Self {
        Self::ORIGIN
    }
}

impl std::ops::Sub for Point3D {
    type Output = Vector3D;

    /// The displacement that takes the right-hand point to the left-hand point.
    fn sub(self, rhs: Self) -> Vector3D {
        Vector3D::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Add<Vector3D> for Point3D {
    type Output = Point3D;

    fn add(self, rhs: Vector3D) -> Point3D {
        self.translated(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_uses_internal_metres() {
        let a = Point3D::from_meters(0.0, 0.0, 0.0);
        let b = Point3D::from_millimeters(3000.0, 4000.0, 0.0);

        assert_eq!(a.distance_to(&b).meters(), 5.0);
    }

    #[test]
    fn subtracting_points_gives_a_displacement() {
        let a = Point3D::from_meters(1.0, 2.0, 3.0);
        let b = Point3D::from_meters(3.0, 4.0, 3.0);

        let offset = a.offset_to(&b);
        // (2, 2, 0) has magnitude 2 * sqrt(2).
        assert!((offset.magnitude().meters() - 2.0 * std::f64::consts::SQRT_2).abs() < 1e-12);
        assert_eq!(b.translated(-offset), a);
    }
}
