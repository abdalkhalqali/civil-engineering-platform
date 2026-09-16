use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::math::Point3D;
use crate::units::Length;

/// Tolerance used when checking that a boundary is horizontal: 1 nm in metres.
const PLANARITY_TOLERANCE_METERS: f64 = 1e-9;

/// The closed outline of a horizontal surface element, expressed in plan.
///
/// This is the deliberately simple boundary abstraction for slabs and
/// foundations: an ordered ring of [`Point3D`] vertices in the global coordinate
/// system, all at the same height. It is **not** a mesh, not a surface and not a
/// CAD loop — those belong to the geometry kernel step. When that step lands, a
/// boundary can be replaced by a reference to a geometry id without changing the
/// element's identity or its relationships.
///
/// The ring is implicitly closed: the last vertex connects back to the first one,
/// so the first vertex must not be repeated at the end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanBoundary {
    vertices: Vec<Point3D>,
}

impl PlanBoundary {
    /// Number of vertices a closed outline needs at least.
    pub const MIN_VERTICES: usize = 3;

    /// Builds a boundary from an ordered ring of at least three vertices.
    pub fn new(vertices: Vec<Point3D>) -> Result<Self, ModelError> {
        if vertices.len() < Self::MIN_VERTICES {
            return Err(ModelError::InvalidBoundary {
                reason: format!(
                    "a closed boundary needs at least {} vertices, got {}",
                    Self::MIN_VERTICES,
                    vertices.len()
                ),
            });
        }
        Ok(Self { vertices })
    }

    /// Builds an axis-aligned rectangular boundary.
    ///
    /// `origin` is the corner closest to the global origin, `width` extends along
    /// `+X` and `depth` extends along `+Y`; every vertex keeps the `origin` height.
    pub fn rectangle(origin: Point3D, width: Length, depth: Length) -> Self {
        let z = origin.z;
        let corner = |dx: f64, dy: f64| Point3D {
            x: origin.x + Length::from_meters(dx),
            y: origin.y + Length::from_meters(dy),
            z,
        };
        Self {
            vertices: vec![
                corner(0.0, 0.0),
                corner(width.meters(), 0.0),
                corner(width.meters(), depth.meters()),
                corner(0.0, depth.meters()),
            ],
        }
    }

    /// The ordered ring of vertices; the ring is closed implicitly.
    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// `true` when every vertex lies on one horizontal plane.
    pub fn is_horizontal(&self) -> bool {
        let Some(first) = self.vertices.first() else {
            return false;
        };
        self.vertices.iter().all(|vertex| {
            (vertex.z.meters() - first.z.meters()).abs() <= PLANARITY_TOLERANCE_METERS
        })
    }
}
