//! Tests for what the kernel exposes to Flutter, and for the unit rule the whole
//! model relies on.

use geometry_kernel_rs::api;
use geometry_kernel_rs::elements::{BaseElement, Element, ElementCategory, StructuralColumn};
use geometry_kernel_rs::model::{CrossSection, EngineeringModel, Level, MODEL_SCHEMA_VERSION};
use geometry_kernel_rs::units::Length;

#[test]
fn m0_kernel_status_proof_of_concept_still_works() {
    assert_eq!(
        api::get_kernel_status(),
        "Engineering Geometry Kernel (Rust) is connected successfully!"
    );
}

#[test]
fn create_empty_model_returns_a_flat_summary() {
    let summary = api::create_empty_model();

    assert_eq!(summary.schema_version, MODEL_SCHEMA_VERSION);
    assert_eq!(summary.revision, 0);
    assert_eq!(summary.total_entities(), 0);
    assert!(summary.is_empty());
    assert_eq!(summary.project_id.len(), 36);
}

#[test]
fn create_empty_model_is_independent_of_any_stored_model() {
    // The API is stateless: two calls describe two different, empty projects.
    let first = api::create_empty_model();
    let second = api::create_empty_model();

    assert_ne!(first.project_id, second.project_id);
    assert_eq!(first.total_entities(), 0);
    assert_eq!(second.total_entities(), 0);
}

#[test]
fn create_workspace_snapshot_contains_real_model_geometry() {
    let snapshot = api::create_workspace_snapshot(
        "Snapshot Test".to_string(),
        "مبنى إنشائي".to_string(),
        1200.0,
    );

    assert_eq!(snapshot.project_name, "Snapshot Test");
    assert_eq!(snapshot.project_type, "مبنى إنشائي");
    assert_eq!(snapshot.land_area_m2, 1200.0);
    assert_eq!(snapshot.levels.len(), 2);
    assert_eq!(snapshot.grids.len(), 4);
    assert_eq!(snapshot.elements.len(), 9);
    assert_eq!(
        snapshot
            .elements
            .iter()
            .filter(|element| element.category == "column")
            .count(),
        4
    );
    assert_eq!(
        snapshot
            .elements
            .iter()
            .filter(|element| element.category == "beam")
            .count(),
        4
    );
    let slab = snapshot
        .elements
        .iter()
        .find(|element| element.category == "slab")
        .expect("starter slab is present");
    assert_eq!(slab.boundary.len(), 4);
    assert_eq!(slab.boundary[0].z, 4.0);
}

#[test]
fn four_hundred_millimetres_is_stored_as_zero_point_four_metres() {
    let section = CrossSection::rectangular(
        "400x400",
        Length::from_millimeters(400.0),
        Length::from_millimeters(400.0),
    );

    // Never 400: the kernel stores internal SI units.
    assert_eq!(section.width(), Length::from_meters(0.4));
    assert_eq!(section.width().value(), 0.4);
    assert_eq!(section.depth().millimeters(), 400.0);
}

#[test]
fn a_model_can_still_be_built_entirely_from_the_public_api() {
    let mut model = EngineeringModel::new();
    let level = model
        .add_level(Level::new("Level 1", Length::from_meters(3.2)))
        .expect("level");
    let column = model
        .add_element(Element::Column(StructuralColumn::new(
            BaseElement::new(ElementCategory::Column, "C001"),
            // A missing reference is refused, so this call must fail.
            uuid_of(&model),
            level,
            uuid_of(&model),
            uuid_of(&model),
        )))
        .is_err();
    assert!(column);
    assert_eq!(model.element_count(), 0);
    assert!(model.validate().is_valid());
}

/// An id that is deliberately not used by the model.
fn uuid_of(model: &EngineeringModel) -> uuid::Uuid {
    let mut other = EngineeringModel::new();
    while model.contains_id(other.project_id) {
        other = EngineeringModel::new();
    }
    other.project_id
}
