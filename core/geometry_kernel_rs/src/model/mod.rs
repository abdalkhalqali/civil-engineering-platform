//! The engineering model: the single source of truth of the platform.
//!
//! ```text
//! EngineeringModel
//!   ├── levels          → Level
//!   ├── grids           → Grid
//!   ├── materials       → Material
//!   ├── cross_sections  → CrossSection
//!   └── elements        → Element (Column | Beam | Slab | Wall | Foundation)
//! ```
//!
//! Everything else the platform will grow later (3D geometry, 2D views, the
//! analytical model, quantities, BOQ, reports, BIM/IFC export, AI commands) is
//! derived from this model — never the other way round. The model does not depend
//! on Flutter, on a database or on a renderer.
//!
//! # Lookup
//!
//! All collections are [`IdMap`] — an id-keyed map, i.e. `ID → entity` in
//! `O(log n)` with no linear scans anywhere. See [`storage`] for why a
//! [`std::collections::BTreeMap`] was chosen over a [`std::collections::HashMap`].

mod cross_section;
mod engineering_model;
mod grid;
mod level;
mod material;
// `pub` because the generated bridge code addresses mirrored types at their
// defining path (`crate::model::summary::ModelSummary`), not through a re-export.
mod storage;
pub mod summary;
mod validation;

pub use cross_section::{CrossSection, IBeamProfile, ProfileType};
pub use engineering_model::{EngineeringModel, MODEL_SCHEMA_VERSION};
pub use grid::{Grid, GridDirection};
pub use level::Level;
pub use material::{Material, MaterialType};
pub use storage::IdMap;
pub use summary::ModelSummary;
pub use validation::{ValidationCode, ValidationIssue, ValidationMode, ValidationReport};
