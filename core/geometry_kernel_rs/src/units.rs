//! Internal unit system of the kernel.
//!
//! # Rule: the kernel stores SI values only
//!
//! ```text
//! User input (mm, MPa, degrees, ...)
//!         │  conversion at the boundary
//!         ▼
//! Internal SI representation (m, Pa, radian, ...)
//!         │
//!         ▼
//! Engineering Model
//! ```
//!
//! A model value such as a 400 mm column side is stored as `Length::from_meters(0.40)`,
//! never as a bare `400` whose unit is guesswork. Elements therefore never carry a
//! `unit: String` field: the *type* carries the unit.
//!
//! | Quantity              | Internal unit | Type                  |
//! |-----------------------|---------------|-----------------------|
//! | Length                | metre (m)     | [`Length`]            |
//! | Area                  | m²            | [`Area`]              |
//! | Second moment of area | m⁴            | [`SecondMomentOfArea`]|
//! | Force                 | newton (N)    | [`Force`]             |
//! | Mass                  | kilogram (kg) | [`Mass`]              |
//! | Mass density          | kg/m³         | [`MassDensity`]       |
//! | Angle                 | radian (rad)  | [`Angle`]             |
//! | Stress / elastic modulus | pascal (Pa) | [`Stress`]           |
//!
//! Adding a quantity means adding one `define_unit!` invocation plus its
//! conversion helpers — no element type has to change.

use serde::{Deserialize, Serialize};

/// Defines a strongly typed physical quantity stored in its internal SI unit.
///
/// The generated type is a `serde`-transparent newtype (it serializes as a plain
/// number) with the arithmetic that makes sense for a linear quantity.
macro_rules! define_unit {
    ($(#[$attr:meta])* $name:ident) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(f64);

        impl $name {
            /// Zero of this quantity.
            pub const ZERO: Self = Self(0.0);

            /// Builds a value that is already expressed in the internal SI unit.
            ///
            /// Prefer the explicit `from_*` constructors when the caller works in
            /// another unit (millimetres, megapascals, degrees, ...).
            pub const fn new(value_in_si: f64) -> Self {
                Self(value_in_si)
            }

            /// The value in the internal SI unit.
            pub const fn value(self) -> f64 {
                self.0
            }

            /// Absolute value, keeping the unit.
            pub fn abs(self) -> Self {
                Self(self.0.abs())
            }

            pub fn is_zero(self) -> bool {
                self.0 == 0.0
            }

            pub fn is_positive(self) -> bool {
                self.0 > 0.0
            }

            pub fn is_negative(self) -> bool {
                self.0 < 0.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl std::ops::Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl std::ops::Mul<f64> for $name {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }

        impl std::ops::Div<f64> for $name {
            type Output = Self;
            fn div(self, rhs: f64) -> Self {
                Self(self.0 / rhs)
            }
        }

        impl std::ops::AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl std::ops::SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl From<f64> for $name {
            /// Interprets the raw number as the internal SI unit.
            fn from(value: f64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for f64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

define_unit!(
    /// Length, stored in **metres** (m).
    Length
);
define_unit!(
    /// Area, stored in **square metres** (m²).
    Area
);
define_unit!(
    /// Second moment of area (moment of inertia of a section), stored in **m⁴**.
    SecondMomentOfArea
);
define_unit!(
    /// Force, stored in **newtons** (N).
    Force
);
define_unit!(
    /// Mass, stored in **kilograms** (kg).
    Mass
);
define_unit!(
    /// Mass density, stored in **kilograms per cubic metre** (kg/m³).
    MassDensity
);
define_unit!(
    /// Plane angle, stored in **radians** (rad).
    Angle
);
define_unit!(
    /// Stress and elastic modulus, stored in **pascals** (Pa).
    Stress
);

impl Length {
    /// Length in metres (internal unit).
    pub const fn from_meters(meters: f64) -> Self {
        Self(meters)
    }

    /// Length in millimetres, for example `Length::from_millimeters(400.0)` → `0.4 m`.
    pub fn from_millimeters(millimeters: f64) -> Self {
        Self(millimeters / 1_000.0)
    }

    /// Length in centimetres.
    pub fn from_centimeters(centimeters: f64) -> Self {
        Self(centimeters / 100.0)
    }

    /// Length in kilometres.
    pub fn from_kilometers(kilometers: f64) -> Self {
        Self(kilometers * 1_000.0)
    }

    pub const fn meters(self) -> f64 {
        self.0
    }

    pub fn millimeters(self) -> f64 {
        self.0 * 1_000.0
    }

    pub fn centimeters(self) -> f64 {
        self.0 * 100.0
    }
}

impl Area {
    pub const fn from_square_meters(square_meters: f64) -> Self {
        Self(square_meters)
    }

    /// Area from mm², for example `160_000 mm²` → `0.16 m²`.
    pub fn from_square_millimeters(square_millimeters: f64) -> Self {
        Self(square_millimeters / 1_000_000.0)
    }

    pub const fn square_meters(self) -> f64 {
        self.0
    }

    pub fn square_millimeters(self) -> f64 {
        self.0 * 1_000_000.0
    }
}

impl SecondMomentOfArea {
    pub const fn from_meters_to_the_fourth(m4: f64) -> Self {
        Self(m4)
    }

    /// Second moment of area from mm⁴.
    pub fn from_millimeters_to_the_fourth(mm4: f64) -> Self {
        Self(mm4 / 1e12)
    }

    pub const fn meters_to_the_fourth(self) -> f64 {
        self.0
    }

    pub fn millimeters_to_the_fourth(self) -> f64 {
        self.0 * 1e12
    }
}

impl Force {
    pub const fn from_newtons(newtons: f64) -> Self {
        Self(newtons)
    }

    /// Force from kilonewtons, for example `120 kN` → `120_000 N`.
    pub fn from_kilonewtons(kilonewtons: f64) -> Self {
        Self(kilonewtons * 1_000.0)
    }

    pub const fn newtons(self) -> f64 {
        self.0
    }

    pub fn kilonewtons(self) -> f64 {
        self.0 / 1_000.0
    }
}

impl Mass {
    pub const fn from_kilograms(kilograms: f64) -> Self {
        Self(kilograms)
    }

    /// Mass from metric tonnes, for example `2.5 t` → `2_500 kg`.
    pub fn from_tonnes(tonnes: f64) -> Self {
        Self(tonnes * 1_000.0)
    }

    pub const fn kilograms(self) -> f64 {
        self.0
    }

    pub fn tonnes(self) -> f64 {
        self.0 / 1_000.0
    }
}

impl MassDensity {
    pub const fn from_kilograms_per_cubic_meter(kilograms_per_cubic_meter: f64) -> Self {
        Self(kilograms_per_cubic_meter)
    }

    /// Density from tonnes per cubic metre (same numeric value as g/cm³),
    /// for example reinforced concrete `2.5 t/m³` → `2500 kg/m³`.
    pub fn from_tonnes_per_cubic_meter(tonnes_per_cubic_meter: f64) -> Self {
        Self(tonnes_per_cubic_meter * 1_000.0)
    }

    pub const fn kilograms_per_cubic_meter(self) -> f64 {
        self.0
    }

    pub fn tonnes_per_cubic_meter(self) -> f64 {
        self.0 / 1_000.0
    }
}

impl Angle {
    pub const fn from_radians(radians: f64) -> Self {
        Self(radians)
    }

    /// Angle from degrees, for example `30°` → `0.5236 rad`.
    pub fn from_degrees(degrees: f64) -> Self {
        Self(degrees.to_radians())
    }

    pub const fn radians(self) -> f64 {
        self.0
    }

    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }
}

impl Stress {
    pub const fn from_pascals(pascals: f64) -> Self {
        Self(pascals)
    }

    /// Stress from kilopascals.
    pub fn from_kilopascals(kilopascals: f64) -> Self {
        Self(kilopascals * 1_000.0)
    }

    /// Stress from megapascals, for example concrete `C30` → `30 MPa` = `30e6 Pa`.
    pub fn from_megapascals(megapascals: f64) -> Self {
        Self(megapascals * 1_000_000.0)
    }

    /// Stress from gigapascals, for example steel `E` → `200 GPa` = `200e9 Pa`.
    pub fn from_gigapascals(gigapascals: f64) -> Self {
        Self(gigapascals * 1_000_000_000.0)
    }

    pub const fn pascals(self) -> f64 {
        self.0
    }

    pub fn kilopascals(self) -> f64 {
        self.0 / 1_000.0
    }

    pub fn megapascals(self) -> f64 {
        self.0 / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millimetres_are_converted_to_metres() {
        // 400 mm must never be stored as the bare number 400.
        assert_eq!(Length::from_millimeters(400.0).meters(), 0.4);
    }

    #[test]
    fn stress_and_angle_convert_to_si() {
        assert_eq!(Stress::from_megapascals(30.0).pascals(), 30_000_000.0);
        assert!((Angle::from_degrees(180.0).radians() - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn arithmetic_keeps_the_unit() {
        let total = Length::from_meters(1.5) + Length::from_millimeters(500.0);
        assert_eq!(total.meters(), 2.0);
        assert_eq!((total * 2.0).meters(), 4.0);
    }
}
