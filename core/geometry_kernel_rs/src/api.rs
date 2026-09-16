//! Public Rust API of the geometry kernel, exposed to Flutter through
//! `flutter_rust_bridge`.
//!
//! Scope of this file (M0): communication proof of concept only.
//! No geometry, CAD, BIM or structural-analysis logic belongs here yet.

/// Called once by `RustLib.init()` on the Dart side.
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities (logging / panic reporting).
    flutter_rust_bridge::setup_default_user_utils();
}

/// Proof of concept: returns a status string that Flutter displays on screen.
#[flutter_rust_bridge::frb(sync)]
pub fn get_kernel_status() -> String {
    "Engineering Geometry Kernel (Rust) is connected successfully!".to_string()
}
