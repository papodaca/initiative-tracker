use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use directories::BaseDirs;

use crate::domain::{AppState, APP_ID};

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("could not resolve XDG data directory for {APP_ID}")]
    NoDataDir,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// `$XDG_DATA_HOME/im.apodaca.InitiativeTracker/state.json`
pub fn default_state_path() -> Result<PathBuf, PersistError> {
    let base = BaseDirs::new().ok_or(PersistError::NoDataDir)?;
    Ok(base.data_dir().join(APP_ID).join("state.json"))
}

/// `$XDG_DATA_HOME/im.apodaca.InitiativeTracker/images`
///
/// Scene images selected via the file portal are copied here so Flatpak does
/// not need broad home/Pictures filesystem overrides.
pub fn default_images_dir() -> Result<PathBuf, PersistError> {
    let base = BaseDirs::new().ok_or(PersistError::NoDataDir)?;
    Ok(base.data_dir().join(APP_ID).join("images"))
}

pub fn load_json(path: &Path) -> Result<AppState, PersistError> {
    let text = fs::read_to_string(path)?;
    let mut state: AppState = serde_json::from_str(&text)?;
    state.normalize();
    Ok(state)
}

/// Atomic write: temp file in the same directory + rename.
pub fn save_json(path: &Path, state: &AppState) -> Result<(), PersistError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        let text = serde_json::to_string_pretty(state)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Theme;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("initiative-tracker-test-{name}-{nanos}.json"))
    }

    #[test]
    fn round_trip_json() {
        let path = temp_path("roundtrip");
        let mut state = AppState::default();
        state.theme = Theme::Dark;
        state.display_size = 1.5;
        save_json(&path, &state).unwrap();
        let loaded = load_json(&path).unwrap();
        assert_eq!(loaded.theme, Theme::Dark);
        assert!((loaded.display_size - 1.5).abs() < f64::EPSILON);
        assert_eq!(loaded.current_campaign, "default");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn default_path_uses_app_id_segment() {
        let path = default_state_path().unwrap();
        let s = path.to_string_lossy();
        assert!(
            s.contains("im.apodaca.InitiativeTracker"),
            "unexpected path: {s}"
        );
        assert!(s.ends_with("state.json"));
    }

    #[test]
    fn default_images_dir_uses_app_id_segment() {
        let path = default_images_dir().unwrap();
        let s = path.to_string_lossy();
        assert!(
            s.contains("im.apodaca.InitiativeTracker"),
            "unexpected path: {s}"
        );
        assert!(s.ends_with("images"));
    }
}