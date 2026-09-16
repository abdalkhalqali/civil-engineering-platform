use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::{
    BaseElement, ElementCategory, Foundation, StructuralBeam, StructuralColumn, StructuralSlab,
    StructuralWall,
};
use crate::error::ReferenceKind;

/// Any physical element stored in the engineering model.
///
/// The variant carries the concrete element and the enum is *closed over known
/// kinds*, so storage is type-safe (`match` is exhaustive), serializable and
/// cheap — no `Box<dyn Any>`, no downcasting, no dynamic dispatch on the hot path.
/// Adding `Stair`, `Roof`, `Pile`, ... later means adding one variant plus its
/// struct; every existing element type keeps working unchanged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Column(StructuralColumn),
    Beam(StructuralBeam),
    Slab(StructuralSlab),
    Wall(StructuralWall),
    Foundation(Foundation),
}

impl Element {
    /// The properties every element shares.
    pub fn base(&self) -> &BaseElement {
        match self {
            Element::Column(element) => &element.base,
            Element::Beam(element) => &element.base,
            Element::Slab(element) => &element.base,
            Element::Wall(element) => &element.base,
            Element::Foundation(element) => &element.base,
        }
    }

    /// Mutable access to the shared properties, which is how coming commands
    /// (`MoveElementCommand`, `ChangePropertyCommand`, ...) will edit placement and
    /// naming while the concrete variant stays untouched.
    pub fn base_mut(&mut self) -> &mut BaseElement {
        match self {
            Element::Column(element) => &mut element.base,
            Element::Beam(element) => &mut element.base,
            Element::Slab(element) => &mut element.base,
            Element::Wall(element) => &mut element.base,
            Element::Foundation(element) => &mut element.base,
        }
    }

    /// Stable identity of the element.
    pub fn id(&self) -> Uuid {
        self.base().id
    }

    /// Human readable name of the element.
    pub fn name(&self) -> &str {
        &self.base().name
    }

    /// Category of the element, which must agree with the variant.
    pub fn category(&self) -> ElementCategory {
        self.base().category
    }

    /// Category implied by the variant itself, used to detect inconsistent data.
    pub const fn category_of_variant(&self) -> ElementCategory {
        match self {
            Element::Column(_) => ElementCategory::Column,
            Element::Beam(_) => ElementCategory::Beam,
            Element::Slab(_) => ElementCategory::Slab,
            Element::Wall(_) => ElementCategory::Wall,
            Element::Foundation(_) => ElementCategory::Foundation,
        }
    }

    /// Every entity this element points at, as `(kind, id)` pairs.
    ///
    /// Validation walks this list instead of hand-coding one check per element
    /// type, so a new element type only has to describe its references once.
    pub fn referenced_ids(&self) -> Vec<(ReferenceKind, Uuid)> {
        match self {
            Element::Column(column) => vec![
                (ReferenceKind::Level, column.base_level_id),
                (ReferenceKind::Level, column.top_level_id),
                (ReferenceKind::CrossSection, column.cross_section_id),
                (ReferenceKind::Material, column.material_id),
            ],
            Element::Beam(beam) => vec![
                (ReferenceKind::Level, beam.reference_level_id),
                (ReferenceKind::CrossSection, beam.cross_section_id),
                (ReferenceKind::Material, beam.material_id),
            ],
            Element::Slab(slab) => vec![
                (ReferenceKind::Level, slab.level_id),
                (ReferenceKind::Material, slab.material_id),
            ],
            Element::Wall(wall) => vec![
                (ReferenceKind::Level, wall.base_level_id),
                (ReferenceKind::Level, wall.top_level_id),
                (ReferenceKind::Material, wall.material_id),
            ],
            Element::Foundation(foundation) => vec![
                (ReferenceKind::Level, foundation.level_id),
                (ReferenceKind::Material, foundation.material_id),
            ],
        }
    }

    /// Borrows the element as a column when it is one.
    pub fn as_column(&self) -> Option<&StructuralColumn> {
        match self {
            Element::Column(element) => Some(element),
            _ => None,
        }
    }

    /// Borrows the element as a beam when it is one.
    pub fn as_beam(&self) -> Option<&StructuralBeam> {
        match self {
            Element::Beam(element) => Some(element),
            _ => None,
        }
    }

    /// Borrows the element as a slab when it is one.
    pub fn as_slab(&self) -> Option<&StructuralSlab> {
        match self {
            Element::Slab(element) => Some(element),
            _ => None,
        }
    }

    /// Borrows the element as a wall when it is one.
    pub fn as_wall(&self) -> Option<&StructuralWall> {
        match self {
            Element::Wall(element) => Some(element),
            _ => None,
        }
    }

    /// Borrows the element as a foundation when it is one.
    pub fn as_foundation(&self) -> Option<&Foundation> {
        match self {
            Element::Foundation(element) => Some(element),
            _ => None,
        }
    }
}

macro_rules! element_from {
    ($element:ty, $variant:ident) => {
        impl From<$element> for Element {
            fn from(element: $element) -> Self {
                Element::$variant(element)
            }
        }
    };
}

element_from!(StructuralColumn, Column);
element_from!(StructuralBeam, Beam);
element_from!(StructuralSlab, Slab);
element_from!(StructuralWall, Wall);
element_from!(Foundation, Foundation);
