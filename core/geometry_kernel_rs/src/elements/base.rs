use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::ElementCategory;
use crate::math::Transform3D;

/// Properties shared by every element of the model.
///
/// Concrete elements embed this struct instead of repeating the fields, so
/// identity, naming and placement behave identically for every element type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseElement {
    /// Stable identity of the element for its whole lifetime.
    ///
    /// A vector index is *not* an identity: elements may be reordered, filtered or
    /// stored differently without changing this id.
    pub id: Uuid,
    /// What kind of entity this is.
    pub category: ElementCategory,
    /// Human readable name (for example `C001`), not required to be unique.
    pub name: String,
    /// Placement of the element in the global coordinate system.
    pub transform: Transform3D,
    /// Free-form annotations (discipline, phase, external references, ...).
    ///
    /// A [`BTreeMap`] is used instead of [`std::collections::HashMap`] so that
    /// serialization is byte-for-byte deterministic, which keeps diffs, content
    /// hashes and future history/events stable.
    pub metadata: BTreeMap<String, String>,
}

impl BaseElement {
    /// Creates an element base with a freshly generated identity.
    pub fn new(category: ElementCategory, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            name: name.into(),
            transform: Transform3D::IDENTITY,
            metadata: BTreeMap::new(),
        }
    }

    /// Creates an element base with an explicit identity, for example when
    /// rebuilding a model that already has ids (imports, tests, fixtures).
    pub fn with_id(id: Uuid, category: ElementCategory, name: impl Into<String>) -> Self {
        Self {
            id,
            category,
            name: name.into(),
            transform: Transform3D::IDENTITY,
            metadata: BTreeMap::new(),
        }
    }

    /// The stable identity of the element.
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Same base, placed with `transform`.
    pub fn with_transform(mut self, transform: Transform3D) -> Self {
        self.transform = transform;
        self
    }

    /// Same base, with one annotation added.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}
