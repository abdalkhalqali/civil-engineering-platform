//! Test suite for the project persistence layer (STEP 3).
//!
//! Tests 1..14 are the acceptance tests requested for this step.

use uuid::Uuid;

use geometry_kernel_rs::commands::{AddLevelCommand, ModelCommand};
use geometry_kernel_rs::elements::{
    BaseElement, Element, ElementCategory, Foundation, FoundationType, PlanBoundary,
    StructuralBeam, StructuralColumn, StructuralSlab, StructuralWall,
};
use geometry_kernel_rs::math::Point3D;
use geometry_kernel_rs::model::{CrossSection, Level, Material, MODEL_SCHEMA_VERSION};
use geometry_kernel_rs::project::format::{ProjectFormatV1, ProjectSerializer};
use geometry_kernel_rs::project::{Project, ProjectError, PROJECT_FORMAT_VERSION};
use geometry_kernel_rs::units::{Length, Stress};

const EPSILON: f64 = 1e-12;

fn mm(value: f64) -> Length {
    Length::from_millimeters(value)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

// ── Helper: build a complete project with all element kinds ─────────────

fn build_complete_project() -> Project {
    let mut project = Project::new("Complete Project");
    let model = &mut project.model;

    let l1 = model
        .add_level(Level::new("Level 1", Length::from_meters(0.0)))
        .unwrap();
    let l2 = model
        .add_level(Level::new("Level 2", Length::from_meters(3.2)))
        .unwrap();

    model
        .add_grid(geometry_kernel_rs::model::Grid::along_y(
            "A",
            Length::from_meters(0.0),
        ))
        .unwrap();
    model
        .add_grid(geometry_kernel_rs::model::Grid::along_x(
            "1",
            Length::from_meters(0.0),
        ))
        .unwrap();

    let mat = model.add_material(Material::concrete_c30()).unwrap();
    let sec = model
        .add_cross_section(CrossSection::rectangular("400x400", mm(400.0), mm(400.0)))
        .unwrap();

    model
        .add_element(Element::Column(StructuralColumn::new(
            BaseElement::new(ElementCategory::Column, "C001"),
            l1,
            l2,
            sec,
            mat,
        )))
        .unwrap();

    model
        .add_element(Element::Beam(StructuralBeam::new(
            BaseElement::new(ElementCategory::Beam, "B001"),
            l2,
            Point3D::from_meters(0.0, 0.0, 3.2),
            Point3D::from_meters(6.0, 0.0, 3.2),
            sec,
            mat,
        )))
        .unwrap();

    model
        .add_element(Element::Slab(StructuralSlab::new(
            BaseElement::new(ElementCategory::Slab, "SL001"),
            l2,
            mm(200.0),
            mat,
            PlanBoundary::rectangle(
                Point3D::from_meters(0.0, 0.0, 3.2),
                Length::from_meters(6.0),
                Length::from_meters(5.0),
            ),
        )))
        .unwrap();

    model
        .add_element(Element::Wall(StructuralWall::new(
            BaseElement::new(ElementCategory::Wall, "W001"),
            l1,
            l2,
            Point3D::from_meters(0.0, 0.0, 0.0),
            Point3D::from_meters(6.0, 0.0, 0.0),
            mm(250.0),
            mat,
        )))
        .unwrap();

    model
        .add_element(Element::Foundation(Foundation::new(
            BaseElement::new(ElementCategory::Foundation, "F001"),
            l1,
            mm(600.0),
            mat,
            FoundationType::Isolated,
            PlanBoundary::rectangle(
                Point3D::from_meters(0.0, 0.0, 0.0),
                Length::from_meters(2.0),
                Length::from_meters(2.0),
            ),
        )))
        .unwrap();

    project
}

// ── Test 1: Create project ─────────────────────────────────────────────

#[test]
fn test_1_create_project() {
    let project = Project::new("My Project");

    assert_eq!(project.metadata.name, "My Project");
    assert!(project.metadata.description.is_empty());
    assert_eq!(project.model.revision, 0);
    assert_eq!(project.model.schema_version, MODEL_SCHEMA_VERSION);
    assert!(project.model.is_empty());
    assert_eq!(project.model.element_count(), 0);
    assert!(project.model.validate().is_valid());
    assert_eq!(project.project_id(), project.model.project_id);

    let summary = project.summary();
    assert_eq!(summary.format_version, PROJECT_FORMAT_VERSION);
    assert_eq!(summary.model_schema_version, MODEL_SCHEMA_VERSION);
    assert_eq!(summary.project_id.len(), 36);
    assert_eq!(summary.project_id.matches('-').count(), 4);
    assert_eq!(summary.name, "My Project");
    assert_eq!(summary.revision, 0);
    assert_eq!(summary.elements, 0);
}

// ── Test 2: Save project ───────────────────────────────────────────────

#[test]
fn test_2_save_project_succeeds() {
    let project = Project::new("Save Test");
    let bytes = ProjectFormatV1::serialize(&project);

    assert!(bytes.is_ok());
    let bytes = bytes.unwrap();
    assert!(!bytes.is_empty());
    assert!(bytes.len() > 10);

    // The bytes should contain recognizable JSON segments.
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("\"format_version\""));
    assert!(text.contains("\"model_schema_version\""));
    assert!(text.contains("\"project_id\""));
    assert!(text.contains("\"name\""));
}

// ── Test 3: Load project ───────────────────────────────────────────────

#[test]
fn test_3_load_project_returns_correct_project() {
    let mut project = Project::new("Load Test");
    project.model.add_level(Level::new("L1", Length::ZERO)).ok();

    let bytes = ProjectFormatV1::serialize(&project).unwrap();
    let loaded = ProjectFormatV1::deserialize(&bytes);

    assert!(loaded.is_ok());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.metadata.name, "Load Test");
    assert_eq!(loaded.project_id(), project.project_id());
    assert_eq!(loaded.model.revision, 1);
    assert_eq!(loaded.model.levels.len(), 1);
}

// ── Test 4: Deterministic round trip ───────────────────────────────────

#[test]
fn test_4_round_trip_is_deterministic() {
    let project = Project::new("Deterministic");

    let bytes_a = ProjectFormatV1::serialize(&project).unwrap();
    let bytes_b = ProjectFormatV1::serialize(&project).unwrap();

    // Same model → same bytes (deterministic).
    assert_eq!(bytes_a, bytes_b);

    // And the double-deserialize produces identical projects.
    let loaded_a = ProjectFormatV1::deserialize(&bytes_a).unwrap();
    let loaded_b = ProjectFormatV1::deserialize(&bytes_b).unwrap();
    assert_eq!(loaded_a, loaded_b);
}

// ── Test 5: Identity preservation ──────────────────────────────────────

#[test]
fn test_5_uuids_are_preserved_through_serialisation() {
    let mut project = Project::new("Identity Test");
    let level_id = project
        .model
        .add_level(Level::new("L1", Length::from_meters(0.0)))
        .unwrap();
    let material_id = project
        .model
        .add_material(Material::concrete_c30())
        .unwrap();
    let section_id = project
        .model
        .add_cross_section(CrossSection::rectangular("300x300", mm(300.0), mm(300.0)))
        .unwrap();
    let column_id = project
        .model
        .add_element(Element::Column(StructuralColumn::new(
            BaseElement::new(ElementCategory::Column, "C001"),
            level_id,
            level_id,
            section_id,
            material_id,
        )))
        .unwrap();

    let project_id_before = project.project_id();
    let bytes = ProjectFormatV1::serialize(&project).unwrap();
    let loaded = ProjectFormatV1::deserialize(&bytes).unwrap();

    // All ids survived.
    assert_eq!(loaded.project_id(), project_id_before);
    assert!(loaded.model.level(level_id).is_some());
    assert!(loaded.model.material(material_id).is_some());
    assert!(loaded.model.cross_section(section_id).is_some());
    assert!(loaded.model.element(column_id).is_some());

    // Name survived.
    let column = loaded.model.element(column_id).unwrap();
    assert_eq!(column.name(), "C001");
}

// ── Test 6: Revision preservation ──────────────────────────────────────

#[test]
fn test_6_revision_is_preserved_through_save_and_load() {
    let mut project = Project::new("Revision Test");
    // Add 5 levels to bump revision to 5.
    for i in 0..5 {
        project
            .model
            .add_level(Level::new(format!("L{i}"), Length::from_meters(i as f64)))
            .unwrap();
    }
    let revision_before = project.revision();
    assert_eq!(revision_before, 5);

    let bytes = ProjectFormatV1::serialize(&project).unwrap();
    let loaded = ProjectFormatV1::deserialize(&bytes).unwrap();
    assert_eq!(loaded.revision(), 5);
}

// ── Test 7: References preservation ────────────────────────────────────

#[test]
fn test_7_references_between_elements_are_preserved() {
    let mut project = Project::new("Ref Test");
    let model = &mut project.model;

    let l1 = model.add_level(Level::new("Base", Length::ZERO)).unwrap();
    let l2 = model
        .add_level(Level::new("Top", Length::from_meters(3.5)))
        .unwrap();
    let mat = model.add_material(Material::concrete_c30()).unwrap();
    let sec = model
        .add_cross_section(CrossSection::rectangular("400x400", mm(400.0), mm(400.0)))
        .unwrap();

    model
        .add_element(Element::Column(StructuralColumn::new(
            BaseElement::new(ElementCategory::Column, "C001"),
            l1,
            l2,
            sec,
            mat,
        )))
        .unwrap();

    model
        .add_element(Element::Beam(StructuralBeam::new(
            BaseElement::new(ElementCategory::Beam, "B001"),
            l2,
            Point3D::ORIGIN,
            Point3D::from_meters(5.0, 0.0, 0.0),
            sec,
            mat,
        )))
        .unwrap();

    let bytes = ProjectFormatV1::serialize(&project).unwrap();
    let loaded = ProjectFormatV1::deserialize(&bytes).unwrap();

    // All references survive.
    for element in loaded.model.elements.values() {
        for (kind, id) in element.referenced_ids() {
            match kind {
                geometry_kernel_rs::error::ReferenceKind::Level => {
                    assert!(
                        loaded.model.level(id).is_some(),
                        "level {id} referenced by {} is missing",
                        element.name()
                    );
                }
                geometry_kernel_rs::error::ReferenceKind::Material => {
                    assert!(
                        loaded.model.material(id).is_some(),
                        "material {id} referenced by {} is missing",
                        element.name()
                    );
                }
                geometry_kernel_rs::error::ReferenceKind::CrossSection => {
                    assert!(
                        loaded.model.cross_section(id).is_some(),
                        "section {id} referenced by {} is missing",
                        element.name()
                    );
                }
                _ => {}
            }
        }
    }
}

// ── Test 8: Invalid format version ─────────────────────────────────────

#[test]
fn test_8_invalid_format_version_is_rejected() {
    let project = Project::new("Version Reject");
    let bytes = ProjectFormatV1::serialize(&project).unwrap();

    // Tamper with format_version.
    let header_end = bytes.iter().position(|&b| b == b'\n').unwrap();
    let header_str = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    let tampered = header_str.replace(
        &format!("\"format_version\":{PROJECT_FORMAT_VERSION}"),
        "\"format_version\":999",
    );
    let mut new_bytes = tampered.into_bytes();
    new_bytes.extend_from_slice(&bytes[header_end..]);

    let result = ProjectFormatV1::deserialize(&new_bytes);
    assert!(result.is_err());
    match result.unwrap_err() {
        ProjectError::UnsupportedFormatVersion { found, .. } => {
            assert_eq!(found, 999);
        }
        other => panic!("expected UnsupportedFormatVersion, got {other:?}"),
    }
}

// ── Test 9: Invalid model schema ───────────────────────────────────────

#[test]
fn test_9_invalid_model_schema_is_rejected() {
    let project = Project::new("Schema Reject");
    let bytes = ProjectFormatV1::serialize(&project).unwrap();

    // Tamper with model_schema_version in header AND payload so they disagree.
    let header_end = bytes.iter().position(|&b| b == b'\n').unwrap();
    let mut header_str = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    header_str = header_str.replace(
        &format!("\"model_schema_version\":{MODEL_SCHEMA_VERSION}"),
        "\"model_schema_version\":999",
    );
    let mut new_bytes = header_str.into_bytes();
    new_bytes.extend_from_slice(&bytes[header_end..]);

    let result = ProjectFormatV1::deserialize(&new_bytes);
    assert!(result.is_err());
    match result.unwrap_err() {
        ProjectError::UnsupportedModelSchema { found, .. } => {
            assert_eq!(found, 999);
        }
        other => panic!("expected UnsupportedModelSchema, got {other:?}"),
    }
}

// ── Test 10: Corrupted payload ─────────────────────────────────────────

#[test]
fn test_10_corrupted_payload_does_not_panic() {
    // Completely garbage data.
    let garbage = b"this is not a valid project file at all";
    let result = ProjectFormatV1::deserialize(garbage);
    assert!(result.is_err());
    // No panic.

    // Binary garbage.
    let binary: Vec<u8> = (0..256).map(|i| i as u8).collect();
    let result = ProjectFormatV1::deserialize(&binary);
    assert!(result.is_err());
    // No panic.

    // Empty file.
    let result = ProjectFormatV1::deserialize(b"");
    assert!(result.is_err());

    // Valid JSON but wrong structure.
    let result = ProjectFormatV1::deserialize(b"{}\n");
    assert!(result.is_err());
}

// ── Test 11: Rejected mutation doesn't change revision ─────────────────

#[test]
fn test_11_rejected_mutation_preserves_revision() {
    let mut project = Project::new("Reject Test");

    // Set up references first, capture revision while no mutable borrow exists.
    let level = project
        .model
        .add_level(Level::new("L1", Length::ZERO))
        .unwrap();
    let mat = project
        .model
        .add_material(Material::concrete_c30())
        .unwrap();
    let sec = project
        .model
        .add_cross_section(CrossSection::rectangular("300x300", mm(300.0), mm(300.0)))
        .unwrap();
    let revision_before = project.model.revision;

    // This should fail: column references a non-existent level.
    let foreign_id = Uuid::new_v4();
    let result = project
        .model
        .add_element(Element::Column(StructuralColumn::new(
            BaseElement::new(ElementCategory::Column, "C_BAD"),
            foreign_id, // doesn't exist
            level,
            sec,
            mat,
        )));
    assert!(result.is_err());

    // Revision unchanged.
    assert_eq!(project.model.revision, revision_before);
}

// ── Test 12: Successful command increments revision ────────────────────

#[test]
fn test_12_successful_command_increments_revision() {
    let mut project = Project::new("Command Test");
    let revision_before = project.revision();
    assert_eq!(revision_before, 0);

    let cmd = AddLevelCommand::new("Roof", Length::from_meters(6.4));
    cmd.apply(&mut project.model).unwrap();

    assert_eq!(project.revision(), 1);

    // Serialize and verify revision survives.
    let bytes = ProjectFormatV1::serialize(&project).unwrap();
    let loaded = ProjectFormatV1::deserialize(&bytes).unwrap();
    assert_eq!(loaded.revision(), 1);
    assert!(loaded.model.level(Uuid::nil()).is_none() || loaded.model.levels.len() == 1);
}

// ── Test 13: Empty project round trip ───────────────────────────────────

#[test]
fn test_13_empty_project_save_and_load() {
    let project = Project::new("Empty");

    let bytes = ProjectFormatV1::serialize(&project).unwrap();
    let loaded = ProjectFormatV1::deserialize(&bytes).unwrap();

    assert_eq!(loaded.project_id(), project.project_id());
    assert_eq!(loaded.metadata.name, "Empty");
    assert_eq!(loaded.revision(), 0);
    assert!(loaded.model.is_empty());
    assert!(loaded.model.validate().is_valid());
}

// ── Test 14: Complete engineering model round trip ─────────────────────

#[test]
fn test_14_complete_model_save_load_validate() {
    let project = build_complete_project();

    // Verify initial state.
    assert_eq!(project.model.levels.len(), 2);
    assert_eq!(project.model.grids.len(), 2);
    assert!(project.model.materials.len() >= 1);
    assert!(project.model.cross_sections.len() >= 1);
    assert_eq!(project.model.elements.len(), 5); // C001, B001, SL001, W001, F001
    assert!(project.model.validate().is_valid());

    let project_id = project.project_id();
    let revision = project.revision();
    let summary_before = project.summary();

    // Serialize.
    let bytes = ProjectFormatV1::serialize(&project).unwrap();

    // Deserialize.
    let loaded = ProjectFormatV1::deserialize(&bytes).unwrap();

    // Identity.
    assert_eq!(loaded.project_id(), project_id);
    assert_eq!(loaded.metadata.name, "Complete Project");

    // Revision.
    assert_eq!(loaded.revision(), revision);

    // Validation.
    assert!(loaded.model.validate().is_valid());

    // Summary.
    let summary_after = loaded.summary();
    assert_eq!(summary_before, summary_after);

    // Elements survived with correct types.
    let mut found_kinds = std::collections::HashSet::new();
    for element in loaded.model.elements.values() {
        found_kinds.insert(element.category());
    }
    assert!(found_kinds.contains(&ElementCategory::Column));
    assert!(found_kinds.contains(&ElementCategory::Beam));
    assert!(found_kinds.contains(&ElementCategory::Slab));
    assert!(found_kinds.contains(&ElementCategory::Wall));
    assert!(found_kinds.contains(&ElementCategory::Foundation));

    // Dimensions survived.
    let column = loaded
        .model
        .elements
        .values()
        .find(|e| e.name() == "C001")
        .unwrap()
        .as_column()
        .unwrap();
    let base = loaded.model.level(column.base_level_id).unwrap();
    let top = loaded.model.level(column.top_level_id).unwrap();
    let (bottom, top_elev) = column.extent(base.elevation, top.elevation);
    assert!(close(bottom.meters(), 0.0));
    assert!(close(top_elev.meters(), 3.2));

    // Properties survived.
    let beam = loaded
        .model
        .elements
        .values()
        .find(|e| e.name() == "B001")
        .unwrap()
        .as_beam()
        .unwrap();
    assert!(close(beam.length().meters(), 6.0));

    let slab = loaded
        .model
        .elements
        .values()
        .find(|e| e.name() == "SL001")
        .unwrap()
        .as_slab()
        .unwrap();
    assert!(close(slab.boundary_area().square_meters(), 30.0));

    // Material survived.
    let material = loaded.model.materials.values().next().unwrap();
    assert_eq!(material.name, "Concrete C30");
    assert_eq!(
        material.compressive_strength,
        Stress::from_megapascals(30.0)
    );

    // Cross section survived.
    let section = loaded.model.cross_sections.values().next().unwrap();
    assert!(close(section.area().square_meters(), 0.16));
}
