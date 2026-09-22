//! Acceptance tests of the command layer.
//!
//! Every test asserts on the **engineering model**, never on a viewport: if an
//! element moved on screen while `EngineeringModel` kept its old values, these tests
//! would fail.

use geometry_kernel_rs::commands::elements::ElementCommand;
use geometry_kernel_rs::elements::Element;
use geometry_kernel_rs::session::ModelSession;
use uuid::Uuid;

/// A session with a project name and the starter environment.
fn session() -> ModelSession {
    ModelSession::new("اختبار", "مبنى إنشائي", 1000.0)
}

/// The id of the lowest level.
fn ground_level(session: &ModelSession) -> Uuid {
    session
        .model()
        .levels
        .values()
        .min_by(|a, b| a.elevation.meters().partial_cmp(&b.elevation.meters()).unwrap())
        .expect("the starter project has levels")
        .id
}

/// The id of the highest level.
fn top_level(session: &ModelSession) -> Uuid {
    session
        .model()
        .levels
        .values()
        .max_by(|a, b| a.elevation.meters().partial_cmp(&b.elevation.meters()).unwrap())
        .expect("the starter project has levels")
        .id
}

/// Creates one column at the origin of the starter environment.
fn create_column(session: &mut ModelSession) -> Uuid {
    let base = ground_level(session);
    let top = top_level(session);
    let result = session.execute(ElementCommand::CreateColumn {
        name: String::new(),
        x_m: 0.0,
        y_m: 0.0,
        base_level_id: base,
        top_level_id: top,
        width_m: 0.4,
        depth_m: 0.4,
    });
    assert!(result.ok, "the column must be created: {}", result.message);
    result
        .changed_ids
        .first()
        .and_then(|id| Uuid::parse_str(id).ok())
        .expect("a created column reports its identity")
}

#[test]
fn a_new_session_is_a_real_engineering_environment() {
    let session = session();

    assert_eq!(session.model().levels.len(), 2, "two levels");
    assert_eq!(session.model().grids.len(), 4, "four grid lines");
    assert_eq!(session.model().materials.len(), 1, "one material");
    assert_eq!(session.model().cross_sections.len(), 1, "one column section");
    assert_eq!(session.model().elements.len(), 0, "no elements yet");
    assert!(session.model().validate().is_valid());
}

#[test]
fn t1_a_column_records_the_two_levels_it_spans() {
    let mut session = session();
    let base = ground_level(&session);
    let top = top_level(&session);
    let id = create_column(&mut session);

    let column = session
        .model()
        .element(id)
        .and_then(Element::as_column)
        .expect("the element is a column");

    assert_eq!(column.base_level_id, base);
    assert_eq!(column.top_level_id, top);

    // The relationship is in the serialized model, not only in memory.
    let json = session.model().to_json().unwrap();
    assert!(json.contains(&top.to_string()));
}

#[test]
fn t2_the_top_grip_changes_the_top_level_in_the_model() {
    let mut session = session();
    let id = create_column(&mut session);
    let top = top_level(&session);

    // A third level, as a user would create it.
    let result = session.execute(ElementCommand::AddLevel {
        name: "Level 3".to_string(),
        elevation_m: 6.4,
    });
    assert!(result.ok);
    let level_3 = Uuid::parse_str(&result.changed_ids[0]).unwrap();

    let revision_before = session.revision();
    let result = session.execute(ElementCommand::SetElementTopLevel {
        id,
        top_level_id: level_3,
    });
    assert!(result.ok, "{}", result.message);

    let column = session.model().element(id).and_then(Element::as_column).unwrap();
    assert_eq!(
        column.top_level_id, level_3,
        "the top level reference must change, not just the drawn box"
    );
    assert_ne!(column.top_level_id, top);
    assert_eq!(session.revision(), revision_before + 1, "one accepted mutation");
    assert_eq!(result.changed_ids, vec![id.to_string()]);
}

#[test]
fn t3_changing_a_section_changes_the_model_and_its_derived_area() {
    let mut session = session();
    let id = create_column(&mut session);

    let area_before = current_column_area(&session, id);
    let result = session.execute(ElementCommand::SetElementSection {
        id,
        width_mm: 500.0,
        depth_mm: 500.0,
    });
    assert!(result.ok, "{}", result.message);

    let column = session.model().element(id).and_then(Element::as_column).unwrap();
    let section = session
        .model()
        .cross_section(column.cross_section_id)
        .expect("the new section exists");
    assert_eq!(section.width().millimeters(), 500.0);
    assert_eq!(section.depth().millimeters(), 500.0);

    // The area is a function of the profile, so it follows without being stored.
    let area_after = current_column_area(&session, id);
    assert!((area_after - 0.25).abs() < 1e-12);
    assert!(area_after > area_before);
}

#[test]
fn t4_a_reference_that_does_not_resolve_is_refused() {
    use geometry_kernel_rs::elements::{BaseElement, ElementCategory, StructuralColumn};
    use geometry_kernel_rs::model::EngineeringModel;

    let mut model = EngineeringModel::new();
    let level = model
        .add_level(geometry_kernel_rs::model::Level::new(
            "Level 1",
            geometry_kernel_rs::units::Length::from_meters(0.0),
        ))
        .unwrap();
    let revision = model.revision;

    let result = model.add_element(Element::Column(StructuralColumn::new(
        BaseElement::new(ElementCategory::Column, "C001"),
        level,
        level,
        Uuid::new_v4(), // a section that is not in the model
        Uuid::new_v4(), // a material that is not in the model
    )));

    assert!(result.is_err(), "a dangling reference must be refused");
    assert_eq!(model.elements.len(), 0, "nothing was stored");
    assert_eq!(model.revision, revision, "a refused mutation keeps the revision");
}

#[test]
fn t5_move_deletion_and_undo_restore_the_model_exactly() {
    let mut session = session();
    let id = create_column(&mut session);
    let before_move = session.model().to_json().unwrap();

    assert!(session
        .execute(ElementCommand::MoveElement {
            id,
            dx_m: 4.0,
            dy_m: 1.5,
            dz_m: 0.0,
        })
        .ok);
    let moved = session.model().element(id).unwrap().base().transform;
    assert_eq!(moved.translation.x.meters(), 4.0);
    assert_eq!(moved.translation.y.meters(), 1.5);

    assert!(session.undo().ok, "undo of a move");
    assert!(
        same_content(&session.model().to_json().unwrap(), &before_move),
        "undo restores the engineering model exactly"
    );

    assert!(session.redo().ok, "redo of a move");
    let moved_again = session.model().element(id).unwrap().base().transform;
    assert_eq!(moved_again.translation.x.meters(), 4.0);

    // Deletion removes the element from the model, and undo puts it back.
    let before_delete = session.model().to_json().unwrap();
    assert!(session.execute(ElementCommand::DeleteElement { id }).ok);
    assert!(session.model().element(id).is_none(), "deleted from the model");
    assert!(session.undo().ok);
    assert!(session.model().element(id).is_some(), "restored to the model");
    assert!(same_content(
        &session.model().to_json().unwrap(),
        &before_delete
    ));
}

#[test]
fn t6_undo_then_redo_is_byte_identical() {
    let mut session = session();
    let base = ground_level(&session);
    let top = top_level(&session);
    assert!(session
        .execute(ElementCommand::CreateColumn {
            name: "C010".to_string(),
            x_m: 4.0,
            y_m: 3.0,
            base_level_id: base,
            top_level_id: top,
            width_m: 0.4,
            depth_m: 0.4,
        })
        .ok);
    assert!(session
        .execute(ElementCommand::CreateBeam {
            name: "B010".to_string(),
            start_x_m: 0.0,
            start_y_m: 0.0,
            end_x_m: 4.0,
            end_y_m: 3.0,
            level_id: top,
            width_m: 0.3,
            depth_m: 0.5,
        })
        .ok);

    let after_commands = session.model().to_json().unwrap();
    assert!(session.undo().ok);
    assert!(session.undo().ok);
    assert_eq!(
        session.model().element_count(),
        0,
        "both elements left the model"
    );
    assert!(session.redo().ok);
    assert!(session.redo().ok);
    assert!(
        same_content(&session.model().to_json().unwrap(), &after_commands),
        "redo rebuilds the same elements, identities included"
    );
}

/// Two serialized models hold the same engineering content.
///
/// The revision counter is excluded on purpose: undo and redo are themselves
/// accepted mutations, so the revision legitimately moves forward while the content
/// goes back or forward with it. Comparing it here would test the counter, not the
/// model.
fn same_content(left: &str, right: &str) -> bool {
    without_revision(left) == without_revision(right)
}

fn without_revision(json: &str) -> String {
    json.lines()
        .filter(|line| !line.trim_start().starts_with("\"revision\""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn t7_save_and_reopen_keeps_the_model() {
    let mut session = session();
    let id = create_column(&mut session);
    let top = top_level(&session);
    assert!(session.execute(ElementCommand::AddLevel {
        name: "Level 3".to_string(),
        elevation_m: 6.4,
    })
    .ok);
    let level_3 = session
        .model()
        .levels
        .values()
        .find(|level| level.name == "Level 3")
        .unwrap()
        .id;
    assert!(session
        .execute(ElementCommand::SetElementTopLevel {
            id,
            top_level_id: level_3,
        })
        .ok);
    assert!(top != level_3);

    let bytes = session.save().expect("the project saves");
    assert!(!bytes.is_empty());

    let mut reopened = ModelSession::new("", "", 0.0);
    reopened.load(&bytes).expect("the project loads");
    assert_eq!(reopened.project_id(), session.project_id());
    assert_eq!(
        reopened.model().to_json().unwrap(),
        session.model().to_json().unwrap(),
        "the reopened model is the same model"
    );
    assert_eq!(
        reopened.model().element(id).and_then(Element::as_column).unwrap().top_level_id,
        level_3,
        "the relationship survived the file"
    );
    assert!(reopened.model().validate().is_valid());
}

#[test]
fn t8_reading_a_view_never_changes_the_model() {
    let mut session = session();
    let id = create_column(&mut session);

    let revision = session.revision();
    let json = session.model().to_json().unwrap();

    // Everything a viewport does, many times over.
    let first = session.render_data();
    let _ = session.snap(0.0, 0.0, 0.5, 0.0);
    let _ = session.snap(2.0, 2.0, 0.5, 3.2);
    let _ = session.element_details(id);
    let second = session.render_data();

    assert_eq!(session.revision(), revision, "reading is not a mutation");
    assert_eq!(session.model().to_json().unwrap(), json);
    assert_eq!(first, second, "render data is a pure function of the model");
}

#[test]
fn a_renderer_primitive_carries_the_element_it_came_from() {
    let mut session = session();
    let id = create_column(&mut session);

    let data = session.render_data();
    assert_eq!(data.members.len(), 1);
    assert_eq!(data.members[0].element_id, id.to_string());
    assert_eq!(data.members[0].category, "column");
    assert_eq!(data.revision, session.revision());

    // The model itself stores no render data and no mesh.
    let json = session.model().to_json().unwrap();
    assert!(!json.contains("members"), "the model owns no render payload");
    assert!(!json.contains("mesh"));
}

#[test]
fn snapping_prefers_a_grid_intersection_over_free_space() {
    let session = session();

    // Near the A-1 intersection (x = 0, y = 0).
    let snap = session.snap(0.05, -0.04, 0.3, 0.0);
    assert!(snap.snapped);
    assert_eq!(snap.kind, "grid_intersection");
    assert_eq!(snap.label, "A-1");

    // Far from everything: free placement, as a free hand expects.
    let free = session.snap(9.9, 9.9, 0.2, 0.0);
    assert!(!free.snapped);
    assert_eq!(free.kind, "free");
}

#[test]
fn a_rejected_command_leaves_the_model_untouched() {
    let mut session = session();
    let _ = create_column(&mut session);
    let json = session.model().to_json().unwrap();
    let revision = session.revision();

    // A top level below the base level is not an engineering possibility.
    let result = session.execute(ElementCommand::CreateColumn {
        name: String::new(),
        x_m: 0.0,
        y_m: 0.0,
        base_level_id: top_level(&session),
        top_level_id: ground_level(&session),
        width_m: 0.4,
        depth_m: 0.4,
    });

    assert!(!result.ok, "an inverted extent is refused");
    assert!(!result.message.is_empty(), "the refusal explains itself");
    assert_eq!(session.model().element_count(), 1, "nothing was added");
    assert_eq!(session.model().to_json().unwrap(), json, "the model is untouched");
    assert_eq!(session.revision(), revision, "a refused command keeps the revision");
}

fn current_column_area(session: &ModelSession, id: Uuid) -> f64 {
    let column = session.model().element(id).and_then(Element::as_column).unwrap();
    session
        .model()
        .cross_section(column.cross_section_id)
        .expect("the section exists")
        .area()
        .square_meters()
}
