//! JSON persistence under XDG and best-effort Tauri store import.
//!
//! # Import limitations
//!
//! The Tauri app owns `.settings.dat` (JSON with a top-level `"state"` key).
//! GTK never writes back to that file.
//!
//! Scene images stored as Tauri `asset://localhost/…` URLs are recovered when the
//! path can be percent-decoded to an absolute filesystem path. Opaque or
//! non-`asset://` URLs are skipped (logged). Numeric fields that InPlaceEdit
//! persisted as strings are accepted.

mod json_store;
mod store;
mod tauri_import;

pub use json_store::{default_state_path, load_json, save_json, PersistError};
pub use store::{SettingsUpdate, StateStore};
pub use tauri_import::{import_tauri_state, tauri_candidate_paths, ImportReport};