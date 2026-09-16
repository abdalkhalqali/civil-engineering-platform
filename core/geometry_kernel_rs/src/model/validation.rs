//! Model invariants.
//!
//! Two complementary mechanisms keep the model honest:
//!
//! * **on mutation** — [`super::EngineeringModel::add_element`] refuses an element
//!   whose references do not resolve while the model is in [`ValidationMode::Strict`];
//! * **on demand** — [`super::EngineeringModel::validate`] walks the whole model and
//!   reports every invariant it can check, including problems that a loaded or
//!   hand-edited file can contain.
//!
//! What is checked here is *model* consistency: identity, references, classification
//! and the physical sanity of dimensions. Engineering design checks (code
//! compliance, capacity, detailing) are a different layer and are explicitly out of
//! scope.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::elements::Element;
use crate::model::{CrossSection, EngineeringModel, Material, ProfileType, MODEL_SCHEMA_VERSION};

/// How a model treats references while it is being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ValidationMode {
    /// A mutation that would leave a dangling reference is rejected.
    #[default]
    Strict,
    /// A mutation is accepted and the problem is reported by
    /// [`EngineeringModel::validate`] instead, which is what loading or repairing
    /// partial data needs.
    Permissive,
}

/// The kinds of invariant problems a model can report.
///
/// The set is intentionally small and stable; messages carry the detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationCode {
    /// The same id is used by more than one entity of the model.
    DuplicateId,
    /// A collection holds an entity under a key that is not its own id.
    IdMismatch,
    /// An element's `category` disagrees with its concrete type.
    CategoryMismatch,
    /// An element points at an entity that is not in the model.
    MissingReference,
    /// A dimension is zero or negative where physics requires a positive value,
    /// or a linear element has no length.
    InvalidDimension,
    /// The levels referenced by a vertical element do not describe a usable extent.
    InvalidLevelOrder,
    /// A surface element's boundary is unusable (for example not horizontal).
    InvalidBoundary,
    /// A cross section description is internally inconsistent.
    InvalidCrossSection,
    /// A material property is negative.
    NegativeValue,
    /// The data was written with a newer schema version than this kernel knows.
    UnsupportedSchemaVersion,
}

impl ValidationCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            ValidationCode::DuplicateId => "duplicate_id",
            ValidationCode::IdMismatch => "id_mismatch",
            ValidationCode::CategoryMismatch => "category_mismatch",
            ValidationCode::MissingReference => "missing_reference",
            ValidationCode::InvalidDimension => "invalid_dimension",
            ValidationCode::InvalidLevelOrder => "invalid_level_order",
            ValidationCode::InvalidBoundary => "invalid_boundary",
            ValidationCode::InvalidCrossSection => "invalid_cross_section",
            ValidationCode::NegativeValue => "negative_value",
            ValidationCode::UnsupportedSchemaVersion => "unsupported_schema_version",
        }
    }
}

impl std::fmt::Display for ValidationCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reported invariant problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Kind of the problem.
    pub code: ValidationCode,
    /// Human readable explanation.
    pub message: String,
    /// Element the problem belongs to, when it belongs to one.
    pub element_id: Option<Uuid>,
}

/// Everything [`EngineeringModel::validate`] found.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    /// `true` when no invariant was violated.
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    /// All reported problems, in the order they were found.
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn len(&self) -> usize {
        self.issues.len()
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    /// The codes of all reported problems.
    pub fn codes(&self) -> Vec<ValidationCode> {
        self.issues.iter().map(|issue| issue.code).collect()
    }

    /// `true` when at least one problem with `code` was reported.
    pub fn contains(&self, code: ValidationCode) -> bool {
        self.issues.iter().any(|issue| issue.code == code)
    }

    fn push(&mut self, code: ValidationCode, element_id: Option<Uuid>, message: String) {
        self.issues.push(ValidationIssue {
            code,
            message,
            element_id,
        });
    }
}

impl EngineeringModel {
    /// Checks every invariant of the model and reports what it found.
    ///
    /// The model is never mutated and nothing is silently repaired.
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();

        if self.schema_version > MODEL_SCHEMA_VERSION {
            report.push(
                ValidationCode::UnsupportedSchemaVersion,
                None,
                format!(
                    "model schema version {} is newer than the supported version {}",
                    self.schema_version, MODEL_SCHEMA_VERSION
                ),
            );
        }

        self.validate_id_uniqueness(&mut report);
        self.validate_storage_keys(&mut report);

        for cross_section in self.cross_sections.values() {
            validate_cross_section(cross_section, &mut report);
        }
        for material in self.materials.values() {
            validate_material(material, &mut report);
        }
        for element in self.elements.values() {
            self.validate_element(element, &mut report);
        }

        report
    }

    /// Ids must be unique across the whole model, not only inside one collection.
    fn validate_id_uniqueness(&self, report: &mut ValidationReport) {
        let mut owners: BTreeMap<Uuid, Vec<&'static str>> = BTreeMap::new();

        for id in self.levels.keys() {
            owners.entry(*id).or_default().push("level");
        }
        for id in self.grids.keys() {
            owners.entry(*id).or_default().push("grid");
        }
        for id in self.materials.keys() {
            owners.entry(*id).or_default().push("material");
        }
        for id in self.cross_sections.keys() {
            owners.entry(*id).or_default().push("cross_section");
        }
        for id in self.elements.keys() {
            owners.entry(*id).or_default().push("element");
        }

        for (id, kinds) in owners {
            if kinds.len() > 1 {
                report.push(
                    ValidationCode::DuplicateId,
                    None,
                    format!("id {id} is used by several entities: {}", kinds.join(", ")),
                );
            }
        }
    }

    /// Every entity must be stored under its own id.
    fn validate_storage_keys(&self, report: &mut ValidationReport) {
        for (key, level) in &self.levels {
            if *key != level.id {
                report.push(
                    ValidationCode::IdMismatch,
                    None,
                    format!("level {} is stored under key {key}", level.id),
                );
            }
        }
        for (key, grid) in &self.grids {
            if *key != grid.id {
                report.push(
                    ValidationCode::IdMismatch,
                    None,
                    format!("grid {} is stored under key {key}", grid.id),
                );
            }
        }
        for (key, material) in &self.materials {
            if *key != material.id {
                report.push(
                    ValidationCode::IdMismatch,
                    None,
                    format!("material {} is stored under key {key}", material.id),
                );
            }
        }
        for (key, cross_section) in &self.cross_sections {
            if *key != cross_section.id {
                report.push(
                    ValidationCode::IdMismatch,
                    None,
                    format!(
                        "cross section {} is stored under key {key}",
                        cross_section.id
                    ),
                );
            }
        }
        for (key, element) in &self.elements {
            if *key != element.id() {
                report.push(
                    ValidationCode::IdMismatch,
                    Some(element.id()),
                    format!("element {} is stored under key {key}", element.id()),
                );
            }
        }
    }

    /// One element: classification, references, dimensions and level order.
    fn validate_element(&self, element: &Element, report: &mut ValidationReport) {
        let id = element.id();
        let category = element.category();

        if category != element.category_of_variant() {
            report.push(
                ValidationCode::CategoryMismatch,
                Some(id),
                format!(
                    "element {} is stored as a {} but declares the category {}",
                    element.name(),
                    element.category_of_variant(),
                    category
                ),
            );
        }

        for (reference, missing_id) in element.referenced_ids() {
            if !self.reference_resolves(reference, missing_id) {
                report.push(
                    ValidationCode::MissingReference,
                    Some(id),
                    format!(
                        "element {} references {reference} {missing_id}, which is not in the model",
                        element.name()
                    ),
                );
            }
        }

        match element {
            Element::Column(column) => {
                if let (Some(base), Some(top)) = (
                    self.level(column.base_level_id),
                    self.level(column.top_level_id),
                ) {
                    let (base_elevation, top_elevation) =
                        column.extent(base.elevation, top.elevation);
                    if top_elevation <= base_elevation {
                        report.push(
                            ValidationCode::InvalidLevelOrder,
                            Some(id),
                            format!(
                                "column {} spans {} -> {} m, which is not an upward extent",
                                column.base.name,
                                base_elevation.meters(),
                                top_elevation.meters()
                            ),
                        );
                    }
                }
            }
            Element::Beam(beam) => {
                if beam.is_degenerate() {
                    report.push(
                        ValidationCode::InvalidDimension,
                        Some(id),
                        format!("beam {} has a zero length axis", beam.base.name),
                    );
                }
            }
            Element::Slab(slab) => {
                validate_dimension(
                    slab.thickness.is_positive(),
                    &slab.base.name,
                    "slab thickness",
                    report,
                    id,
                );
                validate_boundary(&slab.boundary, &slab.base.name, "slab", report, id);
            }
            Element::Wall(wall) => {
                if wall.is_degenerate() {
                    report.push(
                        ValidationCode::InvalidDimension,
                        Some(id),
                        format!("wall {} has a zero length axis", wall.base.name),
                    );
                }
                validate_dimension(
                    wall.thickness.is_positive(),
                    &wall.base.name,
                    "wall thickness",
                    report,
                    id,
                );
                if let (Some(base), Some(top)) = (
                    self.level(wall.base_level_id),
                    self.level(wall.top_level_id),
                ) {
                    if top.elevation <= base.elevation {
                        report.push(
                            ValidationCode::InvalidLevelOrder,
                            Some(id),
                            format!(
                                "wall {} spans {} -> {} m, which is not an upward extent",
                                wall.base.name,
                                base.elevation.meters(),
                                top.elevation.meters()
                            ),
                        );
                    }
                }
            }
            Element::Foundation(foundation) => {
                validate_dimension(
                    foundation.thickness.is_positive(),
                    &foundation.base.name,
                    "foundation thickness",
                    report,
                    id,
                );
                validate_boundary(
                    &foundation.footprint,
                    &foundation.base.name,
                    "foundation",
                    report,
                    id,
                );
            }
        }
    }

    fn reference_resolves(&self, reference: crate::error::ReferenceKind, id: Uuid) -> bool {
        use crate::error::ReferenceKind;
        match reference {
            ReferenceKind::Level => self.levels.contains_key(&id),
            ReferenceKind::Grid => self.grids.contains_key(&id),
            ReferenceKind::Material => self.materials.contains_key(&id),
            ReferenceKind::CrossSection => self.cross_sections.contains_key(&id),
        }
    }
}

fn validate_dimension(
    is_positive: bool,
    element_name: &str,
    what: &str,
    report: &mut ValidationReport,
    element_id: Uuid,
) {
    if !is_positive {
        report.push(
            ValidationCode::InvalidDimension,
            Some(element_id),
            format!("{what} of {element_name} must be greater than zero"),
        );
    }
}

fn validate_boundary(
    boundary: &crate::elements::PlanBoundary,
    element_name: &str,
    element_kind: &str,
    report: &mut ValidationReport,
    element_id: Uuid,
) {
    if boundary.vertex_count() < crate::elements::PlanBoundary::MIN_VERTICES {
        report.push(
            ValidationCode::InvalidBoundary,
            Some(element_id),
            format!("{element_kind} {element_name} has an outline with too few vertices"),
        );
    } else if !boundary.is_horizontal() {
        report.push(
            ValidationCode::InvalidBoundary,
            Some(element_id),
            format!("{element_kind} {element_name} has a non horizontal outline"),
        );
    }
}

fn validate_cross_section(cross_section: &CrossSection, report: &mut ValidationReport) {
    let name = cross_section.name.as_str();

    match &cross_section.profile {
        ProfileType::Rectangular { width, depth } => {
            validate_section_dimension(width.is_positive(), name, "width", report);
            validate_section_dimension(depth.is_positive(), name, "depth", report);
        }
        ProfileType::Circular { radius } => {
            validate_section_dimension(radius.is_positive(), name, "radius", report);
        }
        ProfileType::IBeam(profile) => {
            let dimensions_positive = profile.depth.is_positive()
                && profile.flange_width.is_positive()
                && profile.web_thickness.is_positive()
                && profile.flange_thickness.is_positive();
            validate_section_dimension(dimensions_positive, name, "depth / flange / web", report);
            if dimensions_positive && !profile.is_valid() {
                report.push(
                    ValidationCode::InvalidCrossSection,
                    None,
                    format!(
                        "I section {name} is inconsistent: web thickness must be smaller than the flange width and the web depth ({:.3} m) must be positive",
                        profile.web_depth().meters()
                    ),
                );
            }
        }
    }
}

fn validate_section_dimension(
    is_positive: bool,
    section_name: &str,
    what: &str,
    report: &mut ValidationReport,
) {
    if !is_positive {
        report.push(
            ValidationCode::InvalidDimension,
            None,
            format!("{what} of cross section {section_name} must be greater than zero"),
        );
    }
}

fn validate_material(material: &Material, report: &mut ValidationReport) {
    let mut report_negative = |property: &str| {
        report.push(
            ValidationCode::NegativeValue,
            None,
            format!(
                "{property} of material {} must not be negative",
                material.name
            ),
        );
    };

    if material.density.is_negative() {
        report_negative("density");
    }
    if material.compressive_strength.is_negative() {
        report_negative("compressive strength");
    }
    if material.youngs_modulus.is_negative() {
        report_negative("Young's modulus");
    }
}
