//! Test suite of the engineering model core.
//!
//! Tests 1..8 are the acceptance tests requested for this step; the remaining tests
//! cover the invariants, the command extension point and the physical problems the
//! model is expected to report around them.

use uuid::Uuid;

use geometry_kernel_rs::commands::{AddLevelCommand, ModelCommand};
use geometry_kernel_rs::elements::{
    BaseElement, Element, ElementCategory, Foundation, FoundationType, Justification, PlanBoundary,
    StructuralBeam, StructuralColumn, StructuralSlab, StructuralWall,
};
use geometry_kernel_rs::error::{ModelError, ReferenceKind};
use geometry_kernel_rs::math::{Point3D, Rotation3D};
use geometry_kernel_rs::model::{
    CrossSection, EngineeringModel, Grid, IBeamProfile, Level, Material, MaterialType, ProfileType,
    ValidationCode, ValidationMode, MODEL_SCHEMA_VERSION,
};
use geometry_kernel_rs::units::{Angle, Length, MassDensity, Stress};

/// Tolerance for comparisons of floating point quantities derived by arithmetic.
const EPSILON: f64 = 1e-12;

fn mm(value: f64) -> Length {
    Length::from_millimeters(value)
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < EPSILON
}

/// The acceptance scenario of this step: two levels, four grid lines, one material,
/// one cross section, one column and one beam.
struct Project {
    model: EngineeringModel,
    level_1: Uuid,
    level_2: Uuid,
    material: Uuid,
    section: Uuid,
    column: Uuid,
    beam: Uuid,
}

fn build_project() -> Project {
    let mut model = EngineeringModel::new();

    let level_1 = model
        .add_level(Level::new("Level 1", Length::from_meters(0.0)))
        .expect("level 1");
    let level_2 = model
        .add_level(Level::new("Level 2", Length::from_meters(3.2)))
        .expect("level 2");

    model
        .add_grid(Grid::along_y("A", Length::from_meters(0.0)))
        .expect("grid A");
    model
        .add_grid(Grid::along_y("B", Length::from_meters(6.0)))
        .expect("grid B");
    model
        .add_grid(Grid::along_x("1", Length::from_meters(0.0)))
        .expect("grid 1");
    model
        .add_grid(Grid::along_x("2", Length::from_meters(5.0)))
        .expect("grid 2");

    let material = model
        .add_material(Material::concrete_c30())
        .expect("material");
    let section = model
        .add_cross_section(CrossSection::rectangular("400x400", mm(400.0), mm(400.0)))
        .expect("section");

    let column = model
        .add_element(Element::Column(StructuralColumn::new(
            BaseElement::new(ElementCategory::Column, "C001"),
            level_1,
            level_2,
            section,
            material,
        )))
        .expect("column C001");

    let beam = model
        .add_element(Element::Beam(StructuralBeam::new(
            BaseElement::new(ElementCategory::Beam, "B001"),
            level_2,
            Point3D::from_meters(0.0, 0.0, 3.2),
            Point3D::from_meters(6.0, 0.0, 3.2),
            section,
            material,
        )))
        .expect("beam B001");

    Project {
        model,
        level_1,
        level_2,
        material,
        section,
        column,
        beam,
    }
}

// -----------------------------------------------------------------------------
// Test 1 — create model
// -----------------------------------------------------------------------------

#[test]
fn test_1_create_empty_model() {
    let model = EngineeringModel::new();

    assert_eq!(model.schema_version, MODEL_SCHEMA_VERSION);
    assert_eq!(model.revision, 0);
    assert!(model.is_empty());
    assert_eq!(model.element_count(), 0);
    assert!(model.validate().is_valid());

    let summary = model.summary();
    assert!(summary.is_empty());
    // The project id is exposed as a hyphenated UUID shaped string.
    assert_eq!(summary.project_id.len(), 36);
    assert_eq!(summary.project_id.matches('-').count(), 4);
}

// -----------------------------------------------------------------------------
// Test 2 — level
// -----------------------------------------------------------------------------

#[test]
fn test_2_level_keeps_id_name_and_elevation() {
    let mut model = EngineeringModel::new();

    let level_1 = Level::new("Level 1", Length::from_meters(0.0));
    let level_2 = Level::new("Level 2", Length::from_meters(3.2));
    assert_ne!(level_1.id, level_2.id, "every level gets a stable identity");

    let id_1 = model.add_level(level_1).expect("level 1");
    let id_2 = model.add_level(level_2).expect("level 2");

    let stored = model.level(id_1).expect("stored level 1");
    assert_eq!(stored.id, id_1);
    assert_eq!(stored.name, "Level 1");
    assert_eq!(stored.elevation_meters(), 0.0);

    let upper = model.level(id_2).expect("stored level 2");
    assert_eq!(upper.name, "Level 2");
    // 3.2 m is stored as 3.2 m, whatever unit the user typed in.
    assert_eq!(upper.elevation, Length::from_meters(3.2));
    assert_eq!(upper.elevation, Length::from_millimeters(3200.0));
    assert!(upper.is_above(stored));

    assert_eq!(model.levels.len(), 2);
    assert!(model.validate().is_valid());
}

// -----------------------------------------------------------------------------
// Test 3 — material
// -----------------------------------------------------------------------------

#[test]
fn test_3_material_creation() {
    let mut model = EngineeringModel::new();

    let id = model
        .add_material(Material::concrete_c30())
        .expect("material");
    let material = model.material(id).expect("stored material");

    assert_eq!(material.id, id);
    assert_eq!(material.name, "Concrete C30");
    assert_eq!(material.material_type, MaterialType::Concrete);
    assert_eq!(material.density.kilograms_per_cubic_meter(), 2_500.0);
    assert_eq!(material.compressive_strength.megapascals(), 30.0);
    assert_eq!(material.youngs_modulus.megapascals(), 33_000.0);

    // Typed quantities: 30 MPa is stored as 30e6 Pa, not as "30".
    assert_eq!(
        material.compressive_strength,
        Stress::from_megapascals(30.0)
    );
    assert_eq!(material.compressive_strength.pascals(), 30_000_000.0);

    assert!(model.validate().is_valid());
}

// -----------------------------------------------------------------------------
// Test 4 — cross section
// -----------------------------------------------------------------------------

#[test]
fn test_4_rectangular_cross_section_geometry() {
    let mut model = EngineeringModel::new();

    let id = model
        .add_cross_section(CrossSection::rectangular("400x400", mm(400.0), mm(400.0)))
        .expect("section");
    let section = model.cross_section(id).expect("stored section");

    assert_eq!(section.name, "400x400");
    // 400 mm x 400 mm = 0.16 m², computed from the profile.
    assert!(close(section.area().square_meters(), 0.16));

    let (ix, iy) = section.moment_of_inertia();
    assert_eq!(ix, iy, "a square section is symmetric");
    assert!(close(
        ix.meters_to_the_fourth(),
        0.4 * 0.4_f64.powi(3) / 12.0
    ));
    assert_eq!(section.depth(), mm(400.0));
    assert_eq!(section.width(), mm(400.0));

    // Area and inertia are *derived* from the profile: changing the shape changes
    // them, because there is no second stored copy that could disagree.
    let stored = model.cross_sections.get_mut(&id).expect("mutable section");
    stored.profile = ProfileType::Rectangular {
        width: mm(400.0),
        depth: mm(600.0),
    };
    assert!(close(
        model
            .cross_section(id)
            .expect("section")
            .area()
            .square_meters(),
        0.24
    ));
}

// -----------------------------------------------------------------------------
// Test 5 — column
// -----------------------------------------------------------------------------

#[test]
fn test_5_column_links_levels_material_and_section() {
    let project = build_project();
    let element = project.model.element(project.column).expect("column");

    assert_eq!(element.category(), ElementCategory::Column);
    assert_eq!(element.name(), "C001");
    assert_eq!(element.id(), project.column);

    let column = element.as_column().expect("column variant");
    assert_eq!(column.base_level_id, project.level_1);
    assert_eq!(column.top_level_id, project.level_2);

    // References resolve through the model — ids, never embedded copies.
    let base = project
        .model
        .level(column.base_level_id)
        .expect("base level");
    let top = project.model.level(column.top_level_id).expect("top level");
    assert!(project.model.material(column.material_id).is_some());
    assert!(project
        .model
        .cross_section(column.cross_section_id)
        .is_some());

    let (bottom, top_elevation) = column.extent(base.elevation, top.elevation);
    assert!(close(bottom.meters(), 0.0));
    assert!(close(top_elevation.meters(), 3.2));
}

#[test]
fn test_5b_column_offsets_and_plan_rotation() {
    let project = build_project();
    let column_id = project.column;
    let element = project.model.element(column_id).expect("column");

    let rotated = element
        .as_column()
        .expect("column")
        .clone()
        .with_plan_rotation(Angle::from_degrees(30.0))
        .with_offsets(Length::from_millimeters(-50.0), Length::from_meters(0.1));

    // Rotation lives in the transform, the single place placement is stored.
    assert!(matches!(
        rotated.base.transform.rotation,
        Rotation3D::AroundZ(_)
    ));
    assert!(close(
        rotated.base.transform.plan_rotation().degrees(),
        30.0
    ));
    assert!(close(rotated.base_offset.meters(), -0.05));
    assert!(close(rotated.top_offset.meters(), 0.1));
    assert!(close(
        rotated
            .extent(Length::ZERO, Length::from_meters(3.2))
            .1
            .meters(),
        3.3
    ));
}

// -----------------------------------------------------------------------------
// Test 6 — beam
// -----------------------------------------------------------------------------

#[test]
fn test_6_beam_links_level_material_and_section() {
    let project = build_project();
    let element = project.model.element(project.beam).expect("beam");

    assert_eq!(element.category(), ElementCategory::Beam);
    assert_eq!(element.name(), "B001");

    let beam = element.as_beam().expect("beam variant");
    assert_eq!(beam.reference_level_id, project.level_2);
    assert_eq!(beam.start_point, Point3D::from_meters(0.0, 0.0, 3.2));
    assert_eq!(beam.end_point, Point3D::from_meters(6.0, 0.0, 3.2));
    assert!(project.model.cross_section(beam.cross_section_id).is_some());
    assert!(project.model.material(beam.material_id).is_some());

    // Length and direction are derived from the axis, never stored.
    assert!(close(beam.length().meters(), 6.0));
    assert!(beam.direction().is_some());
    assert!(!beam.is_degenerate());
    assert_eq!(beam.z_justification, Justification::Center);

    let raised = beam.clone().with_justification(Justification::Top);
    assert_eq!(raised.z_justification, Justification::Top);
}

// -----------------------------------------------------------------------------
// Test 7 — relationships
// -----------------------------------------------------------------------------

#[test]
fn test_7_relationships_reject_unknown_ids_in_strict_mode() {
    let foreign = EngineeringModel::new();
    let missing = foreign.project_id;

    let mut model = EngineeringModel::new();
    let level = model
        .add_level(Level::new("Level 1", Length::ZERO))
        .expect("level");
    let material = model
        .add_material(Material::concrete_c30())
        .expect("material");
    let section = model
        .add_cross_section(CrossSection::rectangular("400x400", mm(400.0), mm(400.0)))
        .expect("section");

    // Unknown base level.
    let base = BaseElement::new(ElementCategory::Column, "C001");
    let column_id = base.id();
    let column = StructuralColumn::new(base, missing, level, section, material);
    assert_eq!(
        model.add_element(Element::Column(column)),
        Err(ModelError::MissingReference {
            element_id: column_id,
            reference: ReferenceKind::Level,
            missing_id: missing,
        })
    );

    // Unknown material on a beam.
    let base = BaseElement::new(ElementCategory::Beam, "B001");
    let beam_id = base.id();
    let beam = StructuralBeam::new(
        base,
        level,
        Point3D::ORIGIN,
        Point3D::from_meters(1.0, 0.0, 0.0),
        section,
        missing,
    );
    assert_eq!(
        model.add_element(Element::Beam(beam)),
        Err(ModelError::MissingReference {
            element_id: beam_id,
            reference: ReferenceKind::Material,
            missing_id: missing,
        })
    );

    // Unknown cross section.
    let base = BaseElement::new(ElementCategory::Column, "C002");
    let second_column_id = base.id();
    let second_column = StructuralColumn::new(base, level, level, missing, material);
    assert_eq!(
        model.add_element(Element::Column(second_column)),
        Err(ModelError::MissingReference {
            element_id: second_column_id,
            reference: ReferenceKind::CrossSection,
            missing_id: missing,
        })
    );

    assert_eq!(model.element_count(), 0, "nothing invalid was stored");
    assert!(model.validate().is_valid());

    // The same element is accepted while the model is permissive, and the problem is
    // reported by validation instead.
    let mut permissive = EngineeringModel::new().with_validation_mode(ValidationMode::Permissive);
    let base = BaseElement::new(ElementCategory::Column, "C003");
    let stray_id = base.id();
    let stray = StructuralColumn::new(base, missing, missing, missing, missing);
    permissive
        .add_element(Element::Column(stray))
        .expect("permissive model accepts it");

    let report = permissive.validate();
    assert!(!report.is_valid());
    assert!(report.contains(ValidationCode::MissingReference));
    assert_eq!(
        report
            .issues()
            .iter()
            .filter(|issue| issue.element_id == Some(stray_id))
            .count(),
        4,
        "levels, section and material are all reported"
    );
}

#[test]
fn test_7b_ids_must_be_free_across_the_whole_model() {
    let mut model = EngineeringModel::new();
    let shared = model.project_id;

    model
        .add_level(Level::with_id(shared, "Level 1", Length::ZERO))
        .expect("level");

    let clash = Material::with_id(
        shared,
        "Clashing material",
        MaterialType::Concrete,
        MassDensity::ZERO,
        Stress::ZERO,
        Stress::ZERO,
    );
    assert_eq!(
        model.add_material(clash),
        Err(ModelError::DuplicateId { id: shared })
    );
    assert_eq!(model.materials.len(), 0);
    assert_eq!(model.levels.len(), 1);
}

// -----------------------------------------------------------------------------
// Test 8 — serialization round trip
// -----------------------------------------------------------------------------

#[test]
fn test_8_serialization_round_trip() {
    let project = build_project();

    let json = project.model.to_json().expect("serialize");
    assert!(json.contains("\"schema_version\": 1"));

    let restored = EngineeringModel::from_json(&json).expect("deserialize");

    assert_eq!(restored, project.model, "nothing was lost or changed");
    assert_eq!(restored.summary(), project.model.summary());
    assert!(restored.validate().is_valid());

    // Relationships survived the round trip.
    let column = restored
        .element(project.column)
        .expect("column")
        .as_column()
        .expect("column variant")
        .clone();
    assert_eq!(column.base_level_id, project.level_1);
    assert_eq!(column.top_level_id, project.level_2);
    assert_eq!(column.material_id, project.material);
    assert_eq!(column.cross_section_id, project.section);

    // Dimensions survived in internal units: 400 mm is still 0.4 m.
    let section = restored.cross_section(project.section).expect("section");
    assert!(close(section.width().meters(), 0.4));
    assert!(close(section.area().square_meters(), 0.16));
}

// -----------------------------------------------------------------------------
// Acceptance scenario — create, validate, serialize, deserialize, validate
// -----------------------------------------------------------------------------

#[test]
fn acceptance_scenario_survives_the_full_lifecycle() {
    let project = build_project();
    let mut model = project.model;

    let summary = model.summary();
    assert_eq!(summary.schema_version, MODEL_SCHEMA_VERSION);
    assert_eq!(summary.levels, 2);
    assert_eq!(summary.grids, 4);
    assert_eq!(summary.materials, 1);
    assert_eq!(summary.cross_sections, 1);
    assert_eq!(summary.elements, 2);
    assert_eq!(summary.revision, 10);

    assert!(model.validate().is_valid(), "fresh model is consistent");

    let json = model.to_json().expect("serialize");
    model = EngineeringModel::from_json(&json).expect("deserialize");
    assert!(model.validate().is_valid(), "loaded model is consistent");

    let grid_names: Vec<&str> = model
        .grids
        .values()
        .map(|grid| grid.name.as_str())
        .collect();
    assert!(grid_names.contains(&"A"));
    assert!(grid_names.contains(&"B"));
    assert!(grid_names.contains(&"1"));
    assert!(grid_names.contains(&"2"));

    let element_names: Vec<&str> = model
        .elements
        .values()
        .map(|element| element.name())
        .collect();
    assert!(element_names.contains(&"C001"));
    assert!(element_names.contains(&"B001"));

    let beam = model
        .element(project.beam)
        .expect("beam")
        .as_beam()
        .expect("beam variant");
    assert!(close(beam.length().meters(), 6.0));
}

// -----------------------------------------------------------------------------
// Invariants and reporting
// -----------------------------------------------------------------------------

#[test]
fn validation_reports_physical_problems_without_repairing_them() {
    let mut model = EngineeringModel::new();
    let level = model
        .add_level(Level::new("Level 1", Length::ZERO))
        .expect("level");
    let bad_material = model
        .add_material(Material::new(
            "Bad material",
            MaterialType::Other,
            MassDensity::from_kilograms_per_cubic_meter(-1.0),
            Stress::from_megapascals(-5.0),
            Stress::ZERO,
        ))
        .expect("material");

    let outline = PlanBoundary::rectangle(
        Point3D::ORIGIN,
        Length::from_meters(2.0),
        Length::from_meters(2.0),
    );
    let slab = StructuralSlab::new(
        BaseElement::new(ElementCategory::Slab, "SL001"),
        level,
        Length::ZERO, // zero thickness
        bad_material,
        outline,
    );
    model
        .add_element(Element::Slab(slab))
        .expect("slab is accepted, its problems are reported");

    let report = model.validate();
    assert!(!report.is_valid());
    assert!(report.contains(ValidationCode::NegativeValue));
    assert!(report.contains(ValidationCode::InvalidDimension));
    assert!(report
        .issues()
        .iter()
        .all(|issue| !issue.message.is_empty()));
    assert_eq!(model.revision, 3, "validation never mutates the model");
}

#[test]
fn validation_reports_a_column_that_does_not_extend_upwards() {
    let mut model = EngineeringModel::new();
    let lower = model
        .add_level(Level::new("Level 1", Length::from_meters(0.0)))
        .expect("level 1");
    let upper = model
        .add_level(Level::new("Level 2", Length::from_meters(3.2)))
        .expect("level 2");
    let material = model
        .add_material(Material::concrete_c30())
        .expect("material");
    let section = model
        .add_cross_section(CrossSection::rectangular("400x400", mm(400.0), mm(400.0)))
        .expect("section");

    // Base on the upper level, top on the lower one.
    let column = StructuralColumn::new(
        BaseElement::new(ElementCategory::Column, "C001"),
        upper,
        lower,
        section,
        material,
    );
    model.add_element(Element::Column(column)).expect("column");

    let report = model.validate();
    assert!(!report.is_valid());
    assert!(report.contains(ValidationCode::InvalidLevelOrder));
}

#[test]
fn validation_reports_an_inconsistent_i_section() {
    let mut model = EngineeringModel::new();
    let profile = IBeamProfile::new(mm(500.0), mm(200.0), mm(10.0), mm(300.0));
    model
        .add_cross_section(CrossSection::new("Broken I", ProfileType::IBeam(profile)))
        .expect("section");

    let report = model.validate();
    assert!(!report.is_valid());
    assert!(report.contains(ValidationCode::InvalidCrossSection));
}

#[test]
fn i_section_properties_are_derived_from_the_parameters() {
    let profile = ProfileType::IBeam(IBeamProfile::new(mm(300.0), mm(150.0), mm(7.1), mm(10.7)));

    let area = profile.area().square_meters();
    let expected = 2.0 * 0.15 * 0.0107 + (0.3 - 2.0 * 0.0107) * 0.0071;
    assert!(close(area, expected), "area {area} != {expected}");

    let (ix, iy) = profile.moment_of_inertia();
    assert!(ix > iy, "the strong axis carries the larger second moment");
    assert_eq!(profile.depth(), mm(300.0));
    assert_eq!(profile.width(), mm(150.0));
}

#[test]
fn boundaries_must_have_at_least_three_vertices_and_be_horizontal() {
    assert!(PlanBoundary::new(vec![Point3D::ORIGIN, Point3D::ORIGIN]).is_err());

    let tilted = PlanBoundary::new(vec![
        Point3D::from_meters(0.0, 0.0, 0.0),
        Point3D::from_meters(1.0, 0.0, 0.5),
        Point3D::from_meters(0.0, 1.0, 0.0),
    ])
    .expect("three vertices");
    assert!(!tilted.is_horizontal());

    let flat = PlanBoundary::rectangle(
        Point3D::from_meters(1.0, 2.0, 3.0),
        Length::from_meters(2.0),
        Length::from_meters(1.0),
    );
    assert!(flat.is_horizontal());
    assert_eq!(flat.vertex_count(), 4);
}

#[test]
fn every_physical_element_kind_is_representable_and_valid() {
    let mut model = EngineeringModel::new();
    let level = model
        .add_level(Level::new("Level 1", Length::from_meters(0.0)))
        .expect("level 1");
    let upper = model
        .add_level(Level::new("Level 2", Length::from_meters(3.2)))
        .expect("level 2");
    let material = model
        .add_material(Material::concrete_c30())
        .expect("material");

    let outline = PlanBoundary::rectangle(
        Point3D::from_meters(0.0, 0.0, 3.2),
        Length::from_meters(6.0),
        Length::from_meters(5.0),
    );
    let slab = model
        .add_element(Element::Slab(StructuralSlab::new(
            BaseElement::new(ElementCategory::Slab, "SL001"),
            upper,
            mm(200.0),
            material,
            outline,
        )))
        .expect("slab");
    assert!(close(
        model
            .element(slab)
            .expect("slab")
            .as_slab()
            .expect("slab variant")
            .boundary_area()
            .square_meters(),
        30.0
    ));

    let wall = model
        .add_element(Element::Wall(StructuralWall::new(
            BaseElement::new(ElementCategory::Wall, "W001"),
            level,
            upper,
            Point3D::from_meters(0.0, 0.0, 0.0),
            Point3D::from_meters(6.0, 0.0, 0.0),
            mm(250.0),
            material,
        )))
        .expect("wall");
    assert!(close(
        model
            .element(wall)
            .expect("wall")
            .as_wall()
            .expect("wall variant")
            .length()
            .meters(),
        6.0
    ));

    let footprint = PlanBoundary::rectangle(
        Point3D::from_meters(0.0, 0.0, 0.0),
        Length::from_meters(2.0),
        Length::from_meters(2.0),
    );
    let foundation = model
        .add_element(Element::Foundation(Foundation::new(
            BaseElement::new(ElementCategory::Foundation, "F001"),
            level,
            mm(600.0),
            material,
            FoundationType::Isolated,
            footprint,
        )))
        .expect("foundation");
    assert_eq!(
        model
            .element(foundation)
            .expect("foundation")
            .as_foundation()
            .expect("foundation variant")
            .foundation_type,
        FoundationType::Isolated
    );

    assert!(model.validate().is_valid());
    assert_eq!(model.summary().elements, 3);
}

// -----------------------------------------------------------------------------
// Identity, storage and the command extension point
// -----------------------------------------------------------------------------

#[test]
fn identity_is_stable_and_independent_of_storage_order() {
    let project = build_project();
    let column_id = project.column;

    let mut model = project.model;
    let column = model.elements.remove(&column_id).expect("column removed");
    assert!(model.element(column_id).is_none());

    // Re-inserting under its own id keeps the identity; the element itself never
    // changed, so references still point at it.
    model.add_element(column).expect("column re-inserted");
    let restored = model.element(column_id).expect("column is back");
    assert_eq!(restored.id(), column_id);
    assert_eq!(restored.name(), "C001");
    assert!(model.validate().is_valid());
}

#[test]
fn revision_tracks_accepted_mutations_only() {
    let mut model = EngineeringModel::new();
    assert_eq!(model.revision, 0);

    model
        .add_level(Level::new("Level 1", Length::ZERO))
        .expect("level");
    assert_eq!(model.revision, 1);

    let foreign = EngineeringModel::new();
    let stray = Element::Column(StructuralColumn::new(
        BaseElement::new(ElementCategory::Column, "C001"),
        foreign.project_id,
        foreign.project_id,
        foreign.project_id,
        foreign.project_id,
    ));
    assert!(model.add_element(stray).is_err());
    assert_eq!(model.revision, 1, "a rejected mutation changes nothing");
}

#[test]
fn command_layer_applies_through_the_model() {
    let mut model = EngineeringModel::new();
    let command = AddLevelCommand::new("Roof", Length::from_meters(6.4));
    assert_eq!(command.name(), "add_level");

    let id = command.apply(&mut model).expect("command applied");
    let level = model.level(id).expect("level created by the command");
    assert_eq!(level.name, "Roof");
    assert_eq!(level.elevation_meters(), 6.4);
    assert_eq!(model.revision, 1);
}
