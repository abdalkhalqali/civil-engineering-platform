//! Mathematical foundation of the engineering model.
//!
//! # Global model coordinate system
//!
//! The kernel works in a single right-handed, Cartesian, three-dimensional
//! coordinate system with the following documented meaning:
//!
//! ```text
//!   Z  vertical (up), positive above the project datum
//!   │
//!   │
//!   └────── X   first horizontal direction (typically the main building direction)
//!  /
//! Y            second horizontal direction (perpendicular to X, in plan)
//! ```
//!
//! * `X` — first horizontal direction.
//! * `Y` — second horizontal direction.
//! * `Z` — vertical direction; `Z = 0` is the project datum defined by the model's levels.
//!
//! Consequences that the rest of the kernel relies on:
//!
//! * Plan rotation of an element happens **about +Z** (see [`Rotation3D`]).
//! * Vertical elements (columns, walls) measure their extent along `Z` through
//!   the elevations of the levels they reference.
//! * There is deliberately **no geographic (GIS) coordinate system** in this step.
//!
//! # Scope
//!
//! Only the types the engineering model needs today live here: points, vectors
//! and transforms. Curves, surfaces, solids, booleans and meshes belong to the
//! geometry kernel step and are intentionally absent.

mod point3d;
mod transform3d;
mod vector3d;

pub use point3d::Point3D;
pub use transform3d::{Rotation3D, Transform3D};
pub use vector3d::Vector3D;
