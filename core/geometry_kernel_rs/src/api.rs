//! Public boundary of the geometry kernel: `flutter_rust_bridge` (Flutter) and,
//! later, a WebAssembly binding for the browser.
//!
//! # What crosses the boundary
//!
//! Only *sessions*, *commands* and *derived data* cross:
//!
//! ```text
//! EngineeringModel  →  ModelCommand  →  ModelSession  →  boundary  →  UI
//! ```
//!
//! * The [`crate::model::EngineeringModel`] itself never crosses, and no engineering
//!   rule is re-implemented on the client.
//! * A **command** is the only way to change anything. The client sends a
//!   [`CommandRequest`] and receives a [`CommandResult`] that reports *what changed*
//!   — not a fresh copy of the whole model.
//! * A **renderer** receives [`crate::render::RenderData`], which is derived on
//!   demand, carries the element id of every primitive, and is never stored.
//!
//! # Sessions
//!
//! A session is an explicitly owned [`crate::session::ModelSession`]. The boundary
//! keeps a registry keyed by its id, so the client holds a real handle instead of
//! relying on a single hidden "current model". Structured payloads travel as JSON so
//! the command surface can grow without changing a single bridge signature.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use uuid::Uuid;

use crate::model::EngineeringModel;
use crate::model::summary::ModelSummary;
use crate::project::{Project, ProjectSummary};
use crate::session::{
    CommandRequest, CommandResult, ModelSession, SessionState, SnapResult,
};

/// Result of opening or loading a session.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenSessionResult {
    pub ok: bool,
    pub message: String,
    pub session_id: String,
    /// The full session state as JSON, so one round trip is enough to start working.
    pub state_json: String,
}

fn sessions() -> &'static Mutex<BTreeMap<Uuid, ModelSession>> {
    static SESSIONS: OnceLock<Mutex<BTreeMap<Uuid, ModelSession>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Called once by `RustLib.init()` on the Dart side.
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities (logging / panic reporting).
    flutter_rust_bridge::setup_default_user_utils();
}

/// M0 proof of concept, kept working verbatim.
#[flutter_rust_bridge::frb(sync)]
pub fn get_kernel_status() -> String {
    "Engineering Geometry Kernel (Rust) is connected successfully!".to_string()
}

/// Creates an empty engineering model in Rust and returns its summary.
#[flutter_rust_bridge::frb(sync)]
pub fn create_empty_model() -> ModelSummary {
    EngineeringModel::new().summary()
}

/// Creates a new empty project and returns its summary.
#[flutter_rust_bridge::frb(sync)]
pub fn create_project(name: String) -> ProjectSummary {
    Project::new(name).summary()
}

// ------------------------------------------------------------------- sessions

/// Opens a modelling session: a real project with two levels, a grid, a concrete
/// material and a column section, ready to be built in.
#[flutter_rust_bridge::frb(sync)]
pub fn open_session(name: String, project_type: String, land_area_m2: f64) -> OpenSessionResult {
    let session = ModelSession::new(name, project_type, land_area_m2);
    let session_id = session.project_id();
    let state = session.state();
    let state_json = to_json(&state);
    sessions()
        .lock()
        .expect("session registry lock is not poisoned")
        .insert(session_id, session);
    OpenSessionResult {
        ok: true,
        message: String::new(),
        session_id: session_id.to_string(),
        state_json,
    }
}

/// Reads a `.civilx` payload and opens it as a new session.
///
/// The engineering model is rebuilt from the file; the viewport is regenerated from
/// the rebuilt model by the client.
#[flutter_rust_bridge::frb(sync)]
pub fn load_session(data: Vec<u8>) -> OpenSessionResult {
    let mut session = ModelSession::new("", "", 0.0);
    match session.load(&data) {
        Ok(()) => {
            let session_id = session.project_id();
            let state_json = to_json(&session.state());
            sessions()
                .lock()
                .expect("session registry lock is not poisoned")
                .insert(session_id, session);
            OpenSessionResult {
                ok: true,
                message: String::new(),
                session_id: session_id.to_string(),
                state_json,
            }
        }
        Err(message) => OpenSessionResult {
            ok: false,
            message,
            session_id: String::new(),
            state_json: String::new(),
        },
    }
}

/// Closes a session and releases its model.
#[flutter_rust_bridge::frb(sync)]
pub fn close_session(session_id: String) -> bool {
    parse_id(&session_id)
        .map(|id| {
            sessions()
                .lock()
                .expect("session registry lock is not poisoned")
                .remove(&id)
                .is_some()
        })
        .unwrap_or(false)
}

/// The full state of a session, as JSON.
#[flutter_rust_bridge::frb(sync)]
pub fn session_state(session_id: String) -> String {
    with_session(&session_id, |session| to_json(&session.state())).unwrap_or_default()
}

/// Applies one engineering command to the model.
#[flutter_rust_bridge::frb(sync)]
pub fn execute_command(session_id: String, request: CommandRequest) -> CommandResult {
    with_session(&session_id, |session| session.execute_request(request)).unwrap_or_else(|| {
        CommandResult {
            ok: false,
            message: "جلسة غير معروفة".to_string(),
            label: String::new(),
            revision: 0,
            changed_ids: Vec::new(),
            can_undo: false,
            can_redo: false,
        }
    })
}

/// Takes the model back by one change.
#[flutter_rust_bridge::frb(sync)]
pub fn undo_command(session_id: String) -> CommandResult {
    with_session(&session_id, ModelSession::undo).unwrap_or_else(unknown_session)
}

/// Re-applies the last undone change.
#[flutter_rust_bridge::frb(sync)]
pub fn redo_command(session_id: String) -> CommandResult {
    with_session(&session_id, ModelSession::redo).unwrap_or_else(unknown_session)
}

/// Switches the level new elements are created on.
#[flutter_rust_bridge::frb(sync)]
pub fn set_active_level(session_id: String, level_id: String) -> CommandResult {
    with_session(&session_id, |session| {
        let level_id = Uuid::parse_str(&level_id).unwrap_or_else(|_| Uuid::nil());
        if session.set_active_level(level_id) {
            CommandResult {
                ok: true,
                message: String::new(),
                label: "تغيير المستوى النشط".to_string(),
                revision: session.revision(),
                changed_ids: Vec::new(),
                can_undo: session.state().can_undo,
                can_redo: session.state().can_redo,
            }
        } else {
            CommandResult {
                ok: false,
                message: "مستوى غير معروف".to_string(),
                label: String::new(),
                revision: session.revision(),
                changed_ids: Vec::new(),
                can_undo: false,
                can_redo: false,
            }
        }
    })
    .unwrap_or_else(unknown_session)
}

/// Detail of one element, for the properties panel, as JSON.
#[flutter_rust_bridge::frb(sync)]
pub fn element_details(session_id: String, element_id: String) -> String {
    with_session(&session_id, |session| {
        let Ok(id) = Uuid::parse_str(&element_id) else {
            return String::new();
        };
        session
            .element_details(id)
            .map(|detail| to_json(&detail))
            .unwrap_or_default()
    })
    .unwrap_or_default()
}

/// Derived render data, as JSON. The renderer owns nothing here.
#[flutter_rust_bridge::frb(sync)]
pub fn render_data(session_id: String) -> String {
    with_session(&session_id, |session| to_json(&session.render_data())).unwrap_or_default()
}

/// Finds the model point a touch, a pen or a mouse should land on.
#[flutter_rust_bridge::frb(sync)]
pub fn snap_point(
    session_id: String,
    x_m: f64,
    y_m: f64,
    z_m: f64,
    tolerance_m: f64,
) -> SnapResult {
    with_session(&session_id, |session| session.snap(x_m, y_m, tolerance_m, z_m)).unwrap_or_else(
        || SnapResult {
            snapped: false,
            kind: "free".to_string(),
            label: String::new(),
            x_m,
            y_m,
            z_m,
            candidates: Vec::new(),
        },
    )
}

/// Serializes a session to `.civilx` bytes.
#[flutter_rust_bridge::frb(sync)]
pub fn save_session(session_id: String) -> Vec<u8> {
    with_session(&session_id, |session| session.save().unwrap_or_default()).unwrap_or_default()
}

fn unknown_session() -> CommandResult {
    CommandResult {
        ok: false,
        message: "جلسة غير معروفة".to_string(),
        label: String::new(),
        revision: 0,
        changed_ids: Vec::new(),
        can_undo: false,
        can_redo: false,
    }
}

fn with_session<T>(session_id: &str, action: impl FnOnce(&mut ModelSession) -> T) -> Option<T> {
    let id = parse_id(session_id)?;
    let mut registry = sessions()
        .lock()
        .expect("session registry lock is not poisoned");
    registry.get_mut(&id).map(action)
}

fn parse_id(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok()
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| String::new())
}

/// The session state type, re-exported for the bridge.
pub type SessionStateDto = SessionState;
