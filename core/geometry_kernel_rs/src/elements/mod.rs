//! Building elements of the engineering model.
//!
//! Every element is *composition*, never inheritance: concrete element types
//! (column, beam, slab, wall, foundation) embed a [`BaseElement`] that carries
//! identity, naming and placement, and the model stores them as the [`Element`]
//! enum. There is no `Box<dyn Any>`, no trait-object soup and no duplicated base
//! fields.
//!
//! Relationships between elements are expressed with **ids** only
//! ([`BaseElement::id`], `material_id`, `base_level_id`, ...): an element never
//! embeds a copy of another entity, so a value can never exist twice and disagree
//! with itself.
//!
//! Scope of the current step: element *data*. No meshes, no solids, no CAD
//! geometry, no reinforcement, no analysis results.

mod base;
mod beam;
mod boundary;
mod category;
mod column;
mod element;
mod foundation;
mod justification;
mod slab;
mod wall;

pub use base::BaseElement;
pub use beam::StructuralBeam;
pub use boundary::PlanBoundary;
pub use category::ElementCategory;
pub use column::StructuralColumn;
pub use element::Element;
pub use foundation::{Foundation, FoundationType};
pub use justification::Justification;
pub use slab::StructuralSlab;
pub use wall::StructuralWall;
