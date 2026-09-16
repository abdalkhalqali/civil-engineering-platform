use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::units::{MassDensity, Stress};

/// Classification of materials.
///
/// Deliberately a small, extensible set. Design codes, partial factors, stress
/// limits and constitutive laws are **not** part of it: those belong to a code and
/// analysis layer that comes later, and they must not change this model's meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum MaterialType {
    #[default]
    Concrete,
    Steel,
    Other,
}

impl MaterialType {
    pub const fn as_str(self) -> &'static str {
        match self {
            MaterialType::Concrete => "concrete",
            MaterialType::Steel => "steel",
            MaterialType::Other => "other",
        }
    }
}

impl std::fmt::Display for MaterialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A material of the model.
///
/// Every physical property is a typed quantity in internal SI units, so `C30`
/// concrete is stored as `compressive_strength = Stress::from_megapascals(30.0)`
/// (i.e. `30e6 Pa`) and steel stiffness as
/// `youngs_modulus = Stress::from_gigapascals(200.0)` — never as a bare number whose
/// unit is guessed from a field name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Stable identity of the material.
    pub id: Uuid,
    /// Human readable name, for example `Concrete C30`.
    pub name: String,
    /// Classification of the material.
    pub material_type: MaterialType,
    /// Mass density in kg/m³ (reinforced concrete ≈ 2500 kg/m³).
    pub density: MassDensity,
    /// Compressive strength in Pa.
    pub compressive_strength: Stress,
    /// Elastic (Young's) modulus in Pa.
    pub youngs_modulus: Stress,
}

impl Material {
    /// Creates a material with a freshly generated identity.
    pub fn new(
        name: impl Into<String>,
        material_type: MaterialType,
        density: MassDensity,
        compressive_strength: Stress,
        youngs_modulus: Stress,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            material_type,
            density,
            compressive_strength,
            youngs_modulus,
        }
    }

    /// Creates a material with an explicit identity.
    pub fn with_id(
        id: Uuid,
        name: impl Into<String>,
        material_type: MaterialType,
        density: MassDensity,
        compressive_strength: Stress,
        youngs_modulus: Stress,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            material_type,
            density,
            compressive_strength,
            youngs_modulus,
        }
    }

    /// `Concrete C30`: 2500 kg/m³, `fck = 30 MPa`, `E ≈ 33 GPa`.
    pub fn concrete_c30() -> Self {
        Self::new(
            "Concrete C30",
            MaterialType::Concrete,
            MassDensity::from_kilograms_per_cubic_meter(2_500.0),
            Stress::from_megapascals(30.0),
            Stress::from_gigapascals(33.0),
        )
    }

    /// `Steel S355`: 7850 kg/m³, `fy = 355 MPa`, `E = 200 GPa`.
    pub fn steel_s355() -> Self {
        Self::new(
            "Steel S355",
            MaterialType::Steel,
            MassDensity::from_kilograms_per_cubic_meter(7_850.0),
            Stress::from_megapascals(355.0),
            Stress::from_gigapascals(200.0),
        )
    }
}
