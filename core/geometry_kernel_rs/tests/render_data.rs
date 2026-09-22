//! Tests of the render-data layer: it is a projection of the model, and only that.

use geometry_kernel_rs::commands::elements::ElementCommand;
use geometry_kernel_rs::render::RENDER_SCHEMA;
use geometry_kernel_rs::session::ModelSession;
use uuid::Uuid;

fn session() -> ModelSession {
    ModelSession::new("عرض", "مبنى إنشائي", 800.0)
}

fn lowest_level(session: &ModelSession) -> Uuid {
    session
        .model()
        .levels
        .values()
        .min_by(|a, b| a.elevation.meters().partial_cmp(&b.elevation.meters()).unwrap())
        .unwrap()
        .id
}

fn highest_level(session: &ModelSession) -> Uuid {
    session
        .model()
        .levels
        .values()
        .max_by(|a, b| a.elevation.meters().partial_cmp(&b.elevation.meters()).unwrap())
        .unwrap()
        .id
}

fn add_column(session: &mut ModelSession, x_m: f64, y_m: f64) -> Uuid {
    let base = lowest_level(session);
    let top = highest_level(session);
    let result = session.execute(ElementCommand::CreateColumn {
        name: String::new(),
        x_m,
        y_m,
        base_level_id: base,
        top_level_id: top,
        width_m: 0.4,
        depth_m: 0.4,
    });
    assert!(result.ok, "{}", result.message);
    Uuid::parse_str(&result.changed_ids[0]).unwrap()
}

#[test]
fn geometry_is_derived_from_the_levels_it_references() {
    let mut session = session();
    let id = add_column(&mut session, 0.0, 0.0);

    let bottom = session.render_data().members[0].start[2];
    let top = session.render_data().members[0].end[2];
    assert_eq!(bottom, 0.0, "the column starts on Level 1");
    assert_eq!(top, 3.2, "and ends on Level 2");

    // The top grip scenario: re-attaching the column to a higher level changes the
    // *reference*, and the derived geometry follows it.
    let result = session.execute(ElementCommand::AddLevel {
        name: "Level 3".to_string(),
        elevation_m: 6.4,
    });
    assert!(result.ok);
    let level_3 = Uuid::parse_str(&result.changed_ids[0]).unwrap();
    assert!(session
        .execute(ElementCommand::SetElementTopLevel {
            id,
            top_level_id: level_3,
        })
        .ok);

    let derived = session.render_data();
    assert_eq!(
        derived.members[0].end[2], 6.4,
        "the derived top follows the level elevation"
    );
    assert_eq!(derived.members[0].start[2], 0.0, "the base did not move");
}

#[test]
fn every_primitive_carries_its_element_id() {
    let mut session = session();
    let first = add_column(&mut session, 0.0, 0.0);
    let second = add_column(&mut session, 4.0, 3.0);
    let level = highest_level(&session);
    assert!(session
        .execute(ElementCommand::CreateBeam {
            name: String::new(),
            start_x_m: 0.0,
            start_y_m: 0.0,
            end_x_m: 4.0,
            end_y_m: 3.0,
            level_id: level,
            width_m: 0.3,
            depth_m: 0.5,
        })
        .ok);

    let data = session.render_data();
    assert_eq!(data.members.len(), 3);
    let ids: Vec<&str> = data.members.iter().map(|m| m.element_id.as_str()).collect();
    assert!(ids.contains(&first.to_string().as_str()));
    assert!(ids.contains(&second.to_string().as_str()));

    // Grids and levels are present as first-class render data, not as painted lines.
    assert_eq!(data.grids.len(), 4);
    assert_eq!(data.levels.len(), 2);
    assert_eq!(data.schema, RENDER_SCHEMA);
    assert_eq!(data.units, "m");
    assert!(data.bounds.max[0] >= 4.0);
}

#[test]
fn plates_carry_a_plan_outline_and_two_elevations() {
    let mut session = session();
    let level = highest_level(&session);
    let result = session.execute(ElementCommand::CreateSlab {
        name: String::new(),
        level_id: level,
        thickness_m: 0.2,
        points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0]],
    });
    assert!(result.ok, "{}", result.message);

    let data = session.render_data();
    assert_eq!(data.plates.len(), 1);
    let plate = &data.plates[0];
    assert_eq!(plate.category, "slab");
    assert_eq!(plate.points.len(), 4, "a closed outline, not a mesh");
    assert_eq!(plate.z_top, 3.2);
    assert!((plate.z_bottom - 3.0).abs() < 1e-12, "the slab has a thickness");
}

#[test]
fn a_wall_is_a_band_outline_with_a_thickness() {
    let mut session = session();
    let base = lowest_level(&session);
    let top = highest_level(&session);
    assert!(session
        .execute(ElementCommand::CreateWall {
            name: String::new(),
            start_x_m: 0.0,
            start_y_m: 0.0,
            end_x_m: 4.0,
            end_y_m: 0.0,
            base_level_id: base,
            top_level_id: top,
            thickness_m: 0.25,
        })
        .ok);

    let data = session.render_data();
    let plate = &data.plates[0];
    assert_eq!(plate.category, "wall");
    assert_eq!(plate.points.len(), 4);
    assert_eq!(plate.z_bottom, 0.0);
    assert_eq!(plate.z_top, 3.2);
    // The band is the axis expanded by half the thickness on both sides.
    assert!((plate.points[0][1] - 0.125).abs() < 1e-12);
    assert!((plate.points[2][1] + 0.125).abs() < 1e-12);
    assert_eq!(plate.points[0][0], 0.0);
    assert_eq!(plate.points[1][0], 4.0);
}

#[test]
fn the_model_never_stores_render_state() {
    let mut session = session();
    add_column(&mut session, 1.0, 1.0);

    let model_json = session.model().to_json().unwrap();
    assert!(!model_json.contains("members"));
    assert!(!model_json.contains("plates"));
    assert!(!model_json.contains(RENDER_SCHEMA));
    assert!(!model_json.contains("mesh"));
    assert!(!model_json.contains("camera"));

    // Render data is produced on demand and is not part of the file.
    let _ = session.render_data();
    let bytes = session.save().unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(!text.contains(RENDER_SCHEMA));
}
