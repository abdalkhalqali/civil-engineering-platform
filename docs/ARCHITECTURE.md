# Architecture

Living document of the architectural decisions behind the platform. It is updated as
decisions are made; it does not repeat the detailed model documentation, which lives
in [`ENGINEERING_MODEL.md`](./ENGINEERING_MODEL.md).

## System shape

```text
Flutter app (apps/client_flutter)
        │  flutter_rust_bridge v2 (generated, never hand edited)
        ▼
Rust geometry kernel (core/geometry_kernel_rs)
        │
        ▼
Engineering Model  ← single source of truth
        │
        ├── 3D geometry        (later)
        ├── 2D views           (later)
        ├── analytical model   (later)
        ├── quantities / BOQ   (later)
        ├── reports            (later)
        ├── BIM / IFC export   (later)
        └── AI commands        (later)
```

Everything below the model is *derived*. Nothing outside the kernel owns engineering
data, and nothing outside the kernel may re-implement engineering rules.

## Kernel layers

```text
api/        Flutter boundary — flat, minimal, no model transfer yet
commands/   named mutations of the model (extension point)
model/      EngineeringModel and its entities (levels, grids, materials, sections)
elements/   building elements and the abstractions they share
math/       Point3D, Vector3D, Transform3D
units/      internal SI quantities
error/      invariant violations reported by mutations
```

Dependencies point downwards only: `api → commands → model → elements → math/units`.
The kernel has no dependency on Flutter, on a database or on a renderer.

## Decisions

### D1 — Engineering Model is the single source of truth (M1)

The model is plain Rust data held in memory. Geometry, drawings, analytical models,
quantities and exports are derived views. No UI, mesh, database or file format is
allowed to become an alternative source of engineering data.

*Consequence:* a feature is designed as "what does the model record, and what is
derived from it", never as "what does the screen need".

### D2 — Relationships are ids, never embedded copies

Elements reference levels, materials and cross sections by `Uuid`. Duplicating an
entity inside an element would allow two disagreeing values for one fact.

*Consequence:* `Element::referenced_ids()` is the single description of an element's
references; validation and (later) cascade updates use it instead of per-type code.

### D3 — Derived values are functions, not fields

`CrossSection::area()`, `moment_of_inertia()`, `StructuralBeam::length()` and
`StructuralSlab::boundary_area()` are computed from the data that defines them. This
directly answers the "no double source of truth" rule: a stored area could survive an
edit of the profile and lie about the shape.

### D4 — Internal SI units, typed

Every quantity is a newtype holding an SI value (`Length` in m, `Stress` in Pa, `Angle`
in rad, ...), and conversion happens at the boundary (`from_millimeters`,
`from_megapascals`, `from_degrees`). No element stores a `unit: String`, and no value
is a bare number whose unit must be guessed. `400 mm` is stored as `0.4 m`.

### D5 — Stable identity

Entities carry a `Uuid` generated at creation. A vector index is never an identity:
collections can be reordered, filtered or re-stored without changing identity or
breaking references.

### D6 — `IdMap<T> = BTreeMap<Uuid, T>` for storage

Chosen over `HashMap` for deterministic serialization (stable diffs, content hashes,
future event history) while keeping `O(log n)` lookup; declared as an alias so a
specialised structure can replace it in one line. See
[`storage.rs`](../core/geometry_kernel_rs/src/model/storage.rs).

### D7 — `Transform3D` carries placement only, and no scale

Position plus an extensible `Rotation3D` enum (today: identity or plan rotation about
`+Z`). Scale is excluded by design: a scale factor would report a size that no property
of the model states, i.e. a second source of truth for dimensions.

### D8 — Composition and enums, not inheritance

Concrete elements embed `BaseElement`; the model stores them in a closed `Element`
enum. No `Box<dyn Any>`, no downcasting, no trait-object layer: exhaustive matching
keeps every kind handled, and adding a variant later does not disturb existing types.

### D9 — Two-level validation

`ValidationMode::Strict` (default) rejects mutations that would leave a dangling
reference. `ValidationMode::Permissive` accepts them and lets `validate()` report them,
which is what loading/repairing partial data needs. `validate()` never mutates or
silently repairs the model; it returns a report of `ValidationCode` + message.

### D10 — Commands are an extension point, not a system

`commands::ModelCommand` fixes the shape (named, serializable, applies through the
model's `add_*` methods, returns the affected id) so that
`Command → Event → History` can be added later without redesigning the model.
Undo/redo, event sourcing and a command bus are deliberately not implemented.

### D11 — Schema version now, migration later

`MODEL_SCHEMA_VERSION = 1` is stored in every serialized model, and validation reports
data written by a newer kernel. No migration framework, no `.civilx` container, no
database: those are separate, dedicated steps.

### D12 — What crosses the Rust/Flutter boundary

Only `get_kernel_status()` (the M0 proof of concept, kept working),
`create_empty_model() -> ModelSummary`, and `create_project() -> ProjectSummary`,
flat snapshots of counts. The model itself, its entities and every engineering
rule stay in Rust; Flutter receives no model data it could re-interpret.

*Bridge rule:* `flutter_rust_bridge` addresses mirrored types at their **defining
path**, so a module holding such a type must be reachable (`pub mod summary;` in
`model/mod.rs`, `pub mod project` in `lib.rs`). Generated files
(`src/frb_generated.rs`, `lib/ffi_bridge/generated/**`) are produced only by
`flutter_rust_bridge_codegen generate` — never hand edited.

### D13 — Rust crate types

`crate-type = ["cdylib", "staticlib", "rlib"]`. The `cdylib`/`staticlib` artifacts are
what Flutter (cargokit) links against; `rlib` is what lets `cargo test` build the
integration test suite in `tests/` against the public API.

### D14 — Project persistence as a trait, not a format

`ProjectSerializer` abstracts the on-disk format. The concrete `ProjectFormatV1`
uses newline-delimited JSON (header \n metadata \n model). A future format
(binary, compressed, encrypted, database-backed) implements the same trait
without touching the domain model.

### D15 — Format version and schema version are independent

`PROJECT_FORMAT_VERSION` (file container) and `MODEL_SCHEMA_VERSION` (domain
fields) evolve independently. A format v3 file may carry a schema v1 model.
Both are stored in the file header and checked at load time.

### D16 — Metadata is not domain state

`Project.name`, `description`, `created_at`, `modified_at` survive a round-trip
but never affect geometry, calculations, validation or revision. Timestamps are
set once at creation; `touch()` updates `modified_at`.

### D17 — Single project identity

`Project` has no separate `project_id` field. The identity is always
`EngineeringModel::project_id`. This eliminates the risk of two disagreeing
UUIDs for the same project.

## Geometry kernel evaluation

A comprehensive evaluation of geometry kernel candidates (OpenCascade/cadrum,
Truck/monstertruck, custom kernel) is documented in
[`GEOMETRY_KERNEL_EVALUATION.md`](./GEOMETRY_KERNEL_EVALUATION.md). The key
architectural decision is a **Kernel Adapter trait** that isolates the
engineering model from any concrete kernel, allowing the kernel to be swapped
without rewriting model code.

## Explicitly out of scope in this step

3D rendering, Three.js/WebGPU, CAD geometry, meshes, solids, booleans, BIM
UI, structural analysis and solvers, AI, backend, cloud, database servers, SQLite,
PostgreSQL, migrations, lazy loading, and any engineering logic in Flutter. They
arrive as separate steps on top of this model.

## Verification

```bash
cd core/geometry_kernel_rs && cargo check && cargo test
cd ../../apps/client_flutter && flutter pub get && flutter analyze && flutter test
```

`flutter test` needs a native library: a bundled app build provides it through cargokit,
and a desktop host provides it with
`FRB_DART_LOAD_EXTERNAL_LIBRARY_NATIVE_LIB_DIR=<crate>/target/debug` after
`cargo build`.
