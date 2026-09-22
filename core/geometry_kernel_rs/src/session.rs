//! The modelling session: the application layer over the engineering model.
//!
//! ```text
//! EngineeringModel  →  ModelCommand  →  ModelSession  →  Flutter / Web UI
//! ```
//!
//! `ModelSession` is a **thin wrapper over the existing kernel**, not a replacement
//! for it. It owns a [`Project`] (metadata + [`EngineeringModel`]) plus the editing
//! state that belongs to a working session — history, default material, active
//! level, element name counters.
//!
//! What it deliberately is *not*:
//!
//! * not a second model — it stores no geometry and no derived data;
//! * not a snapshot API — the UI receives a session state once, then **commands**
//!   that report only what they changed;
//! * not a viewport — camera, selection, tool and snap settings stay on the client,
//!   so moving the camera can never dirty the project.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::commands::elements::{
    resolve_rectangular_section, Defaults, ElementCommand, FoundationKind, StateOp,
};
use crate::elements::{Element, ElementCategory};
use crate::error::ModelError;
use crate::history::{CommandHistory, HistoryEntry};
use crate::model::{EngineeringModel, Grid, Level, Material};
use crate::project::{Project, ProjectFormatV1, ProjectSerializer};
use crate::render::{render_data, RenderData};
use crate::units::Length;

/// Default column section created with every new project, in millimetres.
pub const DEFAULT_COLUMN_SECTION_MM: f64 = 400.0;
/// Height of the second level created with every new project, in metres.
pub const DEFAULT_STOREY_HEIGHT_M: f64 = 3.2;

// ------------------------------------------------------------------------ DTOs

/// A level, as the client sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelDto {
    pub id: String,
    pub name: String,
    pub elevation_m: f64,
}

/// A grid line, as the client sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridDto {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub offset_m: f64,
}

/// A physical element summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementDto {
    pub id: String,
    pub name: String,
    pub category: String,
    /// Level the element is anchored to (base level for column/wall).
    pub level_id: String,
    /// Top level for elements that span two levels, empty otherwise.
    pub top_level_id: String,
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    pub top_z_m: f64,
    pub width_m: f64,
    pub depth_m: f64,
    pub thickness_m: f64,
    pub rotation_deg: f64,
}

/// Full detail of one element, for the properties panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementDetail {
    pub id: String,
    pub name: String,
    pub category: String,
    pub level_id: String,
    pub level_name: String,
    pub top_level_id: String,
    pub top_level_name: String,
    pub base_elevation_m: f64,
    pub top_elevation_m: f64,
    pub height_m: f64,
    pub x_m: f64,
    pub y_m: f64,
    pub width_m: f64,
    pub depth_m: f64,
    pub width_mm: f64,
    pub depth_mm: f64,
    pub thickness_m: f64,
    pub length_m: f64,
    pub rotation_deg: f64,
    pub section_id: String,
    pub section_name: String,
    pub material_id: String,
    pub material_name: String,
}

/// The whole state of a session, sent once when the client opens or reloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub project_id: String,
    pub name: String,
    pub project_type: String,
    pub land_area_m2: f64,
    pub revision: u64,
    pub active_level_id: String,
    pub levels: Vec<LevelDto>,
    pub grids: Vec<GridDto>,
    pub elements: Vec<ElementDto>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub history: Vec<String>,
}

/// A command, in the flat form the bridge can carry.
///
/// Every field is a primitive or a `Vec` of primitives so the same request shape
/// works through `flutter_rust_bridge`, through JSON, and through a future WASM
/// binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandRequest {
    /// Command name, for example `create_column`.
    pub command: String,
    /// Element or level name; empty means "let the session name it".
    pub name: String,
    pub element_id: String,
    pub level_id: String,
    pub base_level_id: String,
    pub top_level_id: String,
    pub x_m: f64,
    pub y_m: f64,
    pub width_m: f64,
    pub depth_m: f64,
    pub thickness_m: f64,
    pub width_mm: f64,
    pub depth_mm: f64,
    pub start_x_m: f64,
    pub start_y_m: f64,
    pub end_x_m: f64,
    pub end_y_m: f64,
    pub dx_m: f64,
    pub dy_m: f64,
    pub dz_m: f64,
    pub direction: String,
    pub offset_m: f64,
    pub elevation_m: f64,
    pub foundation_kind: String,
    /// Slab outline, flattened as `x0, y0, x1, y1, ...`.
    pub points: Vec<f64>,
}

impl Default for CommandRequest {
    fn default() -> Self {
        Self {
            command: String::new(),
            name: String::new(),
            element_id: String::new(),
            level_id: String::new(),
            base_level_id: String::new(),
            top_level_id: String::new(),
            x_m: 0.0,
            y_m: 0.0,
            width_m: 0.0,
            depth_m: 0.0,
            thickness_m: 0.0,
            width_mm: 0.0,
            depth_mm: 0.0,
            start_x_m: 0.0,
            start_y_m: 0.0,
            end_x_m: 0.0,
            end_y_m: 0.0,
            dx_m: 0.0,
            dy_m: 0.0,
            dz_m: 0.0,
            direction: String::new(),
            offset_m: 0.0,
            elevation_m: 0.0,
            foundation_kind: String::new(),
            points: Vec::new(),
        }
    }
}

/// What a command did, in the flat form the bridge can carry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandResult {
    pub ok: bool,
    pub message: String,
    pub label: String,
    pub revision: u64,
    /// Only the entities the command created or changed.
    pub changed_ids: Vec<String>,
    pub can_undo: bool,
    pub can_redo: bool,
}

impl CommandResult {
    fn failed(message: impl Into<String>, revision: u64) -> Self {
        Self {
            ok: false,
            message: message.into(),
            label: String::new(),
            revision,
            changed_ids: Vec::new(),
            can_undo: false,
            can_redo: false,
        }
    }
}

/// One point the snapping system found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapCandidate {
    pub kind: String,
    pub label: String,
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    pub distance_m: f64,
}

/// The result of a snap query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapResult {
    pub snapped: bool,
    pub kind: String,
    pub label: String,
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    pub candidates: Vec<SnapCandidate>,
}

// --------------------------------------------------------------------- session

/// Name counters, so every new element gets a stable, readable identity.
#[derive(Debug, Clone, Default)]
struct NameCounters {
    counts: BTreeMap<String, u32>,
}

impl NameCounters {
    fn next(&mut self, prefix: &str) -> String {
        let counter = self.counts.entry(prefix.to_string()).or_insert(0);
        *counter += 1;
        format!("{prefix}{:03}", counter)
    }
}

/// A modelling session: the project plus its editing state.
pub struct ModelSession {
    project: Project,
    history: CommandHistory,
    defaults: Defaults,
    active_level_id: Uuid,
    counters: NameCounters,
    /// Entities changed by the most recent accepted command.
    changed: Vec<Uuid>,
}

impl ModelSession {
    /// Opens a new project with the modelling environment ready to build in:
    /// two levels, a four-line grid, one concrete material and one column section.
    ///
    /// Everything it creates is created *in the engineering model* through the
    /// model's own methods, so the starter state is model data — not a viewport
    /// setting and not a fixture of the UI.
    pub fn new(name: impl Into<String>, project_type: impl Into<String>, land_area_m2: f64) -> Self {
        let mut project = Project::new_with_site(name, project_type, land_area_m2);

        let material_id = project
            .model
            .add_material(Material::concrete_c30())
            .expect("a fresh model accepts a material");
        // The default section is resolved on demand by the commands, so only its
        // creation matters here: the project starts with a usable 400 x 400 section.
        resolve_rectangular_section(
            &mut project.model,
            DEFAULT_COLUMN_SECTION_MM / 1000.0,
            DEFAULT_COLUMN_SECTION_MM / 1000.0,
        )
        .expect("a fresh model accepts a section");

        let ground = project
            .model
            .add_level(Level::new("Level 1", Length::from_meters(0.0)))
            .expect("a fresh model accepts a level");
        project
            .model
            .add_level(Level::new(
                "Level 2",
                Length::from_meters(DEFAULT_STOREY_HEIGHT_M),
            ))
            .expect("a fresh model accepts a level");

        for (name, direction, offset) in [
            ("A", "along_y", 0.0),
            ("B", "along_y", 4.0),
            ("1", "along_x", 0.0),
            ("2", "along_x", 3.0),
        ] {
            let grid = if direction == "along_x" {
                Grid::along_x(name, Length::from_meters(offset))
            } else {
                Grid::along_y(name, Length::from_meters(offset))
            };
            project
                .model
                .add_grid(grid)
                .expect("a fresh model accepts a grid line");
        }

        Self {
            project,
            history: CommandHistory::default(),
            defaults: Defaults { material_id },
            active_level_id: ground,
            counters: NameCounters::default(),
            changed: Vec::new(),
        }
    }

    /// The project id.
    pub fn project_id(&self) -> Uuid {
        self.project.project_id()
    }

    /// The revision of the engineering model.
    pub fn revision(&self) -> u64 {
        self.project.model.revision
    }

    /// Read-only access to the model.
    pub fn model(&self) -> &EngineeringModel {
        &self.project.model
    }

    /// The level new elements are created on.
    pub fn active_level_id(&self) -> Uuid {
        self.active_level_id
    }

    /// Switches the level new elements are created on.
    ///
    /// The active level is *editing state*, so it is not recorded in history.
    pub fn set_active_level(&mut self, level_id: Uuid) -> bool {
        if self.project.model.level(level_id).is_some() {
            self.active_level_id = level_id;
            true
        } else {
            false
        }
    }

    /// Applies a command to the engineering model.
    pub fn execute(&mut self, command: ElementCommand) -> CommandResult {
        let label = command.label();
        let command = self.assign_name(command);
        match command.execute(&mut self.project.model, self.defaults) {
            Ok(outcome) => {
                self.changed = outcome.affected.clone();
                self.history.record(HistoryEntry::new(
                    label.clone(),
                    outcome.forward,
                    outcome.inverse,
                ));
                self.project.touch();
                CommandResult {
                    ok: true,
                    message: String::new(),
                    label,
                    revision: self.revision(),
                    changed_ids: self.changed.iter().map(Uuid::to_string).collect(),
                    can_undo: self.history.can_undo(),
                    can_redo: self.history.can_redo(),
                }
            }
            Err(error) => CommandResult::failed(error.to_string(), self.revision()),
        }
    }

    /// Sends a flat request from the bridge.
    pub fn execute_request(&mut self, request: CommandRequest) -> CommandResult {
        match ElementCommand::from_request(&request) {
            Ok(command) => self.execute(command),
            Err(message) => CommandResult::failed(message, self.revision()),
        }
    }

    /// Takes the model back by one change.
    pub fn undo(&mut self) -> CommandResult {
        match self.history.undo(&mut self.project.model) {
            Ok(Some(label)) => {
                self.project.touch();
                CommandResult {
                    ok: true,
                    message: String::new(),
                    label: format!("تراجع عن {label}"),
                    revision: self.revision(),
                    changed_ids: Vec::new(),
                    can_undo: self.history.can_undo(),
                    can_redo: self.history.can_redo(),
                }
            }
            Ok(None) => CommandResult::failed("لا يوجد ما يمكن التراجع عنه", self.revision()),
            Err(error) => CommandResult::failed(error.to_string(), self.revision()),
        }
    }

    /// Re-applies the last undone change.
    pub fn redo(&mut self) -> CommandResult {
        match self.history.redo(&mut self.project.model) {
            Ok(Some(label)) => {
                self.project.touch();
                CommandResult {
                    ok: true,
                    message: String::new(),
                    label: format!("إعادة {label}"),
                    revision: self.revision(),
                    changed_ids: Vec::new(),
                    can_undo: self.history.can_undo(),
                    can_redo: self.history.can_redo(),
                }
            }
            Ok(None) => CommandResult::failed("لا يوجد ما يمكن إعادته", self.revision()),
            Err(error) => CommandResult::failed(error.to_string(), self.revision()),
        }
    }

    /// The full state of the session.
    pub fn state(&self) -> SessionState {
        SessionState {
            project_id: self.project.project_id().to_string(),
            name: self.project.metadata.name.clone(),
            project_type: self.project.metadata.project_type.clone(),
            land_area_m2: self.project.metadata.land_area_m2,
            revision: self.revision(),
            active_level_id: self.active_level_id.to_string(),
            levels: self.levels(),
            grids: self.grids(),
            elements: self.elements(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            history: self.history.labels(),
        }
    }

    /// Derived render data. The model is not modified.
    pub fn render_data(&self) -> RenderData {
        render_data(&self.project.model)
    }

    /// Detail of one element, for the properties panel.
    pub fn element_details(&self, id: Uuid) -> Option<ElementDetail> {
        let element = self.project.model.element(id)?;
        let model = &self.project.model;
        let base = element.base();
        let level_name = |level_id: Uuid| {
            model
                .level(level_id)
                .map(|level| level.name.clone())
                .unwrap_or_default()
        };
        let level_elevation = |level_id: Uuid| {
            model
                .level(level_id)
                .map(|level| level.elevation.meters())
                .unwrap_or(0.0)
        };

        let mut detail = ElementDetail {
            id: id.to_string(),
            name: base.name.clone(),
            category: category_name(base.category).to_string(),
            level_id: String::new(),
            level_name: String::new(),
            top_level_id: String::new(),
            top_level_name: String::new(),
            base_elevation_m: 0.0,
            top_elevation_m: 0.0,
            height_m: 0.0,
            x_m: base.transform.translation.x.meters(),
            y_m: base.transform.translation.y.meters(),
            width_m: 0.0,
            depth_m: 0.0,
            width_mm: 0.0,
            depth_mm: 0.0,
            thickness_m: 0.0,
            length_m: 0.0,
            rotation_deg: base.transform.plan_rotation().degrees(),
            section_id: String::new(),
            section_name: String::new(),
            material_id: String::new(),
            material_name: String::new(),
        };

        match element {
            Element::Column(column) => {
                detail.level_id = column.base_level_id.to_string();
                detail.level_name = level_name(column.base_level_id);
                detail.top_level_id = column.top_level_id.to_string();
                detail.top_level_name = level_name(column.top_level_id);
                detail.base_elevation_m =
                    level_elevation(column.base_level_id) + column.base_offset.meters();
                detail.top_elevation_m =
                    level_elevation(column.top_level_id) + column.top_offset.meters();
                detail.material_id = column.material_id.to_string();
                detail.section_id = column.cross_section_id.to_string();
                fill_section(&mut detail, model.cross_section(column.cross_section_id));
            }
            Element::Beam(beam) => {
                detail.level_id = beam.reference_level_id.to_string();
                detail.level_name = level_name(beam.reference_level_id);
                detail.x_m = beam.start_point.x.meters();
                detail.y_m = beam.start_point.y.meters();
                detail.base_elevation_m = beam.start_point.z.meters();
                detail.top_elevation_m = beam.end_point.z.meters();
                detail.length_m = beam.length().meters();
                detail.material_id = beam.material_id.to_string();
                detail.section_id = beam.cross_section_id.to_string();
                fill_section(&mut detail, model.cross_section(beam.cross_section_id));
            }
            Element::Wall(wall) => {
                detail.level_id = wall.base_level_id.to_string();
                detail.level_name = level_name(wall.base_level_id);
                detail.top_level_id = wall.top_level_id.to_string();
                detail.top_level_name = level_name(wall.top_level_id);
                detail.x_m = wall.start_point.x.meters();
                detail.y_m = wall.start_point.y.meters();
                detail.base_elevation_m = level_elevation(wall.base_level_id);
                detail.top_elevation_m = level_elevation(wall.top_level_id);
                detail.length_m = wall.length().meters();
                detail.thickness_m = wall.thickness.meters();
                detail.material_id = wall.material_id.to_string();
            }
            Element::Slab(slab) => {
                detail.level_id = slab.level_id.to_string();
                detail.level_name = level_name(slab.level_id);
                detail.base_elevation_m = level_elevation(slab.level_id) - slab.thickness.meters();
                detail.top_elevation_m = level_elevation(slab.level_id);
                detail.thickness_m = slab.thickness.meters();
                detail.material_id = slab.material_id.to_string();
            }
            Element::Foundation(foundation) => {
                detail.level_id = foundation.level_id.to_string();
                detail.level_name = level_name(foundation.level_id);
                detail.base_elevation_m =
                    level_elevation(foundation.level_id) - foundation.thickness.meters();
                detail.top_elevation_m = level_elevation(foundation.level_id);
                detail.thickness_m = foundation.thickness.meters();
                detail.material_id = foundation.material_id.to_string();
            }
        }

        detail.height_m = detail.top_elevation_m - detail.base_elevation_m;
        detail.material_name = model
            .material(
                Uuid::parse_str(&detail.material_id).unwrap_or_else(|_| Uuid::nil()),
            )
            .map(|material| material.name.clone())
            .unwrap_or_default();
        Some(detail)
    }

    /// Finds the model point a finger or a pen should land on.
    ///
    /// Candidates are ordered by priority — grid intersections first, then element
    /// points, then the origin — and the closest candidate within `tolerance_m`
    /// wins. Returning the whole candidate list lets the UI show *which* point it
    /// is about to snap to before the element is placed.
    pub fn snap(&self, x_m: f64, y_m: f64, tolerance_m: f64, z_m: f64) -> SnapResult {
        let mut candidates: Vec<(u8, SnapCandidate)> = Vec::new();
        let push = |list: &mut Vec<(u8, SnapCandidate)>, priority: u8, kind: &str, label: String, x: f64, y: f64| {
            let distance = ((x - x_m).powi(2) + (y - y_m).powi(2)).sqrt();
            list.push((
                priority,
                SnapCandidate {
                    kind: kind.to_string(),
                    label,
                    x_m: x,
                    y_m: y,
                    z_m,
                    distance_m: distance,
                },
            ));
        };

        let mut along_y: Vec<(String, f64)> = Vec::new();
        let mut along_x: Vec<(String, f64)> = Vec::new();
        for grid in self.project.model.grids.values() {
            let offset = grid.offset.meters();
            if grid.direction.is_parallel_to_y() {
                along_y.push((grid.name.clone(), offset));
            } else {
                along_x.push((grid.name.clone(), offset));
            }
        }
        for (name_y, x) in &along_y {
            for (name_x, y) in &along_x {
                push(
                    &mut candidates,
                    0,
                    "grid_intersection",
                    format!("{name_y}-{name_x}"),
                    *x,
                    *y,
                );
            }
        }
        for (name, x) in &along_y {
            push(
                &mut candidates,
                3,
                "grid_line",
                format!("محور {name}"),
                *x,
                y_m,
            );
        }
        for (name, y) in &along_x {
            push(
                &mut candidates,
                3,
                "grid_line",
                format!("محور {name}"),
                x_m,
                *y,
            );
        }

        for element in self.project.model.elements.values() {
            match element {
                Element::Column(column) => {
                    let position = column.base.transform.position();
                    push(
                        &mut candidates,
                        1,
                        "element_center",
                        column.base.name.clone(),
                        position.x.meters(),
                        position.y.meters(),
                    );
                }
                Element::Beam(beam) => {
                    push(
                        &mut candidates,
                        1,
                        "beam_endpoint",
                        beam.base.name.clone(),
                        beam.start_point.x.meters(),
                        beam.start_point.y.meters(),
                    );
                    push(
                        &mut candidates,
                        1,
                        "beam_endpoint",
                        beam.base.name.clone(),
                        beam.end_point.x.meters(),
                        beam.end_point.y.meters(),
                    );
                }
                Element::Wall(wall) => {
                    push(
                        &mut candidates,
                        1,
                        "wall_endpoint",
                        wall.base.name.clone(),
                        wall.start_point.x.meters(),
                        wall.start_point.y.meters(),
                    );
                    push(
                        &mut candidates,
                        1,
                        "wall_endpoint",
                        wall.base.name.clone(),
                        wall.end_point.x.meters(),
                        wall.end_point.y.meters(),
                    );
                }
                _ => {}
            }
        }
        push(&mut candidates, 2, "origin", "نقطة الأصل".to_string(), 0.0, 0.0);

        candidates.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.distance_m.partial_cmp(&b.1.distance_m).unwrap_or(std::cmp::Ordering::Equal))
        });

        let within: Vec<SnapCandidate> = candidates
            .iter()
            .filter(|(_, candidate)| candidate.distance_m <= tolerance_m)
            .map(|(_, candidate)| candidate.clone())
            .take(8)
            .collect();

        match within.first() {
            Some(best) => SnapResult {
                snapped: true,
                kind: best.kind.clone(),
                label: best.label.clone(),
                x_m: best.x_m,
                y_m: best.y_m,
                z_m: best.z_m,
                candidates: within,
            },
            None => SnapResult {
                snapped: false,
                kind: "free".to_string(),
                label: String::new(),
                x_m,
                y_m,
                z_m,
                candidates: Vec::new(),
            },
        }
    }

    /// Serializes the project to `.civilx` bytes.
    pub fn save(&self) -> Result<Vec<u8>, String> {
        ProjectFormatV1::serialize(&self.project).map_err(|error| error.to_string())
    }

    /// Replaces the session content with a `.civilx` payload.
    ///
    /// The model is rebuilt from the file; geometry and viewport are regenerated
    /// from the rebuilt model by the caller.
    pub fn load(&mut self, data: &[u8]) -> Result<(), String> {
        let project = ProjectFormatV1::deserialize(data).map_err(|error| error.to_string())?;
        self.defaults.material_id = project
            .model
            .materials
            .values()
            .next()
            .map(|material| material.id)
            .or(Some(self.defaults.material_id))
            .expect("a material id is always available");
        self.active_level_id = project
            .model
            .levels
            .values()
            .min_by(|a, b| {
                a.elevation
                    .meters()
                    .partial_cmp(&b.elevation.meters())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|level| level.id)
            .unwrap_or_else(Uuid::nil);
        self.project = project;
        self.history.clear();
        self.changed.clear();
        self.counters = NameCounters::default();
        Ok(())
    }

    /// Entities changed by the most recent command.
    pub fn last_changed(&self) -> &[Uuid] {
        &self.changed
    }

    fn levels(&self) -> Vec<LevelDto> {
        let mut levels: Vec<LevelDto> = self
            .project
            .model
            .levels
            .values()
            .map(|level| LevelDto {
                id: level.id.to_string(),
                name: level.name.clone(),
                elevation_m: level.elevation.meters(),
            })
            .collect();
        levels.sort_by(|a, b| {
            a.elevation_m
                .partial_cmp(&b.elevation_m)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        levels
    }

    fn grids(&self) -> Vec<GridDto> {
        self.project
            .model
            .grids
            .values()
            .map(|grid| GridDto {
                id: grid.id.to_string(),
                name: grid.name.clone(),
                direction: grid.direction.as_str().to_string(),
                offset_m: grid.offset.meters(),
            })
            .collect()
    }

    fn elements(&self) -> Vec<ElementDto> {
        self.project
            .model
            .elements
            .values()
            .map(|element| self.element_dto(element))
            .collect()
    }

    fn element_dto(&self, element: &Element) -> ElementDto {
        let model = &self.project.model;
        let base = element.base();
        let mut dto = ElementDto {
            id: base.id.to_string(),
            name: base.name.clone(),
            category: category_name(base.category).to_string(),
            level_id: String::new(),
            top_level_id: String::new(),
            x_m: base.transform.translation.x.meters(),
            y_m: base.transform.translation.y.meters(),
            z_m: 0.0,
            top_z_m: 0.0,
            width_m: 0.0,
            depth_m: 0.0,
            thickness_m: 0.0,
            rotation_deg: base.transform.plan_rotation().degrees(),
        };
        let level_elevation = |id: Uuid| {
            model
                .level(id)
                .map(|level| level.elevation.meters())
                .unwrap_or(0.0)
        };
        match element {
            Element::Column(column) => {
                dto.level_id = column.base_level_id.to_string();
                dto.top_level_id = column.top_level_id.to_string();
                dto.z_m = level_elevation(column.base_level_id) + column.base_offset.meters();
                dto.top_z_m = level_elevation(column.top_level_id) + column.top_offset.meters();
                if let Some(section) = model.cross_section(column.cross_section_id) {
                    dto.width_m = section.width().meters();
                    dto.depth_m = section.depth().meters();
                }
            }
            Element::Beam(beam) => {
                dto.level_id = beam.reference_level_id.to_string();
                dto.x_m = beam.start_point.x.meters();
                dto.y_m = beam.start_point.y.meters();
                dto.z_m = beam.start_point.z.meters();
                dto.top_z_m = beam.end_point.z.meters();
                if let Some(section) = model.cross_section(beam.cross_section_id) {
                    dto.width_m = section.width().meters();
                    dto.depth_m = section.depth().meters();
                }
            }
            Element::Wall(wall) => {
                dto.level_id = wall.base_level_id.to_string();
                dto.top_level_id = wall.top_level_id.to_string();
                dto.x_m = wall.start_point.x.meters();
                dto.y_m = wall.start_point.y.meters();
                dto.z_m = level_elevation(wall.base_level_id);
                dto.top_z_m = level_elevation(wall.top_level_id);
                dto.thickness_m = wall.thickness.meters();
            }
            Element::Slab(slab) => {
                dto.level_id = slab.level_id.to_string();
                dto.z_m = level_elevation(slab.level_id) - slab.thickness.meters();
                dto.top_z_m = level_elevation(slab.level_id);
                dto.thickness_m = slab.thickness.meters();
            }
            Element::Foundation(foundation) => {
                dto.level_id = foundation.level_id.to_string();
                dto.z_m = level_elevation(foundation.level_id) - foundation.thickness.meters();
                dto.top_z_m = level_elevation(foundation.level_id);
                dto.thickness_m = foundation.thickness.meters();
            }
        }
        dto
    }

    /// Gives an unnamed command a readable identity such as `C001`.
    ///
    /// Naming is session state: it keeps the model free of counters while every
    /// element still ends up with a stable, readable name.
    fn assign_name(&mut self, command: ElementCommand) -> ElementCommand {
        match command {
            ElementCommand::CreateColumn {
                name,
                x_m,
                y_m,
                base_level_id,
                top_level_id,
                width_m,
                depth_m,
            } => ElementCommand::CreateColumn {
                name: self.name_or(name, "C"),
                x_m,
                y_m,
                base_level_id,
                top_level_id,
                width_m,
                depth_m,
            },
            ElementCommand::CreateBeam {
                name,
                start_x_m,
                start_y_m,
                end_x_m,
                end_y_m,
                level_id,
                width_m,
                depth_m,
            } => ElementCommand::CreateBeam {
                name: self.name_or(name, "B"),
                start_x_m,
                start_y_m,
                end_x_m,
                end_y_m,
                level_id,
                width_m,
                depth_m,
            },
            ElementCommand::CreateSlab {
                name,
                level_id,
                thickness_m,
                points,
            } => ElementCommand::CreateSlab {
                name: self.name_or(name, "S"),
                level_id,
                thickness_m,
                points,
            },
            ElementCommand::CreateWall {
                name,
                start_x_m,
                start_y_m,
                end_x_m,
                end_y_m,
                base_level_id,
                top_level_id,
                thickness_m,
            } => ElementCommand::CreateWall {
                name: self.name_or(name, "W"),
                start_x_m,
                start_y_m,
                end_x_m,
                end_y_m,
                base_level_id,
                top_level_id,
                thickness_m,
            },
            ElementCommand::CreateFoundation {
                name,
                x_m,
                y_m,
                level_id,
                thickness_m,
                width_m,
                depth_m,
                kind,
            } => ElementCommand::CreateFoundation {
                name: self.name_or(name, "F"),
                x_m,
                y_m,
                level_id,
                thickness_m,
                width_m,
                depth_m,
                kind,
            },
            other => other,
        }
    }

    fn name_or(&mut self, name: String, prefix: &str) -> String {
        if name.trim().is_empty() {
            self.counters.next(prefix)
        } else {
            name
        }
    }
}

fn fill_section(detail: &mut ElementDetail, section: Option<&crate::model::CrossSection>) {
    let Some(section) = section else {
        return;
    };
    detail.section_name = section.name.clone();
    detail.width_m = section.width().meters();
    detail.depth_m = section.depth().meters();
    detail.width_mm = section.width().millimeters();
    detail.depth_mm = section.depth().millimeters();
}

/// Stable category name used at the boundary.
pub fn category_name(category: ElementCategory) -> &'static str {
    match category {
        ElementCategory::Column => "column",
        ElementCategory::Beam => "beam",
        ElementCategory::Slab => "slab",
        ElementCategory::Wall => "wall",
        ElementCategory::Foundation => "foundation",
        ElementCategory::Level => "level",
        ElementCategory::Grid => "grid",
    }
}

impl ElementCommand {
    /// Builds a command from the flat boundary request.
    pub fn from_request(request: &CommandRequest) -> Result<ElementCommand, String> {
        let uuid = |value: &str, field: &str| -> Result<Uuid, String> {
            Uuid::parse_str(value).map_err(|_| format!("{field} غير صالح: {value}"))
        };
        match request.command.as_str() {
            "add_level" => Ok(ElementCommand::AddLevel {
                name: request.name.clone(),
                elevation_m: request.elevation_m,
            }),
            "add_grid" => Ok(ElementCommand::AddGrid {
                name: request.name.clone(),
                direction: if request.direction.is_empty() {
                    "along_y".to_string()
                } else {
                    request.direction.clone()
                },
                offset_m: request.offset_m,
            }),
            "create_column" => Ok(ElementCommand::CreateColumn {
                name: request.name.clone(),
                x_m: request.x_m,
                y_m: request.y_m,
                base_level_id: uuid(&request.base_level_id, "base_level_id")?,
                top_level_id: uuid(&request.top_level_id, "top_level_id")?,
                width_m: if request.width_m > 0.0 {
                    request.width_m
                } else {
                    DEFAULT_COLUMN_SECTION_MM / 1000.0
                },
                depth_m: if request.depth_m > 0.0 {
                    request.depth_m
                } else {
                    DEFAULT_COLUMN_SECTION_MM / 1000.0
                },
            }),
            "create_beam" => Ok(ElementCommand::CreateBeam {
                name: request.name.clone(),
                start_x_m: request.start_x_m,
                start_y_m: request.start_y_m,
                end_x_m: request.end_x_m,
                end_y_m: request.end_y_m,
                level_id: uuid(&request.level_id, "level_id")?,
                width_m: if request.width_m > 0.0 { request.width_m } else { 0.3 },
                depth_m: if request.depth_m > 0.0 { request.depth_m } else { 0.5 },
            }),
            "create_slab" => {
                if request.points.len() % 2 != 0 {
                    return Err("نقاط البلاطة يجب أن تكون أزواجًا (x, y)".to_string());
                }
                let points = request
                    .points
                    .chunks_exact(2)
                    .map(|pair| [pair[0], pair[1]])
                    .collect();
                Ok(ElementCommand::CreateSlab {
                    name: request.name.clone(),
                    level_id: uuid(&request.level_id, "level_id")?,
                    thickness_m: if request.thickness_m > 0.0 {
                        request.thickness_m
                    } else {
                        0.2
                    },
                    points,
                })
            }
            "create_wall" => Ok(ElementCommand::CreateWall {
                name: request.name.clone(),
                start_x_m: request.start_x_m,
                start_y_m: request.start_y_m,
                end_x_m: request.end_x_m,
                end_y_m: request.end_y_m,
                base_level_id: uuid(&request.base_level_id, "base_level_id")?,
                top_level_id: uuid(&request.top_level_id, "top_level_id")?,
                thickness_m: if request.thickness_m > 0.0 {
                    request.thickness_m
                } else {
                    0.2
                },
            }),
            "create_foundation" => Ok(ElementCommand::CreateFoundation {
                name: request.name.clone(),
                x_m: request.x_m,
                y_m: request.y_m,
                level_id: uuid(&request.level_id, "level_id")?,
                thickness_m: if request.thickness_m > 0.0 {
                    request.thickness_m
                } else {
                    0.5
                },
                width_m: if request.width_m > 0.0 { request.width_m } else { 1.5 },
                depth_m: if request.depth_m > 0.0 { request.depth_m } else { 1.5 },
                kind: match request.foundation_kind.as_str() {
                    "strip" => FoundationKind::Strip,
                    "mat" => FoundationKind::Mat,
                    "pile_cap" => FoundationKind::PileCap,
                    _ => FoundationKind::Isolated,
                },
            }),
            "move_element" => Ok(ElementCommand::MoveElement {
                id: uuid(&request.element_id, "element_id")?,
                dx_m: request.dx_m,
                dy_m: request.dy_m,
                dz_m: request.dz_m,
            }),
            "set_top_level" => Ok(ElementCommand::SetElementTopLevel {
                id: uuid(&request.element_id, "element_id")?,
                top_level_id: uuid(&request.top_level_id, "top_level_id")?,
            }),
            "set_base_level" => Ok(ElementCommand::SetElementBaseLevel {
                id: uuid(&request.element_id, "element_id")?,
                base_level_id: uuid(&request.base_level_id, "base_level_id")?,
            }),
            "set_section" => Ok(ElementCommand::SetElementSection {
                id: uuid(&request.element_id, "element_id")?,
                width_mm: request.width_mm,
                depth_mm: request.depth_mm,
            }),
            "delete_element" => Ok(ElementCommand::DeleteElement {
                id: uuid(&request.element_id, "element_id")?,
            }),
            "copy_to_level" => Ok(ElementCommand::CopyElementToLevel {
                id: uuid(&request.element_id, "element_id")?,
                target_level_id: uuid(&request.top_level_id, "top_level_id")?,
            }),
            other => Err(format!("أمر غير معروف: {other}")),
        }
    }
}

/// Applies a list of state operations to a model (used by tests and tooling).
pub fn apply_state_ops(
    model: &mut EngineeringModel,
    operations: &[StateOp],
) -> Result<(), ModelError> {
    for operation in operations {
        operation.clone().apply(model)?;
    }
    Ok(())
}
