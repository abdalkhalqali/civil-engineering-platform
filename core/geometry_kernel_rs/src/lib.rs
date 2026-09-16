//! Engineering geometry kernel of the Civil Engineering Platform.
//!
//! # Architecture
//!
//! ```text
//! api/        Flutter boundary (flutter_rust_bridge) — flat, minimal
//! commands/   named mutations of the model (extension point)
//! model/      EngineeringModel — the single source of truth
//! elements/   building elements and the abstractions they share
//! math/       Point3D, Vector3D, Transform3D
//! units/      internal SI quantities (Length, Stress, Angle, ...)
//! error/      invariant violations reported by mutations
//! ```
//!
//! Rules that hold for every module:
//!
//! * **The engineering model is the source of truth.** Geometry, drawings,
//!   analytical models, quantities, reports and exports will all be *derived* from
//!   it — never the other way round.
//! * **SI units only.** A value is a [`units::Length`], a [`units::Stress`], ... and
//!   never a bare number whose unit has to be guessed from a field name.
//! * **Relationships are ids.** An element references a level, a material or a cross
//!   section by identity; it never embeds a copy that could disagree with the
//!   original.
//! * **Derived, not stored.** Lengths, areas and second moments of area are computed
//!   from the data that defines them, so they cannot contradict it.
//! * **Platform independent.** Nothing here depends on Flutter, on a UI, on a
//!   database or on a renderer.
//!
//! Scope of the current step: the model core (units, math, elements, model,
//! validation, serialization). Meshes, solids, booleans, CAD geometry, 3D
//! rendering, structural analysis, `.civilx` files and databases are explicitly
//! later steps.

pub mod api;
pub mod commands;
pub mod elements;
pub mod error;
pub mod math;
pub mod model;
pub mod units;

mod frb_generated;
