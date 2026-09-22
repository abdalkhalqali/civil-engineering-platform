//! The browser boundary: the same [`ModelSession`] the Flutter client drives,
//! exported through `wasm-bindgen`.
//!
//! # Why this exists
//!
//! The web preview must show the *real* engineering model. Building a second model
//! in JavaScript would create a second source of truth, so instead the kernel is
//! compiled to WebAssembly and the browser drives it exactly like the native client:
//!
//! ```text
//! EngineeringModel → ModelCommand → ModelSession → (FRB | wasm-bindgen) → UI
//! ```
//!
//! The renderer in the browser receives [`crate::render::RenderData`] and owns
//! nothing: it draws what it is given and reports pointer events back as commands.
//!
//! Structured payloads are JSON strings so the command surface can grow without
//! changing this binding.

use wasm_bindgen::prelude::*;

use crate::session::{CommandRequest, ModelSession};

/// A modelling session, owned by the page.
#[wasm_bindgen]
pub struct WasmSession {
    inner: ModelSession,
}

#[wasm_bindgen]
impl WasmSession {
    /// Opens a new project with two levels, a grid, a concrete material and a column
    /// section, ready to be built in.
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, project_type: String, land_area_m2: f64) -> WasmSession {
        Self {
            inner: ModelSession::new(name, project_type, land_area_m2),
        }
    }

    /// The full session state, as JSON.
    pub fn state(&self) -> String {
        to_json(&self.inner.state())
    }

    /// Applies one command, given as a JSON [`CommandRequest`].
    pub fn execute(&mut self, request_json: &str) -> String {
        match serde_json::from_str::<CommandRequest>(request_json) {
            Ok(request) => to_json(&self.inner.execute_request(request)),
            Err(error) => failure(&format!("طلب غير صالح: {error}")),
        }
    }

    /// Takes the model back by one change.
    pub fn undo(&mut self) -> String {
        to_json(&self.inner.undo())
    }

    /// Re-applies the last undone change.
    pub fn redo(&mut self) -> String {
        to_json(&self.inner.redo())
    }

    /// Switches the level new elements are created on.
    pub fn set_active_level(&mut self, level_id: &str) -> String {
        match uuid::Uuid::parse_str(level_id) {
            Ok(id) if self.inner.set_active_level(id) => {
                to_json(&self.inner.state())
            }
            _ => failure("مستوى غير معروف"),
        }
    }

    /// Detail of one element, for the properties panel, as JSON.
    pub fn element_details(&self, element_id: &str) -> String {
        match uuid::Uuid::parse_str(element_id) {
            Ok(id) => self
                .inner
                .element_details(id)
                .map(|detail| to_json(&detail))
                .unwrap_or_else(|| failure("عنصر غير معروف")),
            Err(_) => failure("معرّف غير صالح"),
        }
    }

    /// Derived render data, as JSON.
    pub fn render_data(&self) -> String {
        to_json(&self.inner.render_data())
    }

    /// Finds the model point a pointer should land on, as JSON.
    pub fn snap(&self, x_m: f64, y_m: f64, z_m: f64, tolerance_m: f64) -> String {
        to_json(&self.inner.snap(x_m, y_m, tolerance_m, z_m))
    }

    /// The model revision, so a renderer can skip a redraw it does not need.
    pub fn revision(&self) -> f64 {
        self.inner.revision() as f64
    }

    /// Serializes the project to `.civilx` bytes.
    pub fn save(&self) -> Vec<u8> {
        self.inner.save().unwrap_or_default()
    }

    /// Replaces the session content with a `.civilx` payload.
    pub fn load(&mut self, data: &[u8]) -> String {
        match self.inner.load(data) {
            Ok(()) => self.state(),
            Err(message) => failure(&message),
        }
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| String::new())
}

fn failure(message: &str) -> String {
    serde_json::json!({ "ok": false, "message": message }).to_string()
}
