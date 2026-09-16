# Engineering Model Core

The engineering model is the **single source of truth** of the platform. It is pure
Rust data — no database, no UI, no renderer — and every other system (geometry, 2D
views, the analytical model, quantities, BOQ, reports, BIM/IFC, AI commands) will be
*derived* from it.

Source of truth: [`core/geometry_kernel_rs/src`](../core/geometry_kernel_rs/src).

```text
EngineeringModel
│
├── levels          → Level
├── grids           → Grid
├── materials       → Material
├── cross_sections  → CrossSection
└── elements        → Element = Column | Beam | Slab | Wall | Foundation
```

---

## 1. Engineering Model

`model::EngineeringModel` holds the whole project. All collections are
[`IdMap<T>`](#storage) (`ID → entity`), so lookup never scans a vector.

```rust
use geometry_kernel_rs::model::{EngineeringModel, Level};
use geometry_kernel_rs::units::Length;

let mut model = EngineeringModel::new();
let level_1 = model.add_level(Level::new("Level 1", Length::from_meters(0.0)))?;
```

| Item | Meaning |
|---|---|
| `schema_version` | version of the serialized schema (currently `1`) |
| `project_id` | stable identity of the project |
| `revision` | number of accepted mutations, starts at `0` |
| `validation_mode` | editing policy, not model data ([validation](#18-validation)) |

`add_level` / `add_grid` / `add_material` / `add_cross_section` / `add_element` return
`Result<Uuid, ModelError>` and bump `revision`. `level(id)`, `grid(id)`,
`material(id)`, `cross_section(id)`, `element(id)` perform `O(log n)` lookups.

## 2. BaseElement

Every element embeds the same base struct instead of repeating fields:

```rust
pub struct BaseElement {
    pub id: Uuid,                          // stable identity, never an index
    pub category: ElementCategory,
    pub name: String,                      // for example "C001"
    pub transform: Transform3D,            // placement
    pub metadata: BTreeMap<String, String>,
}
```

`BaseElement::new(category, name)` generates a v4 UUID; `with_id(..)` is used when
rebuilding data that already has identities.

## 3. ElementCategory

```rust
pub enum ElementCategory {
    Column, Beam, Slab, Wall, Foundation,   // physical building elements
    Level, Grid,                            // reference / organisational elements
}
```

`is_physical()` and `is_reference()` let consumers stay correct when new variants
(`Stair`, `Opening`, `Roof`, `Pile`, `RetainingWall`, `Road`, `Terrain`,
`SurveyPoint`) are added later.

## 4. Point3D

A position in the global coordinate system. Components are `Length`s, so a
coordinate can never be an ambiguous bare number.

```rust
Point3D::from_meters(0.0, 0.0, 3.2)          // x, y, z in metres
Point3D::from_millimeters(0.0, 0.0, 3200.0)  // same point
point.distance_to(&other)                     // Length
point.offset_to(&other)                       // Vector3D
point.translated(offset)                      // Point3D
```

## 5. Vector3D

A displacement between two points: `Point3D - Point3D → Vector3D`.

```rust
let axis = start.offset_to(&end);   // Vector3D
axis.magnitude()                    // Length
axis.normalized()                   // Option<Vector3D> — None for a null vector
axis.scaled(2.0)
```

Products of two displacements (dot/cross) are *not* defined: their result is an area
or a normal, not a length, and this type refuses to pretend otherwise. They arrive
with the geometry kernel step.

## 6. Transform3D

```rust
pub struct Transform3D {
    pub translation: Point3D,   // position of the local origin
    pub rotation: Rotation3D,   // orientation
}

pub enum Rotation3D {
    Identity,
    AroundZ(Angle),             // plan rotation about the global +Z axis
}
```

`Transform3D` is **placement, not geometry**: sizes stay in the element's own
properties. There is deliberately **no scale factor** — a scaled column would report
a size that no property of the model states, i.e. a second source of truth.
`Rotation3D` is an enum so inclined members can be added later without breaking data.

## 7. Units

The kernel stores **SI values only**; conversion happens at the boundary:

```text
User input (400 mm, 30 MPa, 30°) → conversion → internal SI (0.4 m, 30e6 Pa, 0.5236 rad) → model
```

| Quantity | Internal unit | Type |
|---|---|---|
| Length | m | `Length` |
| Area | m² | `Area` |
| Second moment of area | m⁴ | `SecondMomentOfArea` |
| Force | N | `Force` |
| Mass | kg | `Mass` |
| Mass density | kg/m³ | `MassDensity` |
| Angle | rad | `Angle` |
| Stress / E modulus | Pa | `Stress` |

```rust
Length::from_millimeters(400.0).meters()   // 0.4
Stress::from_megapascals(30.0).pascals()   // 30_000_000.0
Angle::from_degrees(90.0).radians()        // 1.5707963267948966
```

A model value is therefore never `width = 400` with an unknown unit, and no element
carries a `unit: String` field: the *type* carries the unit. Adding a quantity is one
`define_unit!` invocation plus its conversions.

## 8. Level

```rust
let level_1 = Level::new("Level 1", Length::from_meters(0.0));
let level_2 = Level::new("Level 2", Length::from_meters(3.2));
```

`id`, `name`, `elevation` (metres, relative to the project datum `Z = 0`). The sign is
meaningful: a basement is a level with a negative elevation. A datum/reference-system
description is a documented future field; it is not part of the level yet.

## 9. Grid

Model data only — no drawing, no extents, no bubbles.

```rust
Grid::along_y("A", Length::from_meters(0.0));   // line parallel to Y at x = 0
Grid::along_x("1", Length::from_meters(0.0));   // line parallel to X at y = 0
```

`offset` is the **signed coordinate on the axis perpendicular to the line**:
the `Y` coordinate of an `AlongX` line, the `X` coordinate of an `AlongY` line. Names
are free text, so the usual letter lines (`A`, `B`, `C`) and numbered lines
(`1`, `2`, `3`) are both expressible.

## 10. Material

```rust
pub struct Material {
    pub id: Uuid,
    pub name: String,
    pub material_type: MaterialType,          // Concrete | Steel | Other
    pub density: MassDensity,                 // kg/m³
    pub compressive_strength: Stress,         // Pa
    pub youngs_modulus: Stress,               // Pa
}
```

```rust
Material::concrete_c30();   // 2500 kg/m³, fck = 30 MPa, E ≈ 33 GPa
Material::steel_s355();     // 7850 kg/m³, fy = 355 MPa, E = 200 GPa
```

Design codes, partial factors and constitutive laws are **not** here: they belong to a
later code/analysis layer and must not change this model's meaning.

## 11. CrossSection

```rust
pub enum ProfileType {
    Rectangular { width: Length, depth: Length },  // width along local x, depth along local y
    Circular { radius: Length },
    IBeam(IBeamProfile),                           // depth, flange_width, web_thickness, flange_thickness
}

pub struct CrossSection {
    pub id: Uuid,
    pub name: String,
    pub profile: ProfileType,
}
```

```rust
let section = CrossSection::rectangular("400x400", Length::from_millimeters(400.0), Length::from_millimeters(400.0));
section.area().square_meters();              // 0.16
section.moment_of_inertia();                 // (Ix, Iy) in m⁴
```

**No double source of truth**: `area` and `moment_of_inertia` are *functions of the
profile*, not fields. Storing them next to the profile would allow an edit of the shape
to leave a stale area behind. Change the profile and every derived quantity follows.

## 12. StructuralColumn

```rust
pub struct StructuralColumn {
    pub base: BaseElement,
    pub base_level_id: Uuid,
    pub top_level_id: Uuid,
    pub base_offset: Length,     // signed adjustment of the bottom end
    pub top_offset: Length,      // signed adjustment of the top end
    pub cross_section_id: Uuid,
    pub material_id: Uuid,
}
```

The extent is `[base_level.elevation + base_offset, top_level.elevation + top_offset]`
via `column.extent(base_elevation, top_elevation)`.

The plan rotation lives in `base.transform.rotation` (the single place placement is
stored) instead of a separate `rotation_angle` field that could contradict it:
`column.with_plan_rotation(Angle::from_degrees(30.0))`.

## 13. StructuralBeam

```rust
pub struct StructuralBeam {
    pub base: BaseElement,
    pub reference_level_id: Uuid,
    pub start_point: Point3D,
    pub end_point: Point3D,
    pub cross_section_id: Uuid,
    pub material_id: Uuid,
    pub z_justification: Justification,   // Top | Center | Bottom
}
```

* length and direction are **derived**: `beam.length()`, `beam.direction()`;
* `beam.is_degenerate()` flags a zero length axis (reported by validation);
* `with_justification(Justification::Top)` places the section relative to the level.
  Lateral justification and per-end justification of inclined members are future
  extensions of the same idea.

## 14. StructuralSlab

```rust
pub struct StructuralSlab {
    pub base: BaseElement,
    pub level_id: Uuid,
    pub thickness: Length,
    pub material_id: Uuid,
    pub boundary: PlanBoundary,
}
```

`PlanBoundary` is the temporary boundary abstraction: an ordered ring of `Point3D`
vertices in plan (implicitly closed, at least three, all at one height).

```rust
let outline = PlanBoundary::rectangle(Point3D::from_meters(0.0, 0.0, 3.2), Length::from_meters(6.0), Length::from_meters(5.0));
slab.boundary_area().square_meters();   // 30.0, derived by the shoelace formula
```

No mesh, no solid, no CAD loop: when the geometry kernel lands, this boundary can
become a reference to a geometry id without touching the slab's identity or its
relationships.

## 15. StructuralWall

```rust
pub struct StructuralWall {
    pub base: BaseElement,
    pub base_level_id: Uuid,
    pub top_level_id: Uuid,
    pub start_point: Point3D,     // centre plane axis
    pub end_point: Point3D,
    pub thickness: Length,
    pub material_id: Uuid,
}
```

`wall.length()` is derived. Openings, reinforcement and the analytical representation
are designed for but **not implemented**: openings will reference the wall by id (a
separate collection, or an added field), reinforcement is a later layer keyed by
element id, and the analytical model is derived separately.

## 16. Foundation

```rust
pub struct Foundation {
    pub base: BaseElement,
    pub level_id: Uuid,
    pub thickness: Length,
    pub material_id: Uuid,
    pub foundation_type: FoundationType,   // Isolated | Strip | Mat | PileCap
    pub footprint: PlanBoundary,
}
```

Bearing capacity, settlement, soil interaction and reinforcement design are explicitly
out of scope and will never become fields of this struct.

## 17. Element Relationships

Relationships are **ids**, never embedded copies:

```text
Column ── base_level_id ───────► Level
       ├─ top_level_id ────────► Level
       ├─ cross_section_id ────► CrossSection
       └─ material_id ─────────► Material
```

```rust
// Good: one material, referenced many times.
let material_id = model.add_material(Material::concrete_c30())?;

// Wrong (by design impossible): a column that embeds a full Material copy
// which can drift away from the shared definition.
```

Consequences: editing a level moves every column that references it; a material value
exists exactly once; `Element::referenced_ids()` lists an element's references so
validation does not need one hand-written check per element kind.

## 18. Validation

Two mechanisms:

1. **On mutation** — `add_element` rejects an element whose references do not resolve
   while the model is in `ValidationMode::Strict` (the default):

   ```rust
   model.add_element(element)?;   // Err(MissingReference { .. }) — nothing is stored
   ```

   `ValidationMode::Permissive` accepts such an element and lets
   `model.validate()` report it, which is what loading or repairing partial data needs.

2. **On demand** — `model.validate()` returns a `ValidationReport` without ever
   mutating or silently repairing the model:

   ```rust
   let report = model.validate();
   if !report.is_valid() {
       for issue in report.issues() {
           eprintln!("{}: {}", issue.code, issue.message);   // ValidationCode + message
       }
   }
   ```

Checked invariants: unique ids across the whole model (not just inside one
collection), entities stored under their own id, element category agreeing with its
concrete type, references resolving, positive dimensions (a zero thickness or a zero
length axis is reported), an upward extent for columns and walls, horizontal planar
boundaries, self-consistent I sections, non-negative material properties, and a
supported schema version.

Design validation (code compliance, capacity, detailing) is a different layer and is
out of scope.

## 19. Serialization

`serde` + `serde_json` (already part of the project):

```rust
let json = model.to_json()?;                        // Rust struct → JSON
let restored = EngineeringModel::from_json(&json)?; // JSON → Rust struct
```

Round trip is lossless (verified by `test_8_serialization_round_trip`), including
identities, offsets, internal units and references. Destructuring trusts its input, so
a hand-edited file can hold dangling references — call `validate()` after loading.

Collections are `BTreeMap`s, so the same model always serializes to the same bytes:
diffs and content hashes stay stable.

## 20. Schema Version

```rust
pub const MODEL_SCHEMA_VERSION: u32 = 1;
```

Every serialized model carries `schema_version`. A loader can therefore recognise
older data, and `validate()` reports `UnsupportedSchemaVersion` for data written by a
newer kernel. **Migrations are not implemented in this step** — only the marker that
makes them possible later.

## 21. Physical vs Analytical Model

* **Physical model** (this step): what is built — columns, beams, slabs, walls,
  foundations, their levels, grids, materials and cross sections, placed in the model
  coordinate system.
* **Analytical model** (later step): the idealised model used for structural analysis
  (members, nodes, supports, releases, loads). It will be *derived from* the physical
  model, and will reference physical element ids. It must never be edited as a second
  truth.

Because the physical model already uses stable ids, references and typed SI
quantities, the analytical layer can be added without changing any element type.

## Storage

`IdMap<T> = BTreeMap<Uuid, T>` is used for every collection:

* `O(log n)` lookup, never a linear scan;
* deterministic serialization (stable diffs, hashes and future event history);
* ids as keys, so storage order never changes identity;
* declared as an alias, so a specialised map can replace it in one line.

## Coordinate system

Right-handed Cartesian, documented in `math/mod.rs`:

```text
  Z  vertical (up), positive above the project datum
  │
  └────── X   first horizontal direction
 /
Y             second horizontal direction (perpendicular to X, in plan)
```

Plan rotation is about `+Z`. There is deliberately no geographic (GIS) system in this
step.

## Examples

Complete, runnable examples live in the test suite:

* [`tests/engineering_model.rs`](../core/geometry_kernel_rs/tests/engineering_model.rs) —
  the acceptance scenario (two levels, four grid lines, one material, one section, a
  column and a beam), create → validate → serialize → deserialize → validate;
* [`tests/kernel_api.rs`](../core/geometry_kernel_rs/tests/kernel_api.rs) — what the
  bridge exposes and the internal-unit rule.
