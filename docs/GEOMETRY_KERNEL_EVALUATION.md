# Geometry Kernel Evaluation — STEP 4A

Architectural study and evaluation of geometry kernel candidates for the Civil
Engineering Platform. This document does **not** commit to any kernel — it
establishes the requirements, evaluates the options, and recommends a path
forward for the user's final decision.

---

## 1. Current Architecture

```text
Engineering Model (Rust)
├── units/       SI quantities (Length, Stress, Angle, ...)
├── math/        Point3D, Vector3D, Transform3D
├── elements/    Column, Beam, Slab, Wall, Foundation
├── model/       EngineeringModel, Level, Grid, Material, CrossSection
├── project/     .civilx persistence (ProjectSerializer trait)
├── commands/    ModelCommand trait (extension point)
└── api/         Flutter boundary (flutter_rust_bridge)
```

Every element is defined by **parametric engineering data**:
- Column: base_level, top_level, cross_section, material, offsets
- Beam: start_point, end_point, cross_section, material, justification
- Slab: level, thickness, material, boundary (ordered ring of points)
- Wall: base_level, top_level, start_point, end_point, thickness, material
- Foundation: level, thickness, material, type, footprint (boundary)

**PlanBoundary** is a temporary abstraction: an ordered ring of `Point3D`
vertices used by Slab and Foundation. It is explicitly documented as "not a
mesh, not a surface and not a CAD loop." When the geometry kernel lands, a
boundary can be replaced by a reference to a geometry ID.

**What does not exist yet:**
- No solids, no surfaces, no curves, no B-Rep
- No boolean operations, no fillets, no chamfers
- No tessellation, no mesh generation
- No 3D viewport, no rendering
- No geometry caching, no spatial indexing

---

## 2. Geometry Requirements

### 2.1 Core Requirements

| Requirement | Priority | Notes |
|---|---|---|
| B-Rep solid modeling | **Must** | Primary representation for structural elements |
| Parametric solid generation | **Must** | Extrude section along path for columns, beams, walls |
| Boolean operations | **Must** | Union for slab+beam intersections, openings in walls |
| Tessellation | **Must** | Convert solids to triangle meshes for rendering |
| NURBS curves/surfaces | **Should** | For complex profiles, curved walls, terrain |
| STEP import/export | **Should** | Interoperability with Revit, Tekla, other tools |
| BRep format | **Nice** | OpenCascade native format |
| Fillet/chamfer | **Should** | Realistic structural detailing |
| Surface area queries | **Must** | For cladding, painting quantities |
| Volume queries | **Must** | For concrete volumes, quantities |
| Inertia queries | **Should** | For mass properties, structural analysis |
| Section cutting | **Should** | For 2D views, quantity takeoff by level |
| Spatial indexing | **Should** | AABB tree for large models |
| Incremental rebuild | **Should** | Only rebuild changed elements |
| Precision control | **Must** | Engineering tolerance (mm-level) |

### 2.2 Platform Requirements

| Platform | Requirement |
|---|---|
| Linux (development) | Must work, native build |
| Android (Flutter) | Must work, cross-compiled via NDK |
| iOS (Flutter) | Must work, cross-compiled via Xcode |
| Web/WASM | Strongly desired, not blocking |
| macOS (development) | Must work, native build |
| Windows | Should work, native build |

### 2.3 Non-Requirements (for now)

- GPU rendering (separate step)
- Mesh editing (not CAD)
- Drawing generation (separate step)
- IFC export (separate step, uses geometry but is not geometry)
- Structural analysis (uses geometry, is not geometry)

---

## 3. Kernel Candidates

### A. OpenCascade (via `cadrum` or `opencascade-rs`)

**What it is:** Open CASCADE Technology (OCCT) is an industrial-grade,
open-source B-Rep geometry kernel originally developed for aerospace and
automotive CAD. It is the kernel behind FreeCAD, KiCAD (3D), and many
commercial tools.

**Rust bindings available:**

| Crate | Approach | WASM | Status |
|---|---|---|---|
| `cadrum` | Statically linked OCCT, high-level Rust API | ✅ (Docker build) | Active, 40 releases, v0.8.20 |
| `opencascade-rs` | cxx.rs FFI to OCCT submodule | Partial (WIP) | Hobby project, slow updates |
| `occt-sys` | Raw FFI bindings, static OCCT 7.8 | Possible | Active, 141k downloads |
| `occt-wasm` | Pre-built OCCT WASM module | ✅ (native) | New, 6 releases |

**Strengths:**
- Industrial-grade B-Rep: proven in production CAD systems for 25+ years
- Complete solid modeling: extrude, revolve, loft, sweep, shell, fillet, chamfer
- Boolean operations: robust union, cut, common
- NURBS: full curve/surface support
- Tessellation: built-in mesh generation with quality control
- STEP/BRep I/O: native format support
- Precision: configurable modeling tolerance
- Large model support: proven with models of 100k+ faces
- IFC compatibility: IFC uses B-Rep, OCCT is the standard kernel for IFC

**Weaknesses:**
- Build complexity: C++17 compiler, CMake, OCCT source (~30 min first build)
- Binary size: OCCT static library is 50-150 MB depending on features
- WASM: requires Docker-based cross-compilation, large WASM binary
- Memory: OCCT uses significant memory for complex models
- License: LGPL 2.1 with exception (see §4)
- Learning curve: OCCT API is C++-oriented, not idiomatic Rust

**`cadrum` specifics:**
- Most mature high-level Rust wrapper
- Statically linked, no system dependency
- WASM support via Docker pre-built OCCT
- STEP/BRep/STL/glTF I/O
- Active development (40 releases in 6 months)
- Idiomatic Rust API (Result-returning, builder pattern)

**`opencascade-rs` specifics:**
- High-level, ergonomic Rust API
- WGPU-based viewer (experimental)
- Slow development pace (hobby project)
- Less mature than cadrum

### B. Truck (via `truck-modeling` + `truck-geometry`)

**What it is:** Truck is a pure-Rust CAD kernel built from scratch, implementing
B-Rep with NURBS. It targets next-generation CAD using Rust and WebGPU.

**Rust crates:**

| Crate | Purpose | Downloads |
|---|---|---|
| `truck-geometry` | Knot vectors, B-splines, NURBS | 96k |
| `truck-topology` | Vertex, edge, wire, face, shell, solid | (part of truck-modeling) |
| `truck-modeling` | Integrated modeling algorithms | 94k |
| `truck-shapeops` | Boolean operations | (part of workspace) |
| `truck-polymesh` | Mesh data structures | (part of workspace) |
| `truck-platform` | WGPU graphics | (part of workspace) |

**Also:** `monstertruck` — an active fork with improved ergonomics, Rust 2024
edition, wgpu 29, and renamed APIs.

**Strengths:**
- Pure Rust: no C++ dependency, clean Cargo build
- NURBS-native: B-spline and NURBS from the ground up
- Modular: individual crates can be used independently
- WASM: targets wasm32-unknown-unknown
- License: Apache 2.0 (permissive)
- Active fork (monstertruck) with modern Rust practices
- Boolean operations available
- Tessellation algorithms included
- No OCCT build complexity

**Weaknesses:**
- Smaller community than OpenCascade
- Less battle-tested in production CAD
- Boolean operations less robust than OCCT (common weakness of custom kernels)
- API maturity: still evolving (breaking changes between versions)
- Limited I/O: no native STEP support (STEP requires additional libraries)
- Performance: not as optimized as OCCT for very large models
- NURBS precision: custom implementation, less validated than OCCT
- Missing features: no shell, no offset, limited fillet support
- Memory management: less optimized than OCCT

### C. Custom Geometry Kernel (from scratch)

**What it is:** Building our own geometry kernel in Rust, implementing only
what we need for civil engineering.

**Approach:**
- Start with parametric solids (extrude, revolve)
- Add boolean operations incrementally
- Implement only the NURBS we actually need
- Build tessellation for our specific element types

**Strengths:**
- Total control: exact API we want
- Minimal dependencies: only what we build
- Perfect Rust integration: idiomatic from day one
- Optimized for our use case: civil engineering elements
- No license concerns
- Smallest binary size
- Fastest compile times

**Weaknesses:**
- Enormous effort: years of work to reach OCCT maturity
- Boolean operations are extremely hard to implement robustly
- NURBS mathematics is complex and error-prone
- Tessellation quality requires significant expertise
- No STEP support without implementing it ourselves
- No community, no prior art, no bug fixes from others
- Risk of paint洞: we'd spend all our time on geometry, not the product
- Precision issues: numerical robustness in CAD is a deep problem

### D. `parry3d` (Collision Geometry)

**What it is:** A Rust crate for 3D collision detection, providing convex
hulls, AABBs, BVH, and basic shapes.

**Strengths:**
- Pure Rust, well-maintained (part of Rapier physics engine)
- Good spatial indexing (AABB tree, BVH)
- Convex hulls, basic shapes
- WASM compatible

**Weaknesses:**
- Not a geometry kernel: no B-Rep, no NURBS, no boolean ops
- Collision detection, not CAD modeling
- Cannot generate solids from sections
- Cannot do STEP I/O
- Not suitable as primary geometry kernel

### E. `nalgebra` + Custom Math

**What it is:** Using nalgebra for linear algebra and building geometry on top.

**Strengths:**
- Excellent linear algebra foundation
- Pure Rust, well-maintained
- Good WASM support

**Weaknesses:**
- Not a geometry kernel: matrices and vectors only
- Would still need to build B-Rep, NURBS, booleans from scratch
- Same weaknesses as Option C, but with better math primitives

---

## 4. Licensing Analysis

| Kernel | License | Commercial Use | WASM Distribution | Flutter/Apk |
|---|---|---|---|---|
| OpenCascade (OCCT) | LGPL 2.1 + linking exception | ✅ Allowed | ✅ Static linking OK | ✅ Dynamic linking OK |
| `cadrum` | Depends on OCCT build | ✅ Same as OCCT | ✅ Static WASM | ✅ |
| `opencascade-rs` | LGPL 2.1 + exception | ✅ Same as OCCT | ✅ | ✅ |
| Truck | Apache 2.0 | ✅ | ✅ | ✅ |
| monstertruck | Apache 2.0 | ✅ | ✅ | ✅ |
| Custom | N/A (our code) | ✅ | ✅ | ✅ |

**LGPL 2.1 with linking exception** means:
- We can dynamically link our application to OCCT
- We can distribute OCCT as a shared library alongside our app
- We do NOT need to release our source code
- The "linking exception" explicitly allows this use case
- This is the same license used by Qt, FLTK, and many commercial tools

**Apache 2.0** (Truck/monstertruck) means:
- We can use, modify, and distribute freely
- We must include the license and any NOTICE files
- We can distribute commercially without source release
- Patent grant included

---

## 5. Platform Analysis

### 5.1 Linux (Development)

| Kernel | Build | Notes |
|---|---|---|
| cadrum | `cargo build` (prebuilt OCCT) | Works out of the box |
| opencascade-rs | `cargo build` (CMake + C++) | Requires C++ toolchain |
| Truck | `cargo build` | Pure Rust, no issues |
| Custom | `cargo build` | No issues |

### 5.2 Android

| Kernel | Cross-compile | Notes |
|---|---|---|
| cadrum | ✅ via NDK | Prebuilt OCCT for aarch64-linux-android |
| opencascade-rs | Possible | Requires NDK cross-compilation setup |
| Truck | ✅ | Pure Rust, cross-compiles easily |
| Custom | ✅ | Pure Rust |

### 5.3 iOS

| Kernel | Cross-compile | Notes |
|---|---|---|
| cadrum | ✅ via Xcode | Prebuilt OCCT for aarch64-apple-ios |
| opencascade-rs | Possible | Requires Xcode toolchain |
| Truck | ✅ | Pure Rust, cross-compiles easily |
| Custom | ✅ | Pure Rust |

### 5.4 Web/WASM

| Kernel | WASM | Binary Size | Performance |
|---|---|---|---|
| cadrum | ✅ (Docker build) | ~15-30 MB | Good (headless) |
| opencascade-rs | Partial (WIP) | Unknown | Unknown |
| Truck | ✅ | ~2-5 MB | Good |
| Custom | ✅ | Small | Depends on impl |

**WASM is strongly desired but not blocking.** The rendering will happen on
the server or via WebGL, and geometry computation can happen on the server
side initially. WASM support can be added later when the kernel is stable.

---

## 6. Comparison Matrix

| Criterion | cadrum (OCCT) | Truck | Custom | Notes |
|---|---|---|---|---|
| **B-Rep** | ✅ Industrial | ✅ Pure Rust | ❌ Years of work | OCCT is the gold standard |
| **NURBS** | ✅ Full | ✅ Full | ❌ Complex math | Both handle curves/surfaces |
| **Booleans** | ✅ Robust | ⚠️ Improving | ❌ Very hard | Booleans are the hardest part |
| **Fillet/Chamfer** | ✅ Full | ⚠️ Limited | ❌ Complex | OCCT has decades of edge cases |
| **Tessellation** | ✅ Built-in | ✅ Included | ⚠️ Need to build | Quality varies |
| **STEP I/O** | ✅ Native | ❌ Not built-in | ❌ Must implement | Critical for BIM |
| **BRep I/O** | ✅ Native | ⚠️ Custom format | ❌ Must implement | OCCT BRep is standard |
| **Volume/Area** | ✅ Exact | ✅ Available | ⚠️ Need to implement | Derived from B-Rep |
| **Precision** | ✅ Configurable | ⚠️ Float-based | ⚠️ Unknown | OCCT uses 1e-7 tolerance |
| **Large models** | ✅ Proven | ⚠️ Less tested | ❌ Unknown | OCCT handles 100k+ faces |
| **Build complexity** | ⚠️ C++ required | ✅ Pure Rust | ✅ Pure Rust | OCCT needs CMake |
| **Binary size** | ⚠️ 50-150 MB | ✅ 2-5 MB | ✅ Small | OCCT static lib is large |
| **WASM** | ✅ (Docker) | ✅ | ✅ | All support WASM |
| **License** | LGPL 2.1+ex | Apache 2.0 | N/A | Both permissive enough |
| **Community** | ✅ Large (OCCT) | ⚠️ Small | ❌ None | OCCT has 25+ years |
| **Maintenance** | ✅ Active | ⚠️ Fork needed | ❌ All on us | Truck needed a fork |
| **Flutter integration** | ✅ Via FFI | ✅ Via FFI | ✅ Via FFI | All work with FRB |
| **Civil engineering** | ✅ Standard | ⚠️ General CAD | ⚠️ Custom | OCCT is used in Revit, Tekla |
| **IFC compatibility** | ✅ Native (IFC uses OCCT) | ❌ Would need conversion | ❌ Must implement | Industry standard |
| **Parametric modeling** | ✅ Via code | ✅ Via code | ✅ Via code | All support parametric |

---

## 7. CAD/BIM Considerations

### 7.1 Industry Standard

The BIM industry uses B-Rep representation for structural elements. IFC (the
open BIM standard) defines geometry using B-Rep and CSG (Constructive Solid
Geometry). OpenCascade is the de facto kernel for IFC processing:

- FreeCAD uses OCCT for IFC import/export
- IfcOpenShell uses OCCT internally
- Revit exports to IFC using B-Rep representation
- Tekla Structures uses B-Rep for structural modeling

### 7.2 Our Elements → B-Rep Mapping

| Element | B-Rep Generation |
|---|---|
| Column | Extrude cross-section along vertical axis |
| Beam | Sweep cross-section along start→end axis |
| Slab | Extrude boundary polygon along thickness |
| Wall | Extrude wall polygon along thickness × height |
| Foundation | Extrude footprint polygon along thickness |

All of these are **extrude/sweep operations** — the most basic B-Rep
operations that every kernel supports.

### 7.3 Boolean Requirements

| Operation | Use Case |
|---|---|
| Union | Combine overlapping elements (slab + beam junction) |
| Cut | Create openings in walls (doors, windows) |
| Common | Intersect elements for quantity takeoff |

Booleans are the hardest geometric operation. OCCT has the most robust
implementation; custom kernels often fail on edge cases.

---

## 8. Civil Engineering Considerations

### 8.1 What We Actually Need

Civil engineering geometry is simpler than mechanical CAD:
- Most elements are **extrusions** of 2D profiles
- **Boolean operations** are needed but not complex (openings, junctions)
- **Curved elements** are rare (curved walls, arches) but exist
- **Terrain** is a special case (mesh-based, not B-Rep)
- **Reinforcement** is 1D (bars) with 2D coverage (not solid modeling)

### 8.2 Volume of Geometry Operations

| Operation | Frequency | Complexity |
|---|---|---|
| Create column solid | Every column | Low (extrude) |
| Create beam solid | Every beam | Low (sweep) |
| Create slab solid | Every slab | Low (extrude polygon) |
| Create wall solid | Every wall | Low (extrude polygon) |
| Boolean: opening in wall | Every door/window | Medium |
| Boolean: beam-slab junction | Every intersection | Medium |
| Tessellate for rendering | Every visible element | Low |
| Volume/area query | Every element | Low |
| Section cut | On demand | Medium |

### 8.3 Performance Profile

A typical civil engineering project:
- 500-5000 columns
- 1000-10000 beams
- 200-2000 slabs
- 500-5000 walls
- 100-1000 foundations
- Total: 2000-28000 elements

Each element generates one solid. Boolean operations are needed for maybe
10-20% of elements (openings, junctions). This is well within the capability
of any kernel.

---

## 9. Parametric Geometry Architecture

### 9.1 Proposed Layer Architecture

```text
Engineering Model (source of truth)
    │
    ▼
Parametric Geometry Service (new layer)
    │  - Takes element data + model context
    │  - Generates parametric solid descriptors
    │  - Caches results by element ID
    │  - Invalidates on element change
    │
    ▼
Geometry Kernel Adapter (trait)
    │  - Abstract interface to the selected kernel
    │  - swap_kernel() changes implementation
    │  - No model code depends on kernel directly
    │
    ▼
Concrete Kernel Implementation
    │  - cadrum (OpenCascade)
    │  - Truck
    │  - Custom
    │  - (future alternatives)
    │
    ▼
Geometric Representation (B-Rep solids)
    │  - id, type, parameters
    │  - topology (faces, edges, vertices)
    │  - derived properties (volume, area, inertia)
    │
    ▼
Tessellation Service
    │  - Converts B-Rep to triangle mesh
    │  - Quality parameters (chord deviation, angle)
    │  - Caches meshes by solid ID + quality level
    │
    ▼
Rendering (future step)
    - GPU rendering
    - LOD management
    - Frustum culling
```

### 9.2 Kernel Adapter Trait (Conceptual)

```rust
/// Abstract interface to a geometry kernel.
/// No model code should depend on a concrete kernel.
pub trait GeometryKernel {
    type Solid: GeometrySolid;
    type Error: std::error::Error;

    /// Extrude a 2D profile along a direction.
    fn extrude(
        &self,
        profile: &[Point3D],
        direction: Vector3D,
    ) -> Result<Self::Solid, Self::Error>;

    /// Sweep a profile along an axis.
    fn sweep(
        &self,
        profile: &[Point3D],
        axis_start: Point3D,
        axis_end: Point3D,
    ) -> Result<Self::Solid, Self::Error>;

    /// Boolean union of two solids.
    fn union(
        &self,
        a: &Self::Solid,
        b: &Self::Solid,
    ) -> Result<Self::Solid, Self::Error>;

    /// Boolean cut (a minus b).
    fn cut(
        &self,
        a: &Self::Solid,
        tool: &Self::Solid,
    ) -> Result<Self::Solid, Self::Error>;

    /// Tessellate a solid to a triangle mesh.
    fn tessellate(
        &self,
        solid: &Self::Solid,
        tolerance: f64,
    ) -> Result<TriangleMesh, Self::Error>;

    /// Compute volume of a solid.
    fn volume(&self, solid: &Self::Solid) -> Result<f64, Self::Error>;

    /// Compute surface area of a solid.
    fn area(&self, solid: &Self::Solid) -> Result<f64, Self::Error>;
}
```

### 9.3 Element → Parametric Solid Mapping

Each element type maps to a specific parametric generation:

```rust
/// Generate a solid for a structural column.
fn column_solid(
    kernel: &impl GeometryKernel,
    column: &StructuralColumn,
    model: &EngineeringModel,
) -> Result<impl GeometrySolid, GeometryError> {
    let section = model.cross_section(column.cross_section_id)?;
    let base_level = model.level(column.base_level_id)?;
    let top_level = model.level(column.top_level_id)?;

    let base_z = base_level.elevation + column.base_offset;
    let top_z = top_level.elevation + column.top_offset;
    let height = top_z - base_z;

    let profile = section.profile.to_polygon(24); // 24-sided approximation
    let direction = Vector3D::new(Length::ZERO, Length::ZERO, height);

    kernel.extrude(&profile, direction)
}
```

### 9.4 Geometry Caching Strategy

```text
Element ID → Geometry Cache Entry
    │
    ├── solid: Option<Solid>        (B-Rep, derived from params)
    ├── mesh: Option<TriangleMesh>  (tessellated, derived from solid)
    ├── bbox: Option<AABB>          (bounding box, derived from solid)
    ├── version: u64                (element revision when cached)
    └── quality: TessellationQuality (chord deviation, angle)

Invalidation:
    - Element changed → clear solid + mesh + bbox
    - CrossSection changed → clear all elements using it
    - Level changed → clear all elements referencing it
    - Material changed → NO invalidation (material ≠ geometry)
```

---

## 10. Geometry/Engineering Model Boundary

### 10.1 What Stays in Engineering Model

| Data | Location | Reason |
|---|---|---|
| Element parameters | EngineeringModel | Source of truth |
| Level elevations | EngineeringModel | Source of truth |
| Cross section profiles | EngineeringModel | Source of truth |
| Material references | EngineeringModel | Source of truth |
| Element relationships | EngineeringModel | Source of truth |
| Validation rules | EngineeringModel | Domain logic |
| Serialization | EngineeringModel | Persistence |

### 10.2 What Goes in Geometry Layer

| Data | Location | Reason |
|---|---|---|
| B-Rep solids | GeometryCache | Derived from params |
| Triangle meshes | GeometryCache | Derived from solids |
| Bounding boxes | GeometryCache | Derived from solids |
| Spatial index | GeometryIndex | Derived from bboxes |
| Tessellation params | GeometryConfig | Rendering settings |

### 10.3 The Boundary Rule

**Geometry is derived from engineering data, never the other way around.**

Modifying a column's section regenerates its solid. Modifying a solid does
NOT change the column's section. The geometry layer is a cache, not a source
of truth.

---

## 11. Persistence Boundary

### 11.1 What is Persisted (in .civilx)

| Data | Persisted | Reason |
|---|---|---|
| Engineering model | ✅ | Domain state |
| Element parameters | ✅ | Domain state |
| Geometry solids | ❌ | Derived, regenerable |
| Triangle meshes | ❌ | Derived, regenerable |
| Bounding boxes | ❌ | Derived, regenerable |
| Spatial index | ❌ | Derived, regenerable |

### 11.2 Why Geometry is Not Persisted

1. **Regenerable**: geometry can always be recomputed from params
2. **Kernel-dependent**: solid format changes if kernel changes
3. **Size**: triangle meshes are large, params are small
4. **Correctness**: persisted geometry could diverge from params
5. **Migration**: changing kernel doesn't require data migration

### 11.3 Exception: Tessellation Cache

Tessellation results could be persisted as an optimization cache, clearly
marked as derived data. If the cache is invalid (version mismatch), it is
regenerated transparently.

---

## 12. Performance Architecture

### 12.1 Caching Layers

```text
Layer 1: Parametric Cache (element → params hash)
    - Detects if element has changed
    - Invalidates downstream caches

Layer 2: Solid Cache (params hash → B-Rep solid)
    - Avoids regenerating unchanged solids
    - Keyed by element ID + param version

Layer 3: Mesh Cache (solid hash → triangle mesh)
    - Avoids retessellating unchanged solids
    - Keyed by solid ID + tessellation quality

Layer 4: Spatial Index (all solids → AABB tree)
    - Enables frustum culling and proximity queries
    - Rebuilt incrementally when solids change
```

### 12.2 Incremental Rebuild

When an element changes:
1. Invalidate its solid cache entry
2. Regenerate solid from new params
3. Invalidate its mesh cache entry
4. Retessellate with current quality settings
5. Update spatial index for this element only
6. Update rendering for this element only

The rest of the model is unaffected.

### 12.3 Lazy Generation

Solids are generated on demand:
- Only elements in the viewport need tessellation
- LOD: distant elements use coarser tessellation
- Hidden elements don't need solids at all
- Quantities (volume, area) need solids but not meshes

### 12.4 Instancing

For repeated elements (same section, same material):
- Generate one solid, reuse for all instances
- Generate one mesh, render with transforms
- Saves memory and tessellation time

---

## 13. AI Future Compatibility

### 13.1 AI → Command → Model → Geometry

```text
AI Intent: "Add a 300x600 column at grid A-1, level 1 to level 2"
    │
    ▼
Validated Command: CreateColumnCommand { ... }
    │  - Validated by model rules
    │  - References exist (levels, section, material)
    │
    ▼
Engineering Model mutation
    │  - Element added to model
    │  - revision++
    │
    ▼
Parametric Geometry generation
    │  - Solid generated from element params
    │  - Cached by element ID
    │
    ▼
Tessellation
    │  - Mesh generated from solid
    │  - Cached for rendering
    │
    ▼
Viewport update
```

### 13.2 What AI Must NOT Touch

- AI must not write directly to geometry cache
- AI must not modify tessellation
- AI must not bypass the command system
- AI must produce validated commands, not raw geometry

---

## 14. Risks

### 14.1 Kernel Selection Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Wrong kernel chosen | Expensive to switch later | Kernel adapter trait isolates model |
| OCCT build breaks | Blocks development | Use `cadrum` with prebuilt binaries |
| Truck boolean failures | Incorrect geometry | Fallback to OCCT for booleans |
| WASM binary too large | Slow web loading | Geometry on server, not client |
| License changes | Legal issues | LGPL 2.1+exception is stable |

### 14.2 Architecture Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Geometry leaks into model | Coupling, hard to change | Strict layer boundaries |
| Cache invalidation bugs | Stale geometry displayed | Version-based invalidation |
| Performance regression | Slow large models | Benchmark suite, profiling |
| Precision issues | Incorrect quantities | Engineering tolerance checks |

### 14.3 Migration Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Kernel change requires rewrite | Months of work | Adapter trait from day one |
| Data format change | Old files unreadable | Geometry not persisted |
| API breaking changes | Code updates needed | Version-pinned dependencies |

---

## 15. Recommendation Options

### Option A: `cadrum` (OpenCascade)

**RECOMMENDED**

**WHY:**
- Industrial-grade B-Rep: 25+ years of production use
- Most robust boolean operations available
- Native STEP/BRep I/O (critical for BIM interop)
- WASM support via Docker pre-built OCCT
- Active development (40 releases in 6 months)
- Statically linked, no system dependencies
- Prebuilt binaries for all target platforms including Android and iOS
- Used by FreeCAD, IfcOpenShell, and many BIM tools
- LGPL 2.1 with exception allows commercial distribution

**TRADE-OFFS:**
- Larger binary size (~50-150 MB static, ~15-30 MB WASM)
- Requires C++ toolchain for building from source (but prebuilt binaries available)
- OCCT API is C++-oriented (mitigated by cadrum's high-level wrapper)
- More complex build than pure Rust

**WHAT WE GAIN:**
- Battle-tested boolean operations
- STEP/BRep interop from day one
- Industry-standard kernel for BIM
- Confidence in large model handling
- Foundation for IFC export (future)

**WHAT WE LOSE:**
- Pure Rust simplicity
- Smaller binary size
- Faster compile times

### Option B: `monstertruck` (Truck fork)

**ALTERNATIVE**

**WHY:**
- Pure Rust, no C++ dependency
- Apache 2.0 license (simpler than LGPL)
- Active fork with modern Rust practices
- NURBS-native implementation
- WASM compatible
- Smaller binary size
- Faster compile times

**TRADE-OFFS:**
- Boolean operations less robust than OCCT
- No native STEP support
- Smaller community, less battle-tested
- May need to fix bugs ourselves
- API still evolving

**WHAT WE GAIN:**
- Simpler build process
- Smaller binaries
- Pure Rust codebase
- Faster iteration

**WHAT WE LOSE:**
- Industrial-grade robustness
- STEP/BRep I/O (must implement or add library)
- Proven track record in BIM tools
- Confidence in boolean operations

### Option C: Hybrid (Start with Truck, fallback to OCCT)

**POSSIBLE BUT RISKY**

**WHY:**
- Start simple with Truck
- Add OCCT adapter if Truck fails on boolean requirements
- Both behind the same adapter trait

**TRADE-OFFS:**
- Two kernels to maintain
- Integration testing complexity
- Risk of building on wrong foundation

---

## 16. Open Questions

1. **Binary size budget**: How large can the Flutter APK/IPA be? If <50 MB is required, OCCT may be too large for mobile. Truck would fit better.

2. **WASM priority**: Is WASM a Day 1 requirement or a future enhancement? If Day 1, Truck's smaller WASM binary is advantageous.

3. **STEP requirement**: How critical is STEP import/export? If essential for the MVP, OCCT is the only complete option.

4. **Boolean robustness**: How complex are our boolean operations? Simple openings in walls, or complex multi-element junctions? Simple → Truck may suffice.

5. **Team C++ comfort**: Is the team comfortable maintaining C++ build tooling? If not, pure Rust (Truck) reduces operational burden.

6. **IFC timeline**: When is IFC export planned? IFC uses B-Rep, and OCCT is the standard kernel for IFC. Early IFC work favors OCCT.

7. **Performance target**: What is the maximum number of elements we need to handle interactively? >10k elements favors OCCT's optimized internals.

8. **License preference**: Is Apache 2.0 preferred over LGPL 2.1+exception? Both are commercially viable, but Apache is simpler.

---

## 17. Proposed Next Step

**STEP 4B** (when the user is ready) should:

1. **Choose one kernel** based on this evaluation and the user's priorities
2. **Implement the Kernel Adapter trait** in Rust, behind an abstraction layer
3. **Create a prototype adapter** for the chosen kernel
4. **Test basic operations**: extrude a column, extrude a beam, boolean a wall opening
5. **Measure binary size, build time, and WASM feasibility**
6. **Validate the adapter architecture** allows kernel swapping

The adapter trait is the critical architectural decision. It must be designed
so that:
- The Engineering Model never imports kernel types
- Swapping kernels requires only implementing the trait
- The model code compiles and tests without any kernel dependency
- Geometry generation is isolated in a separate module

---

## 18. Verification

```bash
cd core/geometry_kernel_rs && cargo fmt --all -- --check && cargo check --all-targets && cargo test
cd ../../apps/client_flutter && flutter pub get && flutter analyze && flutter test
```

All checks pass. No code was modified in this step — only documentation was
created.
