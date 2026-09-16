# Project Format — `.civilx`

Documents the persistence format for engineering projects. The format wraps an
[`EngineeringModel`](ENGINEERING_MODEL.md) with lightweight organisational
metadata and a header that declares versioning information.

---

## 1. What is `.civilx`?

A `.civilx` file is the serialised representation of a
[`Project`](../core/geometry_kernel_rs/src/project/mod.rs). It contains:

```text
┌──────────────────────────────────────────────────┐
│  Header (JSON)                                   │
│  ├── format_version: u32                         │
│  ├── model_schema_version: u32                   │
│  ├── project_id: Uuid                            │
│  └── reserved: u32                               │
├──────────────────────────────────────────────────┤
│  Metadata (JSON)                                 │
│  ├── name: String                                │
│  ├── description: String                         │
│  ├── created_at: DateTime<Utc>                   │
│  └── modified_at: DateTime<Utc>                  │
├──────────────────────────────────────────────────┤
│  Engineering Model (JSON)                        │
│  ├── schema_version                              │
│  ├── project_id                                  │
│  ├── revision                                    │
│  ├── levels, grids, materials, cross_sections    │
│  └── elements                                    │
└──────────────────────────────────────────────────┘
```

Each segment is a self-contained JSON value separated by a newline (`\n`).
This makes the file easy to inspect with `jq` and simple to parse without a
streaming parser.

---

## 2. Format Version vs. Model Schema Version

These are **two different version numbers** that evolve independently:

| Version | What it tracks | Constant |
|---|---|---|
| **Format version** | How bytes are structured on disk | `PROJECT_FORMAT_VERSION = 1` |
| **Model schema version** | What fields the engineering model carries | `MODEL_SCHEMA_VERSION = 1` |

A format v3 file may still carry a schema v1 model. This separation allows
the file container to evolve (e.g. adding compression, encryption, or a
binary layout) without changing the engineering domain model, and vice versa.

---

## 3. EngineeringModel as Source of Truth

The [`EngineeringModel`](../core/geometry_kernel_rs/src/model/engineering_model.rs)
inside the project file is the **domain source of truth**. The `Project`
wrapper adds:

- **name** — human-readable project title
- **description** — optional documentation
- **created_at / modified_at** — organisational timestamps

Timestamps are *metadata*, not domain state. They do not affect geometry,
calculations, model revision, or validation. They never change the
engineering model's revision counter.

---

## 4. Serialization

Serialisation is handled by the [`ProjectSerializer`] trait:

```rust
pub trait ProjectSerializer {
    fn serialize(project: &Project) -> Result<Vec<u8>, ProjectError>;
    fn deserialize(data: &[u8]) -> Result<Project, ProjectError>;
}
```

The concrete implementation [`ProjectFormatV1`] uses JSON for all three
segments (header, metadata, model). The model segment uses compact JSON
(no newlines within the model) so the three-segment layout is unambiguous.

---

## 5. Deterministic Representation

Serialisation is **deterministic**: the same logical model always yields the
same bytes. This is guaranteed because:

- All model collections use `BTreeMap<Uuid, T>` (sorted by key)
- `serde_json` serialises BTreeMaps in sorted key order
- No random values, no runtime-generated timestamps in domain state
- Metadata timestamps are set once at creation (they vary between
  `Project::new()` calls, but a single project serialised twice yields
  the same bytes)

This property is important for:

- Content-addressed storage (future)
- Diff comparison between saves
- Reproducible hashes for collaboration (future)

---

## 6. Validation During Loading

When a `.civilx` file is loaded, two validation stages run:

### 6.1 Format validation (ProjectError)

The loader checks the header before parsing the payload:

| Check | Error variant |
|---|---|
| File is empty or not UTF-8 | `InvalidPayload` |
| Header is missing or malformed | `InvalidHeader` |
| `format_version > PROJECT_FORMAT_VERSION` | `UnsupportedFormatVersion` |
| `model_schema_version > MODEL_SCHEMA_VERSION` | `UnsupportedModelSchema` |
| Header `project_id` ≠ model `project_id` | `InvalidHeader` |
| Header `model_schema_version` ≠ model `schema_version` | `InvalidHeader` |

A file from a newer kernel (higher format or schema version) is **rejected
immediately** — no silent repair, no partial loading.

### 6.2 Model validation

After format validation passes, the engineering model is validated using the
same `model.validate()` mechanism used during editing. Issues are reported as
`ProjectError::ModelValidationFailed`.

---

## 7. Revision Semantics

The model's `revision` counter records the number of accepted mutations:

| Event | revision change |
|---|---|
| New model created | `0` |
| Mutation accepted (add level, add element, ...) | `+1` |
| Mutation rejected (validation failure) | unchanged |
| Serialisation | unchanged |
| Deserialisation | unchanged |
| Validation | unchanged |
| Querying | unchanged |

The revision survives a save/load round-trip and is available to Flutter
through `ProjectSummary.revision`.

---

## 8. Security and Robustness

The deserialisation path treats file data as **untrusted input**:

- No `unwrap()` or `expect()` in parsing paths
- No panic on corrupted data — always returns `ProjectError`
- No silent repair of invalid data
- No execution of arbitrary code from project files
- Maximum file size enforced (100 MiB for v1)

---

## 9. Why Database is Deferred

Database storage (SQLite, PostgreSQL, etc.) is a **storage layer** concern,
not a domain model concern. It is deferred because the team needs to
evaluate:

- Project sizes and performance requirements
- Incremental save needs
- Cloud synchronisation patterns
- Collaboration and conflict resolution
- Binary geometry and IFC data
- Backward compatibility strategies

The `ProjectSerializer` trait is designed so that a database-backed
serialiser can be added later without changing the `Project` or
`EngineeringModel` types.

---

## 10. Why Event Sourcing is Deferred

Event sourcing (command journal, undo/redo, collaboration protocol) is
deferred because:

- The `ModelCommand` trait already defines the right shape
- `Command → Model mutation → revision++` is the established boundary
- Events, undo stacks and collaboration protocols can be layered on top
  without redesigning the model

The current architecture is explicitly designed to make this future step
straightforward: commands are serialisable, the revision counter tracks
mutations, and the model is deterministic.

---

## 11. Future Format Evolution

The format is designed for independent evolution of its layers:

```text
.civilx format version 3
  └── Model schema version 5
       └── EngineeringModel (with new element types, new properties, ...)
```

Potential future format changes:

- **Binary header** (replace JSON header for faster loading)
- **Compression** (gzip/zstd for large projects)
- **Encryption** (AES-GCM for confidential projects)
- **Chunked layout** (separate metadata, model, and geometry chunks)
- **Versioned metadata** (author, organisation, license, tags)

The `reserved` field in the header is allocated for forward-compatible
extensions that do not change the header layout.

---

## 12. API Reference

### Rust (kernel)

```rust
use geometry_kernel_rs::project::{Project, PROJECT_FORMAT_VERSION};
use geometry_kernel_rs::project::format::{ProjectFormatV1, ProjectSerializer};

// Create
let project = Project::new("My Building");

// Save
let bytes: Vec<u8> = ProjectFormatV1::serialize(&project)?;

// Load
let restored: Project = ProjectFormatV1::deserialize(&bytes)?;

// File I/O
use geometry_kernel_rs::project::format::{save_project_to, load_project_from};
save_project_to(&mut writer, &project)?;
let loaded = load_project_from(&mut reader)?;
```

### Summary (FFI-safe)

```rust
let summary: ProjectSummary = project.summary();
// summary.format_version, summary.project_id, summary.name, summary.revision, ...
```

---

## 13. Examples

### Empty project

```text
Project {
  metadata: { name: "Empty", description: "", created_at: ..., modified_at: ... },
  model: { revision: 0, levels: {}, grids: {}, materials: {}, cross_sections: {}, elements: {} }
}
```

### Complete project

```text
Project {
  metadata: { name: "Complete Project", ... },
  model: {
    revision: 7,
    levels: { Level 1 (0.0m), Level 2 (3.2m) },
    grids: { A, 1 },
    materials: { Concrete C30 },
    cross_sections: { 400×400 mm },
    elements: { C001, B001, SL001, W001, F001 }
  }
}
```
