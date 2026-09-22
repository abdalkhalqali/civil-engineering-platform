//! Tests for what the kernel exposes to the outside world, and for the unit rule the
//! whole model relies on.

use geometry_kernel_rs::api;
use geometry_kernel_rs::elements::{BaseElement, Element, ElementCategory, StructuralColumn};
use geometry_kernel_rs::model::{CrossSection, EngineeringModel, Level, MODEL_SCHEMA_VERSION};
use geometry_kernel_rs::session::CommandRequest;
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
    let first = api::create_empty_model();
    let second = api::create_empty_model();

    assert_ne!(first.project_id, second.project_id);
    assert_eq!(first.total_entities(), 0);
    assert_eq!(second.total_entities(), 0);
}

#[test]
fn create_project_reports_the_format_it_writes() {
    let summary = api::create_project("Project Summary Test".to_string());

    assert_eq!(summary.name, "Project Summary Test");
    assert_eq!(summary.format_version, 1);
    assert_eq!(summary.model_schema_version, MODEL_SCHEMA_VERSION);
    assert_eq!(summary.revision, 0);
}

#[test]
fn a_session_opens_with_a_modelling_environment() {
    let opened = api::open_session("مشروع".to_string(), "مبنى إنشائي".to_string(), 900.0);
    assert!(opened.ok, "{}", opened.message);
    assert_eq!(opened.session_id.len(), 36);

    let state = api::session_state(opened.session_id.clone());
    assert!(state.contains("Level 1"));
    assert!(state.contains("Level 2"));
    assert_eq!(extract_ids(&state, "levels").len(), 2);
    assert_eq!(extract_ids(&state, "grids").len(), 4);
    assert!(extract_elements(&state).contains("[]"), "no elements yet");
}

#[test]
fn a_command_crosses_the_boundary_as_a_flat_request() {
    let opened = api::open_session("أوامر".to_string(), "مبنى إنشائي".to_string(), 500.0);
    let session = opened.session_id.clone();

    // Read the two level ids the session created.
    let state = api::session_state(session.clone());
    let ground = extract_first_id(&state, "levels");
    let top = extract_last_id(&state, "levels");
    assert_ne!(ground, top);

    let result = api::execute_command(
        session.clone(),
        CommandRequest {
            command: "create_column".to_string(),
            x_m: 4.0,
            y_m: 3.0,
            base_level_id: ground,
            top_level_id: top,
            ..Default::default()
        },
    );

    assert!(result.ok, "{}", result.message);
    assert_eq!(result.changed_ids.len(), 1);
    assert!(result.revision > 0);
    assert!(!result.label.is_empty());

    let state_after = api::session_state(session.clone());
    assert!(state_after.contains("column"));

    // The renderer receives derived data carrying the element id, not the model.
    let render = api::render_data(session.clone());
    assert!(render.contains("civilx.render"));
    assert!(render.contains(&result.changed_ids[0]));
    assert!(!render.contains("materials"));
}

#[test]
fn undo_and_redo_cross_the_boundary_and_change_the_model() {
    let opened = api::open_session("تراجع".to_string(), "مبنى إنشائي".to_string(), 400.0);
    let session = opened.session_id.clone();
    let state = api::session_state(session.clone());
    let ground = extract_first_id(&state, "levels");
    let top = extract_last_id(&state, "levels");

    assert!(api::execute_command(
        session.clone(),
        CommandRequest {
            command: "create_column".to_string(),
            x_m: 0.0,
            y_m: 0.0,
            base_level_id: ground.clone(),
            top_level_id: top.clone(),
            ..Default::default()
        },
    )
    .ok);
    let with_column = api::session_state(session.clone());
    assert_eq!(extract_ids(&with_column, "elements").len(), 1);

    let undone = api::undo_command(session.clone());
    assert!(undone.ok, "{}", undone.message);
    let after_undo = api::session_state(session.clone());
    assert_eq!(
        extract_ids(&after_undo, "elements").len(),
        0,
        "undo restored the model"
    );

    let redone = api::redo_command(session.clone());
    assert!(redone.ok, "{}", redone.message);
    let after_redo = api::session_state(session.clone());
    assert_eq!(
        extract_elements(&after_redo),
        extract_elements(&with_column),
        "redo rebuilt the same element"
    );
}

#[test]
fn a_session_is_saved_and_reloaded_through_the_boundary() {
    let opened = api::open_session("حفظ".to_string(), "مبنى إنشائي".to_string(), 300.0);
    let session = opened.session_id.clone();
    let state = api::session_state(session.clone());
    let ground = extract_first_id(&state, "levels");
    let top = extract_last_id(&state, "levels");
    assert!(api::execute_command(
        session.clone(),
        CommandRequest {
            command: "create_column".to_string(),
            x_m: 2.0,
            y_m: 1.0,
            base_level_id: ground,
            top_level_id: top,
            ..Default::default()
        },
    )
    .ok);

    let bytes = api::save_session(session.clone());
    assert!(!bytes.is_empty());
    let before = api::session_state(session.clone());

    let reopened = api::load_session(bytes);
    assert!(reopened.ok, "{}", reopened.message);
    // The session id *is* the project identity, so reopening the same file keeps it.
    assert_eq!(reopened.session_id, opened.session_id);
    assert_eq!(
        extract_elements(&reopened.state_json),
        extract_elements(&before),
        "the reopened session holds the same model"
    );
}

#[test]
fn an_unknown_session_is_reported_instead_of_panicking() {
    let result = api::undo_command("not-a-uuid".to_string());
    assert!(!result.ok);
    assert!(!result.message.is_empty());
    assert!(api::session_state("not-a-uuid".to_string()).is_empty());
    assert!(api::render_data("not-a-uuid".to_string()).is_empty());
    assert!(!api::close_session("not-a-uuid".to_string()));
}

#[test]
fn a_snapped_point_is_reported_with_the_relationship_it_found() {
    let opened = api::open_session("التقاط".to_string(), "مبنى إنشائي".to_string(), 100.0);
    let snap = api::snap_point(opened.session_id.clone(), 0.02, 0.02, 0.0, 0.25);
    assert!(snap.snapped);
    assert_eq!(snap.kind, "grid_intersection");
    assert_eq!(snap.label, "A-1");
    assert!(!snap.candidates.is_empty());

    let free = api::snap_point(opened.session_id, 50.0, 50.0, 0.0, 0.1);
    assert!(!free.snapped);
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

/// Text of the JSON array stored under `key`.
///
/// Bracket matching keeps a following array from leaking into the result, which is
/// what makes the assertions on this text meaningful.
fn array_text(json: &str, key: &str) -> String {
    let needle = format!("\"{key}\":[");
    let Some(found) = json.find(&needle) else {
        return String::new();
    };
    let rest = &json[found + needle.len() - 1..];
    let mut depth = 0usize;
    for (index, character) in rest.char_indices() {
        match character {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return rest[..=index].to_string();
                }
            }
            _ => {}
        }
    }
    String::new()
}

/// Every `"id"` value inside the array stored under `key`.
fn extract_ids(json: &str, key: &str) -> Vec<String> {
    let text = array_text(json, key);
    let marker = "\"id\":\"";
    let mut ids = Vec::new();
    let mut cursor = 0;
    while let Some(found) = text[cursor..].find(marker) {
        let begin = cursor + found + marker.len();
        let Some(end) = text[begin..].find('"') else {
            break;
        };
        ids.push(text[begin..begin + end].to_string());
        cursor = begin + end;
    }
    ids
}

fn extract_first_id(json: &str, key: &str) -> String {
    extract_ids(json, key).first().cloned().unwrap_or_default()
}

fn extract_last_id(json: &str, key: &str) -> String {
    extract_ids(json, key).last().cloned().unwrap_or_default()
}

/// The `elements` array of a state payload, used to compare two models.
fn extract_elements(json: &str) -> String {
    array_text(json, "elements")
}
