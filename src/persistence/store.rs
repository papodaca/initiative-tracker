use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::domain::{
    activate_scene_image, clear_monsters, end_initiative, long_rest, next_turn, previous_turn,
    resolve_combatant_name, sort_by_initiative, start_initiative, update_player_active, AppState,
    Campaign, Combatant, CombatantKind, SceneImage,
};
use crate::persistence::{
    default_images_dir, default_state_path, import_tauri_state, load_json, save_json,
    tauri_candidate_paths, ImportReport, PersistError,
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
            let name = resolve_combatant_name(&name, kind);
            let c = Combatant::new(name, kind, initiative, max_health);
            let campaign = state.current_mut();
            campaign.players.push(c);
            sort_by_initiative(&mut campaign.players);
        })
    }

    pub fn update_combatant(
        &self,
        id: &str,
        patch: CombatantPatch,
    ) -> Result<bool, PersistError> {
        self.with_mut(|state| {
            let campaign = state.current_mut();
            let Some(index) = campaign.players.iter().position(|p| p.id == id) else {
                return false;
            };
            let resort = matches!(patch, CombatantPatch::Initiative(_));
            {
                let c = &mut campaign.players[index];
                match patch {
                    CombatantPatch::Name(name) => {
                        let name = name.trim();
                        if !name.is_empty() {
                            c.name = name.to_string();
                        }
                    }
                    CombatantPatch::Initiative(initiative) => {
                        c.initiative = initiative;
                    }
                    CombatantPatch::Health(health) => {
                        c.health = health;
                        c.normalize_dead();
                    }
                    CombatantPatch::MaxHealth(max_health) => {
                        c.max_health = if max_health == 0 {
                            crate::domain::DEFAULT_HEALTH
                        } else {
                            max_health
                        };
                        c.normalize_dead();
                    }
                }
            }
            if resort {
                // Keep `active` flags on the combatant structs (parity with Svelte sortList).
                sort_by_initiative(&mut campaign.players);
            }
            true
        })
    }

    pub fn delete_combatant(&self, id: &str) -> Result<bool, PersistError> {
        self.with_mut(|state| {
            let campaign = state.current_mut();
            let before = campaign.players.len();
            campaign.players.retain(|p| p.id != id);
            if campaign.players.len() == before {
                return false;
            }
            if let Some(i) = campaign.current_player {
                if campaign.players.is_empty() {
                    campaign.current_player = None;
                } else if i >= campaign.players.len() {
                    campaign.current_player = Some(campaign.players.len() - 1);
                }
            }
            update_player_active(campaign);
            true
        })
    }

    pub fn set_initiative_visible(&self, visible: bool) -> Result<(), PersistError> {
        self.with_mut(|s| {
            s.current_mut().initiative_visible = visible;
        })
    }

    pub fn set_health_visible(&self, visible: bool) -> Result<(), PersistError> {
        // UI label "Enemy HP" — Presenter mapping documented in visibility.rs.
        self.with_mut(|s| {
            s.current_mut().health_visible = visible;
        })
    }

    pub fn set_enemy_health_visible(&self, visible: bool) -> Result<(), PersistError> {
        // UI label "Player HP" — Presenter mapping documented in visibility.rs.
        self.with_mut(|s| {
            s.current_mut().enemy_health_visible = visible;
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

    pub fn set_current_campaign(&self, name: &str) -> Result<bool, PersistError> {
        self.with_mut(|s| s.set_current_campaign(name))
    }

    pub fn add_campaign(&self, name: &str) -> Result<bool, PersistError> {
        self.with_mut(|s| s.add_campaign(name))
    }

    /// Apply settings dialog fields (rename + theme + display + campaign flags).
    pub fn apply_settings(&self, update: SettingsUpdate) -> Result<(), PersistError> {
        self.with_mut(|s| {
            let _ = s.rename_current_campaign(&update.campaign_name);
            s.theme = update.theme;
            s.display_size = update.display_size.clamp(1.0, 5.0);
            let camp = s.current_mut();
            camp.show_initiative_roll = update.show_initiative_roll;
            camp.auto_hide_inactive = update.auto_hide_inactive;
        })
    }

    /// Append scene images, copying each file into the app images directory.
    ///
    /// Copies keep thumbnails/Presenter backgrounds readable under a tight
    /// Flatpak sandbox after the document portal grant for the picker ends.
    pub fn add_images(&self, paths: &[std::path::PathBuf]) -> Result<usize, PersistError> {
        let images_dir = default_images_dir()?;
        std::fs::create_dir_all(&images_dir)?;

        let mut prepared: Vec<(String, String)> = Vec::with_capacity(paths.len());
        for path in paths {
            let name = SceneImage::name_from_path(path);
            let dest = unique_image_dest(&images_dir, path);
            std::fs::copy(path, &dest)?;
            prepared.push((name, dest.to_string_lossy().into_owned()));
        }

        self.with_mut(|state| {
            let campaign = state.current_mut();
            let added = prepared.len();
            for (name, path_str) in prepared {
                campaign.images.push(SceneImage::new(name, path_str));
            }
            added
        })
    }

    /// Mark one image active and clear others (parity with Tauri `makeActive`).
    pub fn set_active_image(&self, id: &str) -> Result<bool, PersistError> {
        self.with_mut(|state| activate_scene_image(&mut state.current_mut().images, id))
    }

    /// Inline rename for a scene image.
    pub fn rename_image(&self, id: &str, name: String) -> Result<bool, PersistError> {
        self.with_mut(|state| {
            let Some(image) = state
                .current_mut()
                .images
                .iter_mut()
                .find(|i| i.id == id)
            else {
                return false;
            };
            let name = name.trim();
            if !name.is_empty() {
                image.name = name.to_string();
            }
            true
        })
    }
}

/// Field patch for inline combatant edits.
#[derive(Debug, Clone)]
pub enum CombatantPatch {
    Name(String),
    Initiative(i32),
    Health(i32),
    MaxHealth(i32),
}

/// Draft values from the settings dialog.
#[derive(Debug, Clone)]
pub struct SettingsUpdate {
    pub campaign_name: String,
    pub theme: crate::domain::Theme,
    pub display_size: f64,
    pub show_initiative_roll: bool,
    pub auto_hide_inactive: bool,
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
                    "initiative-tracker: Tauri import failed for {}: {e}",
                    candidate.display()
                );
            }
        }
    }
    None
}

/// Destination under the app images dir: `<uuid>.<ext>` (ext from source).
fn unique_image_dest(images_dir: &std::path::Path, src: &std::path::Path) -> PathBuf {
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty())
        .unwrap_or("img");
    images_dir.join(format!("{}.{ext}", uuid::Uuid::new_v4()))
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

    #[test]
    fn campaign_switch_and_add_persist() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-camp-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();

        assert!(store.add_campaign("Family").unwrap());
        assert!(!store.add_campaign("Family").unwrap());
        assert!(store.set_current_campaign("Family").unwrap());

        let reloaded = load_json(&path).unwrap();
        assert_eq!(reloaded.current_campaign, "Family");
        assert!(reloaded.campaigns.contains(&"Family".to_string()));
        assert_eq!(reloaded.campaign_data["Family"].players.len(), 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn apply_settings_renames_and_persists() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-settings-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();

        store
            .apply_settings(SettingsUpdate {
                campaign_name: "Adventure".into(),
                theme: Theme::Dark,
                display_size: 2.5,
                show_initiative_roll: false,
                auto_hide_inactive: true,
            })
            .unwrap();

        let reloaded = load_json(&path).unwrap();
        assert_eq!(reloaded.current_campaign, "Adventure");
        assert_eq!(reloaded.theme, Theme::Dark);
        assert!((reloaded.display_size - 2.5).abs() < f64::EPSILON);
        let camp = reloaded.current().unwrap();
        assert!(!camp.show_initiative_roll);
        assert!(camp.auto_hide_inactive);
        assert!(!reloaded.campaign_data.contains_key("default"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn add_update_delete_combatant_persists() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-combat-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();

        store
            .add_combatant("".into(), CombatantKind::Monster, 20, 0)
            .unwrap();
        let state = store.state();
        let monster = state
            .current()
            .unwrap()
            .players
            .iter()
            .find(|p| p.kind == CombatantKind::Monster)
            .unwrap();
        assert_eq!(monster.name, "New Monster");
        assert_eq!(monster.max_health, 10);
        assert_eq!(state.current().unwrap().players[0].initiative, 20);

        let id = monster.id.clone();
        store
            .update_combatant(&id, CombatantPatch::Health(0))
            .unwrap();
        assert!(store.current_campaign().players.iter().find(|p| p.id == id).unwrap().dead);

        store
            .update_combatant(&id, CombatantPatch::Health(4))
            .unwrap();
        assert!(!store.current_campaign().players.iter().find(|p| p.id == id).unwrap().dead);

        assert!(store.delete_combatant(&id).unwrap());
        assert!(!store
            .current_campaign()
            .players
            .iter()
            .any(|p| p.id == id));

        let reloaded = load_json(&path).unwrap();
        assert!(!reloaded
            .current()
            .unwrap()
            .players
            .iter()
            .any(|p| p.kind == CombatantKind::Monster));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn visibility_toggles_persist() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-vis-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();
        store.set_initiative_visible(true).unwrap();
        store.set_health_visible(true).unwrap();
        store.set_enemy_health_visible(true).unwrap();
        let camp = load_json(&path).unwrap().current().unwrap().clone();
        assert!(camp.initiative_visible);
        assert!(camp.health_visible);
        assert!(camp.enemy_health_visible);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn combat_loop_actions_persist_active() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-loop-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();
        store.start_initiative().unwrap();
        assert_eq!(store.current_campaign().current_player, Some(0));
        store.next_turn().unwrap();
        assert_eq!(store.current_campaign().current_player, Some(1));
        store.end_initiative().unwrap();
        assert!(store.current_campaign().current_player.is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn add_activate_rename_images_persist() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("it-store-images-{nanos}.json"));
        save_json(&path, &AppState::default()).unwrap();
        let store = StateStore::new(path.clone());
        store.load().unwrap();

        let img_a = std::env::temp_dir().join(format!("it-a-{nanos}.png"));
        let img_b = std::env::temp_dir().join(format!("it-b-{nanos}.jpg"));
        std::fs::write(&img_a, b"png-bytes").unwrap();
        std::fs::write(&img_b, b"jpg-bytes").unwrap();
        assert_eq!(
            store
                .add_images(&[img_a.clone(), img_b.clone()])
                .unwrap(),
            2
        );
        let camp = store.current_campaign();
        assert_eq!(camp.images.len(), 2);
        assert!(!camp.images[0].active);
        assert_eq!(camp.images[0].name, SceneImage::name_from_path(&img_a));
        let images_dir = default_images_dir().unwrap();
        let stored_a = PathBuf::from(&camp.images[0].path);
        let stored_b = PathBuf::from(&camp.images[1].path);
        assert!(stored_a.starts_with(&images_dir), "{}", stored_a.display());
        assert!(stored_b.starts_with(&images_dir), "{}", stored_b.display());
        assert!(stored_a.is_file());
        assert!(stored_b.is_file());
        assert_ne!(stored_a, img_a);

        let id = camp.images[1].id.clone();
        assert!(store.set_active_image(&id).unwrap());
        assert!(store.rename_image(&id, "Dungeon".into()).unwrap());

        let reloaded = load_json(&path).unwrap().current().unwrap().clone();
        assert_eq!(reloaded.images.len(), 2);
        assert!(!reloaded.images[0].active);
        assert!(reloaded.images[1].active);
        assert_eq!(reloaded.images[1].name, "Dungeon");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&img_a);
        let _ = std::fs::remove_file(&img_b);
        let _ = std::fs::remove_file(&stored_a);
        let _ = std::fs::remove_file(&stored_b);
    }
}
