//! Public Rust API of the geometry kernel, exposed to Flutter through
//! `flutter_rust_bridge`.
//!
//! Scope of this module: the bridge proof of concept (unchanged) plus the smallest
//! possible view of the engineering model.
//!
//! What deliberately does **not** cross the boundary in this step:
//! * the [`crate::model::EngineeringModel`] itself, and any element, level, grid,
//!   material or cross section — the model stays in Rust, which is the single
//!   source of truth;
//! * any engineering logic, which must never be re-implemented on the Flutter side.
//!
//! Flutter only receives flat [`ModelSummary`] counts. Full DTOs, queries and
//! commands over the bridge belong to a later step.

use crate::elements::{
    BaseElement, Element, ElementCategory, PlanBoundary, StructuralBeam, StructuralColumn,
    StructuralSlab,
};
use crate::math::{Point3D, Transform3D};
use crate::model::{CrossSection, EngineeringModel, Grid, Level, Material, ModelSummary};
use crate::project::{Project, ProjectSummary};
use crate::units::Length;
use std::sync::{Mutex, OnceLock};

static CURRENT_PROJECT: OnceLock<Mutex<Option<Project>>> = OnceLock::new();

fn current_project() -> &'static Mutex<Option<Project>> {
    CURRENT_PROJECT.get_or_init(|| Mutex::new(None))
}

/// Flat, render-oriented data derived from the engineering model.
///
/// This is a read-only projection for Flutter. The model and all mutations remain in
/// Rust; the renderer never becomes the source of truth.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSnapshot {
    pub project_id: String,
    pub project_name: String,
    pub project_type: String,
    pub land_area_m2: f64,
    pub revision: u64,
    pub levels: Vec<LevelSnapshot>,
    pub grids: Vec<GridSnapshot>,
    pub elements: Vec<ElementSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelSnapshot {
    pub id: String,
    pub name: String,
    pub elevation_m: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridSnapshot {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub offset_m: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointSnapshot {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementSnapshot {
    pub id: String,
    pub name: String,
    pub category: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub top_z: f64,
    pub width: f64,
    pub depth: f64,
    pub thickness: f64,
    pub start: PointSnapshot,
    pub end: PointSnapshot,
    pub boundary: Vec<PointSnapshot>,
}

/// Called once by `RustLib.init()` on the Dart side.
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities (logging / panic reporting).
    flutter_rust_bridge::setup_default_user_utils();
}

/// M0 proof of concept, kept working verbatim: returns a status string that Flutter
/// displays on screen.
#[flutter_rust_bridge::frb(sync)]
pub fn get_kernel_status() -> String {
    "Engineering Geometry Kernel (Rust) is connected successfully!".to_string()
}

/// Creates an empty engineering model in Rust and returns its summary.
///
/// The model is created (and dropped) inside the kernel: this call exists so the app
/// can prove that the model core is reachable through the bridge, not to move model
/// data into Flutter.
#[flutter_rust_bridge::frb(sync)]
pub fn create_empty_model() -> ModelSummary {
    EngineeringModel::new().summary()
}

/// Creates a new empty project with the given name.
///
/// The project contains an empty engineering model. This proves the project
/// persistence layer is reachable through the bridge.
#[flutter_rust_bridge::frb(sync)]
pub fn create_project(name: String) -> ProjectSummary {
    Project::new(name).summary()
}

/// Creates a real starter project and returns a read-only rendering snapshot.
///
/// The starter contains levels, grids, a concrete material, a reusable section,
/// four columns, four beams and one slab. It is intentionally created in the
/// engineering model so the viewport is derived from model data from the first frame.
#[flutter_rust_bridge::frb(sync)]
pub fn create_workspace_snapshot(
    name: String,
    project_type: String,
    land_area_m2: f64,
) -> WorkspaceSnapshot {
    let mut current = current_project()
        .lock()
        .expect("current project lock is not poisoned");
    *current = Some(starter_project(name, project_type, land_area_m2));
    workspace_snapshot(current.as_ref().expect("starter project was created"))
}

/// Adds a parametric column to the active project and returns a fresh derived snapshot.
///
/// The column is stored in the Rust engineering model and the Flutter viewport only
/// receives the resulting projection.
#[flutter_rust_bridge::frb(sync)]
pub fn add_column_to_workspace(x_m: f64, y_m: f64) -> WorkspaceSnapshot {
    let mut current = current_project()
        .lock()
        .expect("current project lock is not poisoned");
    let project = current
        .as_mut()
        .expect("create_workspace_snapshot must be called first");
    let model = &mut project.model;
    let ground_id = model
        .levels
        .values()
        .next()
        .expect("starter project has a ground level")
        .id;
    let upper_id = model
        .levels
        .values()
        .nth(1)
        .expect("starter project has an upper level")
        .id;
    let material_id = model
        .materials
        .values()
        .next()
        .expect("starter project has a material")
        .id;
    let cross_section_id = model
        .cross_sections
        .values()
        .next()
        .expect("starter project has a cross section")
        .id;
    let number = model
        .elements
        .values()
        .filter(|element| element.category() == ElementCategory::Column)
        .count()
        + 1;
    let name = format!("C-{number:02}");
    let base = BaseElement::new(ElementCategory::Column, name)
        .with_transform(Transform3D::at(Point3D::from_meters(x_m, y_m, 0.0)));
    model
        .add_element(
            StructuralColumn::new(base, ground_id, upper_id, cross_section_id, material_id).into(),
        )
        .expect("column references resolve");
    project.metadata.touch();
    workspace_snapshot(project)
}

/// Adds a parametric beam to the active project and returns a fresh derived snapshot.
#[flutter_rust_bridge::frb(sync)]
pub fn add_beam_to_workspace(
    start_x_m: f64,
    start_y_m: f64,
    end_x_m: f64,
    end_y_m: f64,
) -> WorkspaceSnapshot {
    let mut current = current_project()
        .lock()
        .expect("current project lock is not poisoned");
    let project = current
        .as_mut()
        .expect("create_workspace_snapshot must be called first");
    let model = &mut project.model;
    let reference_level_id = model
        .levels
        .values()
        .nth(1)
        .expect("starter project has an upper level")
        .id;
    let material_id = model
        .materials
        .values()
        .next()
        .expect("starter project has a material")
        .id;
    let cross_section_id = model
        .cross_sections
        .values()
        .next()
        .expect("starter project has a cross section")
        .id;
    let number = model
        .elements
        .values()
        .filter(|element| element.category() == ElementCategory::Beam)
        .count()
        + 1;
    let name = format!("B-{number:02}");
    let base = BaseElement::new(ElementCategory::Beam, name);
    model
        .add_element(
            StructuralBeam::new(
                base,
                reference_level_id,
                Point3D::from_meters(start_x_m, start_y_m, 4.0),
                Point3D::from_meters(end_x_m, end_y_m, 4.0),
                cross_section_id,
                material_id,
            )
            .into(),
        )
        .expect("beam references resolve");
    project.metadata.touch();
    workspace_snapshot(project)
}

fn starter_project(name: String, project_type: String, land_area_m2: f64) -> Project {
    let mut project = Project::new_with_site(name, project_type, land_area_m2);
    let model = &mut project.model;

    let ground = Level::new("Level 1", Length::from_meters(0.0));
    let ground_id = ground.id;
    let upper = Level::new("Level 2", Length::from_meters(4.0));
    let upper_id = upper.id;
    model
        .add_level(ground)
        .expect("starter ground level id is unique");
    model
        .add_level(upper)
        .expect("starter upper level id is unique");

    for (name, offset) in [("A", -3.2), ("B", 3.2)] {
        model
            .add_grid(Grid::along_y(name, Length::from_meters(offset)))
            .expect("starter grid id is unique");
    }
    for (name, offset) in [("1", -2.2), ("2", 2.2)] {
        model
            .add_grid(Grid::along_x(name, Length::from_meters(offset)))
            .expect("starter grid id is unique");
    }

    let material = Material::concrete_c30();
    let material_id = material.id;
    model
        .add_material(material)
        .expect("starter material id is unique");
    let cross_section = CrossSection::rectangular(
        "400x400",
        Length::from_meters(0.4),
        Length::from_meters(0.4),
    );
    let cross_section_id = cross_section.id;
    model
        .add_cross_section(cross_section)
        .expect("starter cross section id is unique");

    let corners = [
        ("C-01", -3.2, -2.2),
        ("C-02", 3.2, -2.2),
        ("C-03", -3.2, 2.2),
        ("C-04", 3.2, 2.2),
    ];
    for (name, x, y) in corners {
        let base = BaseElement::new(ElementCategory::Column, name)
            .with_transform(Transform3D::at(Point3D::from_meters(x, y, 0.0)));
        model
            .add_element(
                StructuralColumn::new(base, ground_id, upper_id, cross_section_id, material_id)
                    .into(),
            )
            .expect("starter column references resolve");
    }

    let beam_points = [
        (
            "B-01",
            Point3D::from_meters(-3.2, -2.2, 4.0),
            Point3D::from_meters(3.2, -2.2, 4.0),
        ),
        (
            "B-02",
            Point3D::from_meters(3.2, -2.2, 4.0),
            Point3D::from_meters(3.2, 2.2, 4.0),
        ),
        (
            "B-03",
            Point3D::from_meters(3.2, 2.2, 4.0),
            Point3D::from_meters(-3.2, 2.2, 4.0),
        ),
        (
            "B-04",
            Point3D::from_meters(-3.2, 2.2, 4.0),
            Point3D::from_meters(-3.2, -2.2, 4.0),
        ),
    ];
    for (name, start, end) in beam_points {
        let base = BaseElement::new(ElementCategory::Beam, name);
        model
            .add_element(
                StructuralBeam::new(base, upper_id, start, end, cross_section_id, material_id)
                    .into(),
            )
            .expect("starter beam references resolve");
    }

    let slab_base = BaseElement::new(ElementCategory::Slab, "SLAB-01");
    let slab_boundary = PlanBoundary::rectangle(
        Point3D::from_meters(-3.2, -2.2, 4.0),
        Length::from_meters(6.4),
        Length::from_meters(4.4),
    );
    model
        .add_element(
            StructuralSlab::new(
                slab_base,
                upper_id,
                Length::from_meters(0.2),
                material_id,
                slab_boundary,
            )
            .into(),
        )
        .expect("starter slab references resolve");

    project
}

fn workspace_snapshot(project: &Project) -> WorkspaceSnapshot {
    let model = &project.model;
    WorkspaceSnapshot {
        project_id: project.project_id().to_string(),
        project_name: project.metadata.name.clone(),
        project_type: project.metadata.project_type.clone(),
        land_area_m2: project.metadata.land_area_m2,
        revision: model.revision,
        levels: model
            .levels
            .values()
            .map(|level| LevelSnapshot {
                id: level.id.to_string(),
                name: level.name.clone(),
                elevation_m: level.elevation.meters(),
            })
            .collect(),
        grids: model
            .grids
            .values()
            .map(|grid| GridSnapshot {
                id: grid.id.to_string(),
                name: grid.name.clone(),
                direction: grid.direction.as_str().to_string(),
                offset_m: grid.offset.meters(),
            })
            .collect(),
        elements: model
            .elements
            .values()
            .map(|element| element_snapshot(element, model))
            .collect(),
    }
}

fn point_snapshot(point: Point3D) -> PointSnapshot {
    let (x, y, z) = point.coordinates_meters();
    PointSnapshot { x, y, z }
}

fn element_snapshot(element: &Element, model: &EngineeringModel) -> ElementSnapshot {
    let position = element.base().transform.position();
    let (x, y, z) = position.coordinates_meters();
    let mut snapshot = ElementSnapshot {
        id: element.id().to_string(),
        name: element.name().to_string(),
        category: element.category().as_str().to_string(),
        x,
        y,
        z,
        top_z: z,
        width: 0.0,
        depth: 0.0,
        thickness: 0.0,
        start: PointSnapshot { x, y, z },
        end: PointSnapshot { x, y, z },
        boundary: Vec::new(),
    };

    match element {
        Element::Column(column) => {
            let base_level = model
                .level(column.base_level_id)
                .expect("column base level exists");
            let top_level = model
                .level(column.top_level_id)
                .expect("column top level exists");
            let section = model
                .cross_section(column.cross_section_id)
                .expect("column cross section exists");
            snapshot.z = column
                .effective_base_elevation(base_level.elevation)
                .meters();
            snapshot.top_z = column.effective_top_elevation(top_level.elevation).meters();
            snapshot.width = section.width().meters();
            snapshot.depth = section.depth().meters();
        }
        Element::Beam(beam) => {
            snapshot.start = point_snapshot(beam.start_point);
            snapshot.end = point_snapshot(beam.end_point);
            let section = model
                .cross_section(beam.cross_section_id)
                .expect("beam cross section exists");
            snapshot.width = section.width().meters();
            snapshot.depth = section.depth().meters();
        }
        Element::Slab(slab) => {
            snapshot.thickness = slab.thickness.meters();
            snapshot.boundary = slab
                .boundary
                .vertices()
                .iter()
                .copied()
                .map(point_snapshot)
                .collect();
        }
        Element::Wall(_) | Element::Foundation(_) => {}
    }

    snapshot
}
