//! Named, reversible mutations of the engineering model.
//!
//! This module is the *only* place where a user intent becomes an engineering
//! change. Everything above it (touch, pen, mouse, keyboard, a future AI assistant)
//! must produce one of these commands — nothing may edit the model any other way.
//!
//! ```text
//! User Input → User Intent → Engineering Command → Engineering Model → Geometry → Viewport
//! ```
//!
//! # Why undo/redo is exact
//!
//! A command reports two lists of [`StateOp`]s — `forward` and `inverse` — and both
//! are expressed on *model data*:
//!
//! * **undo** applies `inverse`;
//! * **redo** applies `forward`.
//!
//! Because a creation records the created entity *itself* (identity included), redo
//! rebuilds exactly the same element instead of a look-alike with a fresh id: an
//! undo followed by a redo yields a byte-identical serialized model. Nothing here
//! knows about meshes, viewports or snapshots.
//!
//! Design rules kept from [`crate::commands`]: a command is data (serializable), it
//! applies through the model's own checked methods so it can never bypass an
//! invariant check nor forget to bump the revision, and it reports the identities it
//! created or changed.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::{
    BaseElement, Element, ElementCategory, Foundation, FoundationType, PlanBoundary,
    StructuralBeam, StructuralColumn, StructuralSlab, StructuralWall,
};
use crate::error::{ModelError, ReferenceKind};
use crate::math::{Point3D, Transform3D};
use crate::model::{CrossSection, EngineeringModel, Grid, GridDirection, Level, ProfileType};
use crate::units::Length;

/// Shape family of a foundation, as it crosses the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoundationKind {
    Isolated,
    Strip,
    Mat,
    PileCap,
}

impl FoundationKind {
    pub const fn to_model(self) -> FoundationType {
        match self {
            FoundationKind::Isolated => FoundationType::Isolated,
            FoundationKind::Strip => FoundationType::Strip,
            FoundationKind::Mat => FoundationType::Mat,
            FoundationKind::PileCap => FoundationType::PileCap,
        }
    }
}

/// One engineering change requested by the user.
///
/// Sizes are in internal metres and every reference is an id of the model — never a
/// copy of an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementCommand {
    /// Adds a level to the project.
    AddLevel { name: String, elevation_m: f64 },
    /// Adds a grid line to the project.
    AddGrid {
        name: String,
        direction: String,
        offset_m: f64,
    },
    /// Creates a column spanning two levels.
    CreateColumn {
        name: String,
        x_m: f64,
        y_m: f64,
        base_level_id: Uuid,
        top_level_id: Uuid,
        width_m: f64,
        depth_m: f64,
    },
    /// Creates a linear member between two points of one level.
    CreateBeam {
        name: String,
        start_x_m: f64,
        start_y_m: f64,
        end_x_m: f64,
        end_y_m: f64,
        level_id: Uuid,
        width_m: f64,
        depth_m: f64,
    },
    /// Creates a plate from a closed outline in plan.
    CreateSlab {
        name: String,
        level_id: Uuid,
        thickness_m: f64,
        points: Vec<[f64; 2]>,
    },
    /// Creates a wall between two points, spanning two levels.
    CreateWall {
        name: String,
        start_x_m: f64,
        start_y_m: f64,
        end_x_m: f64,
        end_y_m: f64,
        base_level_id: Uuid,
        top_level_id: Uuid,
        thickness_m: f64,
    },
    /// Creates a foundation footprint on a level.
    CreateFoundation {
        name: String,
        x_m: f64,
        y_m: f64,
        level_id: Uuid,
        thickness_m: f64,
        width_m: f64,
        depth_m: f64,
        kind: FoundationKind,
    },
    /// Moves an element by a displacement.
    MoveElement {
        id: Uuid,
        dx_m: f64,
        dy_m: f64,
        dz_m: f64,
    },
    /// Re-attaches the top of a column or wall to another level.
    ///
    /// This is what the *top grip* produces: the relationship of the element
    /// changes, so the derived geometry changes with it — the element is never
    /// merely lifted on screen.
    SetElementTopLevel { id: Uuid, top_level_id: Uuid },
    /// Re-attaches the bottom of a column or wall to another level.
    SetElementBaseLevel { id: Uuid, base_level_id: Uuid },
    /// Gives an element a different rectangular section.
    ///
    /// A section is a shared definition, so an existing section with the same size is
    /// reused instead of duplicated.
    SetElementSection {
        id: Uuid,
        width_mm: f64,
        depth_mm: f64,
    },
    /// Removes an element from the model. Deletion, not hiding.
    DeleteElement { id: Uuid },
    /// Copies an element onto another level, as a new element with a new identity.
    CopyElementToLevel { id: Uuid, target_level_id: Uuid },
}

impl ElementCommand {
    /// Stable, human-readable label used by history and the status bar.
    pub fn label(&self) -> String {
        match self {
            ElementCommand::AddLevel { name, .. } => format!("إضافة مستوى {name}"),
            ElementCommand::AddGrid { name, .. } => format!("إضافة محور {name}"),
            ElementCommand::CreateColumn { name, .. } => format!("إنشاء عمود {name}"),
            ElementCommand::CreateBeam { name, .. } => format!("إنشاء كمرة {name}"),
            ElementCommand::CreateSlab { name, .. } => format!("إنشاء بلاطة {name}"),
            ElementCommand::CreateWall { name, .. } => format!("إنشاء جدار {name}"),
            ElementCommand::CreateFoundation { name, .. } => format!("إنشاء أساس {name}"),
            ElementCommand::MoveElement { .. } => "تحريك عنصر".to_string(),
            ElementCommand::SetElementTopLevel { .. } => "تغيير المستوى العلوي".to_string(),
            ElementCommand::SetElementBaseLevel { .. } => "تغيير المستوى السفلي".to_string(),
            ElementCommand::SetElementSection { .. } => "تغيير القطاع".to_string(),
            ElementCommand::DeleteElement { .. } => "حذف عنصر".to_string(),
            ElementCommand::CopyElementToLevel { .. } => "نسخ إلى مستوى".to_string(),
        }
    }
}

/// One edit of engineering state.
///
/// The same type describes both directions of a change, which is what makes redo
/// exact: `CreateColumn` forwards an `AddElement` and inverses it with a
/// `RemoveElement`, while `DeleteElement` does exactly the opposite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateOp {
    /// Adds an entity exactly as given, identity included.
    AddElement(Box<Element>),
    /// Removes an element from the model.
    RemoveElement(Uuid),
    /// Sets the placement of an element.
    SetTransform { id: Uuid, transform: Transform3D },
    /// Sets the base level reference of an element.
    SetBaseLevel { id: Uuid, base_level_id: Uuid },
    /// Sets the top level reference of an element.
    SetTopLevel { id: Uuid, top_level_id: Uuid },
    /// Sets the cross section reference of an element.
    SetSection { id: Uuid, cross_section_id: Uuid },
    /// Adds a level exactly as given, identity included.
    AddLevel(Box<Level>),
    /// Removes a level from the model.
    RemoveLevel(Uuid),
    /// Adds a grid line exactly as given, identity included.
    AddGrid(Box<Grid>),
    /// Removes a grid line from the model.
    RemoveGrid(Uuid),
}

impl StateOp {
    /// Applies the operation to the model.
    pub fn apply(self, model: &mut EngineeringModel) -> Result<(), ModelError> {
        match self {
            StateOp::AddElement(element) => {
                model.add_element(*element)?;
                Ok(())
            }
            StateOp::RemoveElement(id) => model
                .remove_element(id)
                .map(|_| ())
                .ok_or(ModelError::MissingEntity { id }),
            StateOp::SetTransform { id, transform } => {
                let mut element = model
                    .element(id)
                    .cloned()
                    .ok_or(ModelError::MissingEntity { id })?;
                let base = element.base().clone().with_transform(transform);
                *element.base_mut() = base;
                model.replace_element(element)
            }
            StateOp::SetBaseLevel { id, base_level_id } => {
                edit_references(model, id, |element| match element {
                    Element::Column(column) => {
                        column.base_level_id = base_level_id;
                        Ok(())
                    }
                    Element::Wall(wall) => {
                        wall.base_level_id = base_level_id;
                        Ok(())
                    }
                    _ => {
                        return Err(ModelError::UnsupportedElementKind {
                            id,
                            reason: "العنصر لا يملك مستوى سفليًا".to_string(),
                        })
                    }
                })
            }
            StateOp::SetTopLevel { id, top_level_id } => {
                edit_references(model, id, |element| match element {
                    Element::Column(column) => {
                        column.top_level_id = top_level_id;
                        Ok(())
                    }
                    Element::Wall(wall) => {
                        wall.top_level_id = top_level_id;
                        Ok(())
                    }
                    _ => {
                        return Err(ModelError::UnsupportedElementKind {
                            id,
                            reason: "العنصر لا يملك مستوى علويًا".to_string(),
                        })
                    }
                })
            }
            StateOp::SetSection {
                id,
                cross_section_id,
            } => edit_references(model, id, |element| match element {
                Element::Column(column) => {
                    column.cross_section_id = cross_section_id;
                    Ok(())
                }
                Element::Beam(beam) => {
                    beam.cross_section_id = cross_section_id;
                    Ok(())
                }
                _ => {
                    return Err(ModelError::UnsupportedElementKind {
                        id,
                        reason: "العنصر لا يملك قطاعًا".to_string(),
                    })
                }
            }),
            StateOp::AddLevel(level) => {
                model.add_level(*level)?;
                Ok(())
            }
            StateOp::RemoveLevel(id) => model
                .remove_level(id)
                .map(|_| ())
                .ok_or(ModelError::MissingEntity { id }),
            StateOp::AddGrid(grid) => {
                model.add_grid(*grid)?;
                Ok(())
            }
            StateOp::RemoveGrid(id) => model
                .remove_grid(id)
                .map(|_| ())
                .ok_or(ModelError::MissingEntity { id }),
        }
    }
}

/// Clones an element, edits its references, and stores it back through the model's
/// checked replacement path.
fn edit_references<F>(model: &mut EngineeringModel, id: Uuid, edit: F) -> Result<(), ModelError>
where
    F: FnOnce(&mut Element) -> Result<(), ModelError>,
{
    let mut element = model
        .element(id)
        .cloned()
        .ok_or(ModelError::MissingEntity { id })?;
    edit(&mut element)?;
    model.replace_element(element)
}

/// What a command changed, and the two directions that describe the change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandOutcome {
    /// Entities created, changed or removed by the command.
    pub affected: Vec<Uuid>,
    /// Operations that redo the change.
    pub forward: Vec<StateOp>,
    /// Operations that undo the change.
    pub inverse: Vec<StateOp>,
}

impl CommandOutcome {
    fn created(id: Uuid, element: Element) -> Self {
        Self {
            affected: vec![id],
            forward: vec![StateOp::AddElement(Box::new(element))],
            inverse: vec![StateOp::RemoveElement(id)],
        }
    }

    fn changed(id: Uuid, forward: StateOp, inverse: StateOp) -> Self {
        Self {
            affected: vec![id],
            forward: vec![forward],
            inverse: vec![inverse],
        }
    }
}

/// Parameters a command may need that the user did not specify.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Defaults {
    /// Material every new element references unless told otherwise.
    pub material_id: Uuid,
}

impl ElementCommand {
    /// Applies the command to the model.
    pub fn execute(
        self,
        model: &mut EngineeringModel,
        defaults: Defaults,
    ) -> Result<CommandOutcome, ModelError> {
        match self {
            ElementCommand::AddLevel { name, elevation_m } => {
                let level = Level::new(name, Length::from_meters(elevation_m));
                let id = level.id;
                let outcome = CommandOutcome {
                    affected: vec![id],
                    forward: vec![StateOp::AddLevel(Box::new(level.clone()))],
                    inverse: vec![StateOp::RemoveLevel(id)],
                };
                model.add_level(level)?;
                Ok(outcome)
            }

            ElementCommand::AddGrid {
                name,
                direction,
                offset_m,
            } => {
                let offset = Length::from_meters(offset_m);
                let grid = if direction == GridDirection::AlongX.as_str() {
                    Grid::along_x(name, offset)
                } else {
                    Grid::along_y(name, offset)
                };
                let id = grid.id;
                let outcome = CommandOutcome {
                    affected: vec![id],
                    forward: vec![StateOp::AddGrid(Box::new(grid.clone()))],
                    inverse: vec![StateOp::RemoveGrid(id)],
                };
                model.add_grid(grid)?;
                Ok(outcome)
            }

            ElementCommand::CreateColumn {
                name,
                x_m,
                y_m,
                base_level_id,
                top_level_id,
                width_m,
                depth_m,
            } => {
                if is_above(model, base_level_id, top_level_id)? {
                    return Err(ModelError::InvalidExtent {
                        reason: "المستوى العلوي يجب أن يكون أعلى من المستوى السفلي".to_string(),
                    });
                }
                let section_id = resolve_rectangular_section(model, width_m, depth_m)?;
                let base = BaseElement::new(ElementCategory::Column, name)
                    .with_transform(Transform3D::at(Point3D::from_meters(x_m, y_m, 0.0)));
                let element = Element::Column(StructuralColumn::new(
                    base,
                    base_level_id,
                    top_level_id,
                    section_id,
                    defaults.material_id,
                ));
                let id = element.id();
                let outcome = CommandOutcome::created(id, element.clone());
                model.add_element(element)?;
                Ok(outcome)
            }

            ElementCommand::CreateBeam {
                name,
                start_x_m,
                start_y_m,
                end_x_m,
                end_y_m,
                level_id,
                width_m,
                depth_m,
            } => {
                if start_x_m == end_x_m && start_y_m == end_y_m {
                    return Err(ModelError::InvalidExtent {
                        reason: "الكمرة تحتاج نقطتي بداية ونهاية مختلفتين".to_string(),
                    });
                }
                let section_id = resolve_rectangular_section(model, width_m, depth_m)?;
                let elevation = level_elevation(model, level_id)?;
                let base = BaseElement::new(ElementCategory::Beam, name);
                let element = Element::Beam(StructuralBeam::new(
                    base,
                    level_id,
                    Point3D::from_meters(start_x_m, start_y_m, elevation),
                    Point3D::from_meters(end_x_m, end_y_m, elevation),
                    section_id,
                    defaults.material_id,
                ));
                let id = element.id();
                let outcome = CommandOutcome::created(id, element.clone());
                model.add_element(element)?;
                Ok(outcome)
            }

            ElementCommand::CreateSlab {
                name,
                level_id,
                thickness_m,
                points,
            } => {
                if points.len() < PlanBoundary::MIN_VERTICES {
                    return Err(ModelError::InvalidBoundary {
                        reason: format!(
                            "البلاطة تحتاج {} نقاط على الأقل، تم إدخال {}",
                            PlanBoundary::MIN_VERTICES,
                            points.len()
                        ),
                    });
                }
                let elevation = level_elevation(model, level_id)?;
                let vertices = points
                    .iter()
                    .map(|[x, y]| Point3D::from_meters(*x, *y, elevation))
                    .collect();
                let boundary = PlanBoundary::new(vertices)?;
                let base = BaseElement::new(ElementCategory::Slab, name);
                let element = Element::Slab(StructuralSlab::new(
                    base,
                    level_id,
                    Length::from_meters(thickness_m),
                    defaults.material_id,
                    boundary,
                ));
                let id = element.id();
                let outcome = CommandOutcome::created(id, element.clone());
                model.add_element(element)?;
                Ok(outcome)
            }

            ElementCommand::CreateWall {
                name,
                start_x_m,
                start_y_m,
                end_x_m,
                end_y_m,
                base_level_id,
                top_level_id,
                thickness_m,
            } => {
                if start_x_m == end_x_m && start_y_m == end_y_m {
                    return Err(ModelError::InvalidExtent {
                        reason: "الجدار يحتاج نقطتي بداية ونهاية مختلفتين".to_string(),
                    });
                }
                if is_above(model, base_level_id, top_level_id)? {
                    return Err(ModelError::InvalidExtent {
                        reason: "المستوى العلوي للجدار يجب أن يكون أعلى من المستوى السفلي"
                            .to_string(),
                    });
                }
                let elevation = level_elevation(model, base_level_id)?;
                let base = BaseElement::new(ElementCategory::Wall, name);
                let element = Element::Wall(StructuralWall::new(
                    base,
                    base_level_id,
                    top_level_id,
                    Point3D::from_meters(start_x_m, start_y_m, elevation),
                    Point3D::from_meters(end_x_m, end_y_m, elevation),
                    Length::from_meters(thickness_m),
                    defaults.material_id,
                ));
                let id = element.id();
                let outcome = CommandOutcome::created(id, element.clone());
                model.add_element(element)?;
                Ok(outcome)
            }

            ElementCommand::CreateFoundation {
                name,
                x_m,
                y_m,
                level_id,
                thickness_m,
                width_m,
                depth_m,
                kind,
            } => {
                let elevation = level_elevation(model, level_id)?;
                let origin =
                    Point3D::from_meters(x_m - width_m / 2.0, y_m - depth_m / 2.0, elevation);
                let footprint = PlanBoundary::rectangle(
                    origin,
                    Length::from_meters(width_m),
                    Length::from_meters(depth_m),
                );
                let base = BaseElement::new(ElementCategory::Foundation, name);
                let element = Element::Foundation(Foundation::new(
                    base,
                    level_id,
                    Length::from_meters(thickness_m),
                    defaults.material_id,
                    kind.to_model(),
                    footprint,
                ));
                let id = element.id();
                let outcome = CommandOutcome::created(id, element.clone());
                model.add_element(element)?;
                Ok(outcome)
            }

            ElementCommand::MoveElement { id, dx_m, dy_m, dz_m } => {
                let element = model
                    .element(id)
                    .ok_or(ModelError::MissingEntity { id })?;
                let previous = element.base().transform;
                let moved = Transform3D {
                    translation: Point3D::from_meters(
                        previous.translation.x.meters() + dx_m,
                        previous.translation.y.meters() + dy_m,
                        previous.translation.z.meters() + dz_m,
                    ),
                    rotation: previous.rotation,
                };
                let outcome = CommandOutcome::changed(
                    id,
                    StateOp::SetTransform {
                        id,
                        transform: moved,
                    },
                    StateOp::SetTransform {
                        id,
                        transform: previous,
                    },
                );
                outcome.forward[0].clone().apply(model)?;
                Ok(outcome)
            }

            ElementCommand::SetElementTopLevel { id, top_level_id } => {
                let element = model
                    .element(id)
                    .ok_or(ModelError::MissingEntity { id })?;
                let previous = match element {
                    Element::Column(column) => column.top_level_id,
                    Element::Wall(wall) => wall.top_level_id,
                    _ => {
                        return Err(ModelError::UnsupportedElementKind {
                            id,
                            reason: "العنصر لا يملك مستوى علويًا".to_string(),
                        })
                    }
                };
                let outcome = CommandOutcome::changed(
                    id,
                    StateOp::SetTopLevel { id, top_level_id },
                    StateOp::SetTopLevel {
                        id,
                        top_level_id: previous,
                    },
                );
                outcome.forward[0].clone().apply(model)?;
                Ok(outcome)
            }

            ElementCommand::SetElementBaseLevel { id, base_level_id } => {
                let element = model
                    .element(id)
                    .ok_or(ModelError::MissingEntity { id })?;
                let previous = match element {
                    Element::Column(column) => column.base_level_id,
                    Element::Wall(wall) => wall.base_level_id,
                    _ => {
                        return Err(ModelError::UnsupportedElementKind {
                            id,
                            reason: "العنصر لا يملك مستوى سفليًا".to_string(),
                        })
                    }
                };
                let outcome = CommandOutcome::changed(
                    id,
                    StateOp::SetBaseLevel { id, base_level_id },
                    StateOp::SetBaseLevel {
                        id,
                        base_level_id: previous,
                    },
                );
                outcome.forward[0].clone().apply(model)?;
                Ok(outcome)
            }

            ElementCommand::SetElementSection {
                id,
                width_mm,
                depth_mm,
            } => {
                if width_mm <= 0.0 || depth_mm <= 0.0 {
                    return Err(ModelError::InvalidExtent {
                        reason: "أبعاد القطاع يجب أن تكون موجبة".to_string(),
                    });
                }
                let element = model
                    .element(id)
                    .ok_or(ModelError::MissingEntity { id })?;
                let previous = match element {
                    Element::Column(column) => column.cross_section_id,
                    Element::Beam(beam) => beam.cross_section_id,
                    _ => {
                        return Err(ModelError::UnsupportedElementKind {
                            id,
                            reason: "العنصر لا يملك قطاعًا".to_string(),
                        })
                    }
                };
                let cross_section_id = resolve_rectangular_section_mm(model, width_mm, depth_mm)?;
                let outcome = CommandOutcome {
                    affected: vec![id, cross_section_id],
                    forward: vec![StateOp::SetSection {
                        id,
                        cross_section_id,
                    }],
                    inverse: vec![StateOp::SetSection {
                        id,
                        cross_section_id: previous,
                    }],
                };
                outcome.forward[0].clone().apply(model)?;
                Ok(outcome)
            }

            ElementCommand::DeleteElement { id } => {
                let removed = model
                    .element(id)
                    .cloned()
                    .ok_or(ModelError::MissingEntity { id })?;
                let outcome = CommandOutcome {
                    affected: vec![id],
                    forward: vec![StateOp::RemoveElement(id)],
                    inverse: vec![StateOp::AddElement(Box::new(removed))],
                };
                outcome.forward[0].clone().apply(model)?;
                Ok(outcome)
            }

            ElementCommand::CopyElementToLevel { id, target_level_id } => {
                let source = model
                    .element(id)
                    .cloned()
                    .ok_or(ModelError::MissingEntity { id })?;
                let target = model
                    .level(target_level_id)
                    .ok_or(ModelError::MissingReference {
                        element_id: id,
                        reference: ReferenceKind::Level,
                        missing_id: target_level_id,
                    })?;
                let mut copy = source.clone();
                let new_id = Uuid::new_v4();
                let new_name = format!("{}-{}", source.name(), target.name);
                *copy.base_mut() = BaseElement::with_id(new_id, source.category(), new_name);
                rebind_levels(&mut copy, target_level_id);
                let outcome = CommandOutcome::created(new_id, copy.clone());
                model.add_element(copy)?;
                Ok(outcome)
            }
        }
    }
}

/// Points every level reference of a copied element at the target level.
fn rebind_levels(element: &mut Element, target_level_id: Uuid) {
    match element {
        Element::Column(column) => {
            column.base_level_id = target_level_id;
            column.top_level_id = target_level_id;
        }
        Element::Wall(wall) => {
            wall.base_level_id = target_level_id;
            wall.top_level_id = target_level_id;
        }
        Element::Beam(beam) => beam.reference_level_id = target_level_id,
        Element::Slab(slab) => slab.level_id = target_level_id,
        Element::Foundation(foundation) => foundation.level_id = target_level_id,
    }
}

/// Elevation of a level in metres, or a missing-reference error.
fn level_elevation(model: &EngineeringModel, level_id: Uuid) -> Result<f64, ModelError> {
    model
        .level(level_id)
        .map(|level| level.elevation.meters())
        .ok_or(ModelError::MissingReference {
            element_id: Uuid::nil(),
            reference: ReferenceKind::Level,
            missing_id: level_id,
        })
}

/// `true` when the base level would end up above the top level.
fn is_above(
    model: &EngineeringModel,
    base_level_id: Uuid,
    top_level_id: Uuid,
) -> Result<bool, ModelError> {
    let base = level_elevation(model, base_level_id)?;
    let top = level_elevation(model, top_level_id)?;
    Ok(top < base)
}

/// Finds a rectangular section of these metres, or creates it.
pub fn resolve_rectangular_section(
    model: &mut EngineeringModel,
    width_m: f64,
    depth_m: f64,
) -> Result<Uuid, ModelError> {
    resolve_rectangular_section_mm(model, width_m * 1000.0, depth_m * 1000.0)
}

/// Finds a rectangular section of these millimetres, or creates it.
///
/// Sections are shared definitions: changing a column's size therefore reuses an
/// existing section when the project already has one instead of duplicating it.
pub fn resolve_rectangular_section_mm(
    model: &mut EngineeringModel,
    width_mm: f64,
    depth_mm: f64,
) -> Result<Uuid, ModelError> {
    let width = Length::from_millimeters(width_mm);
    let depth = Length::from_millimeters(depth_mm);
    for section in model.cross_sections.values() {
        if let ProfileType::Rectangular {
            width: existing_width,
            depth: existing_depth,
        } = section.profile
        {
            if (existing_width.meters() - width.meters()).abs() < 1e-9
                && (existing_depth.meters() - depth.meters()).abs() < 1e-9
            {
                return Ok(section.id);
            }
        }
    }
    let name = format!("{}x{}", width_mm.round() as i64, depth_mm.round() as i64);
    model.add_cross_section(CrossSection::rectangular(name, width, depth))
}

/// Direction names a grid line can be created with.
pub const GRID_DIRECTIONS: [&str; 2] = [
    GridDirection::AlongX.as_str(),
    GridDirection::AlongY.as_str(),
];
