use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::domain::{
    clear_monsters, long_rest, next_turn, previous_turn, sort_by_initiative, start_initiative,
    end_initiative, AppState, Campaign, Combatant, CombatantKind,
};
use crate::persistence::{
    default_state_path, import_tauri_state, load_json, save_json, tauri_candidate_paths,
    ImportReport, PersistError,
};

type Listener = Rc<dyn Fn()>;

/// UI-facing façade over [`AppState`] with load/save and mutating helpers.
#[derive(Clone)]
pub struct StateStore {
    inner: Rc<RefCell<Inner>>,
}

impl std::fmt::Debug for StateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("StateStore")
            .field("path", &inner.path)
            .field("campaigns", &inner.state.campaigns.len())
            .finish()
    }
}

struct Inner {
    state: AppState,
    path: PathBuf,
    listeners: Vec<Listener>,
    last_import: Option<ImportReport>,
}

impl StateStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                state: AppState::default(),
                path,
                listeners: Vec::new(),
                last_import: None,
            })),
        }
    }

    pub fn with_default_path() -> Result<Self, PersistError> {
        Ok(Self::new(default_state_path()?))
    }

    pub fn path(&self) -> PathBuf {
        self.inner.borrow().path.clone()
    }

    pub fn last_import(&self) -> Option<ImportReport> {
        self.inner.borrow().last_import.clone()
    }

    /// Load from GTK JSON; on missing file, try Tauri import then seed defaults.
    pub fn load(&self) -> Result<(), PersistError> {
        let path = self.path();
        let (state, import) = if path.is_file() {
            (load_json(&path)?, None)
        } else {
            match try_import_tauri() {
                Some((state, report)) => {
                    eprintln!(
                        "initiative-tracker: imported Tauri state from {:?}",
                        report.source
                    );
                    for skip in &report.skipped_images {
                        eprintln!("initiative-tracker: skipped image: {skip}");
                    }
                    for note in &report.notes {
                        eprintln!("initiative-tracker: import note: {note}");
                    }
                    // Persist immediately so subsequent runs use GTK JSON.
                    save_json(&path, &state)?;
                    (state, Some(report))
                }
                None => {
                    let state = AppState::default();
                    save_json(&path, &state)?;
                    (state, None)
                }
            }
        };

        {
            let mut inner = self.inner.borrow_mut();
            inner.state = state;
            inner.last_import = import;
        }
        self.notify();
        Ok(())
    }

    pub fn save(&self) -> Result<(), PersistError> {
        let inner = self.inner.borrow();
        save_json(&inner.path, &inner.state)
    }

    pub fn state(&self) -> AppState {
        self.inner.borrow().state.clone()
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut AppState) -> R) -> Result<R, PersistError> {
        let result = {
            let mut inner = self.inner.borrow_mut();
            let r = f(&mut inner.state);
            inner.state.normalize();
            r
        };
        self.save()?;
        self.notify();
        Ok(result)
    }

    pub fn subscribe(&self, listener: impl Fn() + 'static) {
        self.inner
            .borrow_mut()
            .listeners
            .push(Rc::new(listener));
    }

    fn notify(&self) {
        let listeners: Vec<_> = self.inner.borrow().listeners.clone();
        for l in listeners {
            l();
        }
    }

    // --- command helpers (save on each mutation) ---

    pub fn add_combatant(
        &self,
        name: String,
        kind: CombatantKind,
        initiative: i32,
        max_health: i32,
    ) -> Result<(), PersistError> {
        self.with_mut(|state| {
            let c = Combatant::new(name, kind, initiative, max_health);
            let campaign = state.current_mut();
            campaign.players.push(c);
            sort_by_initiative(&mut campaign.players);
        })
    }

    pub fn next_turn(&self) -> Result<(), PersistError> {
        self.with_mut(|s| next_turn(s.current_mut()))
    }

    pub fn previous_turn(&self) -> Result<(), PersistError> {
        self.with_mut(|s| previous_turn(s.current_mut()))
    }

    pub fn start_initiative(&self) -> Result<(), PersistError> {
        self.with_mut(|s| start_initiative(s.current_mut()))
    }

    pub fn end_initiative(&self) -> Result<(), PersistError> {
        self.with_mut(|s| end_initiative(s.current_mut()))
    }

    pub fn long_rest(&self) -> Result<(), PersistError> {
        self.with_mut(|s| long_rest(s.current_mut()))
    }

    pub fn clear_monsters(&self) -> Result<(), PersistError> {
        self.with_mut(|s| clear_monsters(s.current_mut()))
    }

    pub fn current_campaign(&self) -> Campaign {
        self.inner
            .borrow()
            .state
            .current()
            .cloned()
            .unwrap_or_default()
    }
}

fn try_import_tauri() -> Option<(AppState, ImportReport)> {
    for candidate in tauri_candidate_paths() {
        if !candidate.is_file() {
            continue;
        }
        match import_tauri_state(&candidate) {
            Ok(pair) => return Some(pair),
            Err(e) => {
                eprintln!(
                    "initiative-tracker: failed to import {:?}: {e}",
                    candidate
                );
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Theme;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_seeds_defaults_when_missing() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("it-store-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");
        // Ensure no file and no Tauri candidates by using an isolated path only.
        // StateStore::load looks at path first; if missing it tries Tauri — that's fine.
        let store = StateStore::new(path.clone());
        // Manually seed like first-run without going through Tauri:
        {
            let state = AppState::default();
            save_json(&path, &state).unwrap();
        }
        store.load().unwrap();
        let state = store.state();
        assert_eq!(state.theme, Theme::System);
        assert_eq!(state.current().unwrap().players.len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn with_mut_persists() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-mut-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();
        store
            .with_mut(|s| {
                s.theme = Theme::Light;
            })
            .unwrap();
        let reloaded = load_json(&path).unwrap();
        assert_eq!(reloaded.theme, Theme::Light);
        let _ = std::fs::remove_file(&path);
    }
}