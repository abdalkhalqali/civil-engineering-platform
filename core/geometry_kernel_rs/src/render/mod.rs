//! Derived render data.
//!
//! ```text
//! Engineering Model → Render Data → Viewport Renderer
//! ```
//!
//! This module is the **only** thing a renderer ever sees. It is a *projection* of
//! the model, computed on demand and never stored: there is no mesh in the model,
//! no element id in a mesh and no renderer state in the model.
//!
//! Two rules make the boundary safe:
//!
//! * **Nothing here mutates the model.** Every function takes `&EngineeringModel`.
//! * **Every primitive carries the id of the element it came from**, so a renderer
//!   can pick an element, highlight it and map it back to the model — without ever
//!   becoming a second source of truth.
//!
//! Changing the renderer (Canvas, WebGL, WebGPU, native) never touches this contract
//! or the model; changing the construction of a box here never touches project data.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::Element;
use crate::math::Point3D;
use crate::model::EngineeringModel;

/// Margin, in metres, added around the model when placing grid lines and labels.
const GRID_MARGIN_M: f64 = 2.0;

/// Extent of the model in plan and in elevation, in metres.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RenderBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl RenderBounds {
    /// A bounds box that contains everything the model places in space.
    fn empty() -> Self {
        Self {
            min: [f64::MAX; 3],
            max: [f64::MIN; 3],
        }
    }

    fn include(&mut self, point: [f64; 3]) {
        for axis in 0..3 {
            if point[axis] < self.min[axis] {
                self.min[axis] = point[axis];
            }
            if point[axis] > self.max[axis] {
                self.max[axis] = point[axis];
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.min[0] > self.max[0]
    }
}

/// A level, as the viewport needs it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderLevel {
    pub id: String,
    pub name: String,
    pub elevation_m: f64,
}

/// A grid line, as the viewport needs it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderGrid {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub offset_m: f64,
}

/// A linear member: the renderer builds the oriented solid from these numbers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderMember {
    /// Element this primitive was derived from.
    pub element_id: String,
    pub name: String,
    pub category: String,
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub width_m: f64,
    pub depth_m: f64,
}

/// A horizontal plate: a plan outline swept between two elevations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPlate {
    pub element_id: String,
    pub name: String,
    pub category: String,
    pub z_bottom: f64,
    pub z_top: f64,
    /// Outline in plan, implicitly closed.
    pub points: Vec<[f64; 2]>,
}

/// A text anchor for the viewport (grid bubbles, level names).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderLabel {
    pub text: String,
    pub position: [f64; 3],
    pub kind: String,
}

/// Everything a renderer needs, and nothing it must not have.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderData {
    /// Schema tag of this payload, so a renderer can refuse data it does not know.
    pub schema: String,
    pub project_id: String,
    /// Model revision this data was derived from.
    pub revision: u64,
    /// Display unit of the payload's lengths (`m`).
    pub units: String,
    pub bounds: RenderBounds,
    /// Plan extent of the drawn grid, as `[x_min, y_min, x_max, y_max]`.
    pub grid_extent: [f64; 4],
    pub levels: Vec<RenderLevel>,
    pub grids: Vec<RenderGrid>,
    pub members: Vec<RenderMember>,
    pub plates: Vec<RenderPlate>,
    pub labels: Vec<RenderLabel>,
}

/// Schema tag of [`RenderData`].
pub const RENDER_SCHEMA: &str = "civilx.render/1";

/// Projects the engineering model into render data.
///
/// This is a pure function: the same model always produces the same data, and the
/// model is never modified.
pub fn render_data(model: &EngineeringModel) -> RenderData {
    let mut members = Vec::new();
    let mut plates = Vec::new();
    let mut bounds = RenderBounds::empty();

    for element in model.elements.values() {
        match element {
            Element::Column(column) => {
                let position = column.base.transform.position();
                let base = level_elevation(model, column.base_level_id).unwrap_or(0.0)
                    + column.base_offset.meters();
                let top = level_elevation(model, column.top_level_id).unwrap_or(0.0)
                    + column.top_offset.meters();
                let (width, depth) = section_size(model, column.cross_section_id);
                let start = [position.x.meters(), position.y.meters(), base];
                let end = [position.x.meters(), position.y.meters(), top];
                bounds.include([start[0] - width / 2.0, start[1] - depth / 2.0, base]);
                bounds.include([start[0] + width / 2.0, start[1] + depth / 2.0, top]);
                members.push(RenderMember {
                    element_id: column.base.id.to_string(),
                    name: column.base.name.clone(),
                    category: "column".to_string(),
                    start,
                    end,
                    width_m: width,
                    depth_m: depth,
                });
            }
            Element::Beam(beam) => {
                let (width, depth) = section_size(model, beam.cross_section_id);
                let start = point_to_array(beam.start_point);
                let end = point_to_array(beam.end_point);
                bounds.include(start);
                bounds.include(end);
                members.push(RenderMember {
                    element_id: beam.base.id.to_string(),
                    name: beam.base.name.clone(),
                    category: "beam".to_string(),
                    start,
                    end,
                    width_m: width,
                    depth_m: depth,
                });
            }
            Element::Wall(wall) => {
                let base = level_elevation(model, wall.base_level_id).unwrap_or(0.0);
                let top = level_elevation(model, wall.top_level_id).unwrap_or(0.0);
                let thickness = wall.thickness.meters();
                let outline = band_outline(
                    [
                        wall.start_point.x.meters(),
                        wall.start_point.y.meters(),
                    ],
                    [wall.end_point.x.meters(), wall.end_point.y.meters()],
                    thickness,
                );
                for point in &outline {
                    bounds.include([point[0], point[1], base]);
                    bounds.include([point[0], point[1], top]);
                }
                plates.push(RenderPlate {
                    element_id: wall.base.id.to_string(),
                    name: wall.base.name.clone(),
                    category: "wall".to_string(),
                    z_bottom: base,
                    z_top: top,
                    points: outline,
                });
            }
            Element::Slab(slab) => {
                let top = level_elevation(model, slab.level_id).unwrap_or(0.0);
                let bottom = top - slab.thickness.meters();
                let outline: Vec<[f64; 2]> = slab
                    .boundary
                    .vertices()
                    .iter()
                    .map(|vertex| [vertex.x.meters(), vertex.y.meters()])
                    .collect();
                for point in &outline {
                    bounds.include([point[0], point[1], bottom]);
                    bounds.include([point[0], point[1], top]);
                }
                plates.push(RenderPlate {
                    element_id: slab.base.id.to_string(),
                    name: slab.base.name.clone(),
                    category: "slab".to_string(),
                    z_bottom: bottom,
                    z_top: top,
                    points: outline,
                });
            }
            Element::Foundation(foundation) => {
                let top = level_elevation(model, foundation.level_id).unwrap_or(0.0);
                let bottom = top - foundation.thickness.meters();
                let outline: Vec<[f64; 2]> = foundation
                    .footprint
                    .vertices()
                    .iter()
                    .map(|vertex| [vertex.x.meters(), vertex.y.meters()])
                    .collect();
                for point in &outline {
                    bounds.include([point[0], point[1], bottom]);
                    bounds.include([point[0], point[1], top]);
                }
                plates.push(RenderPlate {
                    element_id: foundation.base.id.to_string(),
                    name: foundation.base.name.clone(),
                    category: "foundation".to_string(),
                    z_bottom: bottom,
                    z_top: top,
                    points: outline,
                });
            }
        }
    }

    let mut levels: Vec<RenderLevel> = model
        .levels
        .values()
        .map(|level| RenderLevel {
            id: level.id.to_string(),
            name: level.name.clone(),
            elevation_m: level.elevation.meters(),
        })
        .collect();
    levels.sort_by(|a, b| a.elevation_m.partial_cmp(&b.elevation_m).unwrap_or(std::cmp::Ordering::Equal));

    let grids: Vec<RenderGrid> = model
        .grids
        .values()
        .map(|grid| RenderGrid {
            id: grid.id.to_string(),
            name: grid.name.clone(),
            direction: grid.direction.as_str().to_string(),
            offset_m: grid.offset.meters(),
        })
        .collect();

    for grid in model.grids.values() {
        let offset = grid.offset.meters();
        if grid.direction.is_parallel_to_y() {
            bounds.include([offset, 0.0, 0.0]);
        } else {
            bounds.include([0.0, offset, 0.0]);
        }
    }
    for level in model.levels.values() {
        bounds.include([0.0, 0.0, level.elevation.meters()]);
    }
    if bounds.is_empty() {
        bounds = RenderBounds {
            min: [0.0, 0.0, 0.0],
            max: [0.0, 0.0, 0.0],
        };
    }

    let grid_extent = [
        bounds.min[0] - GRID_MARGIN_M,
        bounds.min[1] - GRID_MARGIN_M,
        bounds.max[0] + GRID_MARGIN_M,
        bounds.max[1] + GRID_MARGIN_M,
    ];

    let mut labels = Vec::new();
    for grid in model.grids.values() {
        let offset = grid.offset.meters();
        let position = if grid.direction.is_parallel_to_y() {
            [offset, grid_extent[3], 0.0]
        } else {
            [grid_extent[2], offset, 0.0]
        };
        labels.push(RenderLabel {
            text: grid.name.clone(),
            position,
            kind: "grid".to_string(),
        });
    }
    for level in model.levels.values() {
        labels.push(RenderLabel {
            text: format!("{} ({:.2} m)", level.name, level.elevation.meters()),
            position: [grid_extent[0], grid_extent[1], level.elevation.meters()],
            kind: "level".to_string(),
        });
    }

    RenderData {
        schema: RENDER_SCHEMA.to_string(),
        project_id: model.project_id.to_string(),
        revision: model.revision,
        units: "m".to_string(),
        bounds,
        grid_extent,
        levels,
        grids,
        members,
        plates,
        labels,
    }
}

/// Plan outline of a wall centre line, expanded by half the thickness on both sides.
fn band_outline(start: [f64; 2], end: [f64; 2], thickness: f64) -> Vec<[f64; 2]> {
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f64::EPSILON {
        return Vec::new();
    }
    let half = thickness / 2.0;
    let nx = -dy / length * half;
    let ny = dx / length * half;
    vec![
        [start[0] + nx, start[1] + ny],
        [end[0] + nx, end[1] + ny],
        [end[0] - nx, end[1] - ny],
        [start[0] - nx, start[1] - ny],
    ]
}

fn level_elevation(model: &EngineeringModel, level_id: Uuid) -> Option<f64> {
    model.level(level_id).map(|level| level.elevation.meters())
}

/// `(width, depth)` of a section in metres; `(0.3, 0.3)` when it is missing.
fn section_size(model: &EngineeringModel, cross_section_id: Uuid) -> (f64, f64) {
    match model.cross_section(cross_section_id) {
        Some(section) => (section.width().meters(), section.depth().meters()),
        None => (0.3, 0.3),
    }
}

fn point_to_array(point: Point3D) -> [f64; 3] {
    [
        point.x.meters(),
        point.y.meters(),
        point.z.meters(),
    ]
}
