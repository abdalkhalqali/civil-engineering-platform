use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::units::{Area, Length, SecondMomentOfArea};

/// Parameters of a doubly symmetric I section.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IBeamProfile {
    /// Total depth of the section (local `y` direction).
    pub depth: Length,
    /// Width of each flange (local `x` direction).
    pub flange_width: Length,
    /// Thickness of the web.
    pub web_thickness: Length,
    /// Thickness of each flange.
    pub flange_thickness: Length,
}

impl IBeamProfile {
    pub const fn new(
        depth: Length,
        flange_width: Length,
        web_thickness: Length,
        flange_thickness: Length,
    ) -> Self {
        Self {
            depth,
            flange_width,
            web_thickness,
            flange_thickness,
        }
    }

    /// Clear depth of the web, between the two flanges.
    pub fn web_depth(&self) -> Length {
        self.depth - self.flange_thickness * 2.0
    }

    /// Basic geometric validity of the parameter set.
    pub fn is_valid(&self) -> bool {
        self.depth.is_positive()
            && self.flange_width.is_positive()
            && self.web_thickness.is_positive()
            && self.flange_thickness.is_positive()
            && self.web_thickness < self.flange_width
            && self.web_depth().is_positive()
    }
}

/// The shape of a cross section, expressed on the section's local axes.
///
/// The section plane is the element's local `x`/`y` plane; `x` is the local
/// horizontal axis and `y` the local depth axis, so bending "about x" uses the
/// depth along `y`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProfileType {
    /// Solid rectangle: `width` along local `x`, `depth` along local `y`.
    Rectangular { width: Length, depth: Length },
    /// Solid circle of the given radius.
    Circular { radius: Length },
    /// Doubly symmetric I section.
    IBeam(IBeamProfile),
}

impl ProfileType {
    /// Area of the profile, computed from its parameters.
    pub fn area(&self) -> Area {
        match self {
            ProfileType::Rectangular { width, depth } => {
                Area::from_square_meters(width.meters() * depth.meters())
            }
            ProfileType::Circular { radius } => {
                Area::from_square_meters(std::f64::consts::PI * radius.meters().powi(2))
            }
            ProfileType::IBeam(profile) => Area::from_square_meters(profile_area_m2(profile)),
        }
    }

    /// Second moments of area `(Ix, Iy)` about the centroidal axes.
    pub fn moment_of_inertia(&self) -> (SecondMomentOfArea, SecondMomentOfArea) {
        match self {
            ProfileType::Rectangular { width, depth } => {
                let w = width.meters();
                let d = depth.meters();
                (
                    SecondMomentOfArea::from_meters_to_the_fourth(w * d.powi(3) / 12.0),
                    SecondMomentOfArea::from_meters_to_the_fourth(d * w.powi(3) / 12.0),
                )
            }
            ProfileType::Circular { radius } => {
                let r = radius.meters();
                let value = std::f64::consts::PI * r.powi(4) / 4.0;
                (
                    SecondMomentOfArea::from_meters_to_the_fourth(value),
                    SecondMomentOfArea::from_meters_to_the_fourth(value),
                )
            }
            ProfileType::IBeam(profile) => {
                let (ix, iy) = i_section_moments_m4(profile);
                (
                    SecondMomentOfArea::from_meters_to_the_fourth(ix),
                    SecondMomentOfArea::from_meters_to_the_fourth(iy),
                )
            }
        }
    }

    /// Overall depth of the section along the local `y` axis.
    pub fn depth(&self) -> Length {
        match self {
            ProfileType::Rectangular { depth, .. } => *depth,
            ProfileType::Circular { radius } => *radius * 2.0,
            ProfileType::IBeam(profile) => profile.depth,
        }
    }

    /// Overall width of the section along the local `x` axis.
    pub fn width(&self) -> Length {
        match self {
            ProfileType::Rectangular { width, .. } => *width,
            ProfileType::Circular { radius } => *radius * 2.0,
            ProfileType::IBeam(profile) => profile.flange_width,
        }
    }
}

/// Area of a symmetric I section, in m².
fn profile_area_m2(profile: &IBeamProfile) -> f64 {
    let web = profile.web_depth().meters() * profile.web_thickness.meters();
    let flanges = 2.0 * profile.flange_width.meters() * profile.flange_thickness.meters();
    web + flanges
}

/// `(Ix, Iy)` of a symmetric I section in m⁴.
///
/// * `Ix` (bending about the local `x` axis) = web + two flanges about the mid-depth axis.
/// * `Iy` (bending about the local `y` axis) = flanges + web about the mid-width axis.
fn i_section_moments_m4(profile: &IBeamProfile) -> (f64, f64) {
    let depth = profile.depth.meters();
    let flange_width = profile.flange_width.meters();
    let web_thickness = profile.web_thickness.meters();
    let flange_thickness = profile.flange_thickness.meters();
    let web_depth = profile.web_depth().meters();

    let web_ix = web_thickness * web_depth.powi(3) / 12.0;
    let arm = (depth - flange_thickness) / 2.0;
    let flange_ix = flange_width * flange_thickness.powi(3) / 12.0
        + flange_width * flange_thickness * arm.powi(2);
    let ix = web_ix + 2.0 * flange_ix;

    let flange_iy = flange_thickness * flange_width.powi(3) / 12.0;
    let web_iy = web_depth * web_thickness.powi(3) / 12.0;
    let iy = 2.0 * flange_iy + web_iy;

    (ix, iy)
}

/// A reusable section definition of the model.
///
/// # Properties are derived, never stored
///
/// [`CrossSection::area`] and [`CrossSection::moment_of_inertia`] are computed from
/// [`CrossSection::profile`] on demand. Storing them as fields next to the profile
/// would create a second source of truth that could silently disagree with the
/// shape (edit the profile, forget the area). A profile is the only truth; the
/// quantities are functions of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossSection {
    /// Stable identity of the section.
    pub id: Uuid,
    /// Human readable name, for example `400x400`.
    pub name: String,
    /// Shape of the section.
    pub profile: ProfileType,
}

impl CrossSection {
    /// Creates a section with a freshly generated identity.
    pub fn new(name: impl Into<String>, profile: ProfileType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            profile,
        }
    }

    /// Creates a section with an explicit identity.
    pub fn with_id(id: Uuid, name: impl Into<String>, profile: ProfileType) -> Self {
        Self {
            id,
            name: name.into(),
            profile,
        }
    }

    /// A rectangular section, for example `400 mm × 400 mm` written as `400x400`.
    pub fn rectangular(name: impl Into<String>, width: Length, depth: Length) -> Self {
        Self::new(name, ProfileType::Rectangular { width, depth })
    }

    /// A circular section.
    pub fn circular(name: impl Into<String>, radius: Length) -> Self {
        Self::new(name, ProfileType::Circular { radius })
    }

    /// Area of the section, in m².
    pub fn area(&self) -> Area {
        self.profile.area()
    }

    /// Second moments of area `(Ix, Iy)`, in m⁴.
    pub fn moment_of_inertia(&self) -> (SecondMomentOfArea, SecondMomentOfArea) {
        self.profile.moment_of_inertia()
    }

    /// Overall depth of the section.
    pub fn depth(&self) -> Length {
        self.profile.depth()
    }

    /// Overall width of the section.
    pub fn width(&self) -> Length {
        self.profile.width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangular_400x400_properties() {
        let section = CrossSection::rectangular(
            "400x400",
            Length::from_millimeters(400.0),
            Length::from_millimeters(400.0),
        );

        // 0.4 * 0.4 is not exact in binary, so compare with a tolerance.
        assert!((section.area().square_meters() - 0.16).abs() < 1e-15);
        let (ix, iy) = section.moment_of_inertia();
        assert!((ix.meters_to_the_fourth() - 0.0021333333333333334).abs() < 1e-12);
        assert_eq!(ix, iy);
    }
}
