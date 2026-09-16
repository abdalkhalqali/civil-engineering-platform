use serde::{Deserialize, Serialize};

/// How an element is aligned vertically with respect to the level it references.
///
/// A beam whose section is 600 mm deep can sit with its top, its centre or its
/// bottom on the reference level; the same idea is reused by any future element
/// that is placed relative to a level.
///
/// Extension points that are deliberately *not* implemented yet: lateral
/// justification along the member axis, and per-end justification of inclined
/// members. They will arrive as additional fields or a second enum, without
/// changing this one's meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum Justification {
    /// The element's top face lies on the reference level.
    Top,
    /// The element's centre line lies on the reference level.
    #[default]
    Center,
    /// The element's bottom face lies on the reference level.
    Bottom,
}

impl Justification {
    pub const fn as_str(self) -> &'static str {
        match self {
            Justification::Top => "top",
            Justification::Center => "center",
            Justification::Bottom => "bottom",
        }
    }
}

impl std::fmt::Display for Justification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
