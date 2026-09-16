//! Storage primitive for id-keyed model collections.

use std::collections::BTreeMap;

use uuid::Uuid;

/// An id-keyed collection of model entities: `ID → entity`.
///
/// # Why a `BTreeMap` and not a `HashMap`
///
/// * **Fast lookup** — `O(log n)`, never a linear scan; the model never searches a
///   vector for an element.
/// * **Deterministic serialization** — iteration order is stable for a given set of
///   ids, so the same model always produces the same JSON. That is what makes
///   content hashes, diffs and the future command/event history reproducible.
/// * **Stable ids** — keys are UUIDs, so reordering, filtering or re-storing the
///   collection never changes any entity's identity.
/// * **Swappable** — every collection in the model is declared as `IdMap<T>`, so a
///   future specialised structure (a persistent/immutable map, an indexed arena,
///   ...) can be introduced in this single line without touching element or model
///   code.
///
/// The trait surface used by the rest of the kernel (`get`, `insert`,
/// `contains_key`, `values`, `len`) is shared by both `BTreeMap` and `HashMap`.
pub type IdMap<T> = BTreeMap<Uuid, T>;
