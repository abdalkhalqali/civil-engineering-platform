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

use crate::model::{EngineeringModel, ModelSummary};

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
