use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

pub const DEFAULT_HEALTH: i32 = 10;
pub const DEFAULT_CAMPAIGN_NAME: &str = "default";
pub const APP_ID: &str = "im.apodaca.InitiativeTracker";

/// Combatant kind; missing / unknown values deserialize as [`CombatantKind::Player`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CombatantKind {
    #[default]
    Player,
    Npc,
    Monster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Combatant {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub kind: CombatantKind,
    #[serde(deserialize_with = "deserialize_i32_lenient")]
    pub initiative: i32,
    #[serde(deserialize_with = "deserialize_i32_lenient")]
    pub health: i32,
    #[serde(deserialize_with = "deserialize_i32_lenient")]
    pub max_health: i32,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub dead: bool,
}

impl Combatant {
    pub fn new(name: impl Into<String>, kind: CombatantKind, initiative: i32, max_health: i32) -> Self {
        let max_health = if max_health == 0 {
            DEFAULT_HEALTH
        } else {
            max_health
        };
        let health = max_health;
        let mut c = Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            kind,
            initiative,
            health,
            max_health,
            active: false,
            dead: false,
        };
        c.normalize_dead();
        c
    }

    /// Dead is derived from HP (`health <= 0`), matching Svelte.
    pub fn normalize_dead(&mut self) -> bool {
        let dead = self.health <= 0;
        let changed = self.dead != dead;
        self.dead = dead;
        changed
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneImage {
    pub id: String,
    pub name: String,
    /// Filesystem path (absolute or portable). Opaque Tauri `asset://` URLs are dropped on import.
    pub path: String,
    #[serde(default)]
    pub active: bool,
}

impl SceneImage {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            path: path.into(),
            active: false,
        }
    }

    /// Default display name = file stem (parity with Tauri `ImageList`).
    pub fn name_from_path(path: &std::path::Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Image")
            .to_string()
    }
}

/// First image marked `active`, if any.
pub fn active_scene_image(campaign: &Campaign) -> Option<&SceneImage> {
    campaign.images.iter().find(|i| i.active)
}

/// Set exactly one image active by id (clears others). Returns false if id missing.
pub fn activate_scene_image(images: &mut [SceneImage], id: &str) -> bool {
    if !images.iter().any(|i| i.id == id) {
        return false;
    }
    for image in images.iter_mut() {
        image.active = image.id == id;
    }
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    #[serde(default)]
    pub players: Vec<Combatant>,
    #[serde(default)]
    pub images: Vec<SceneImage>,
    #[serde(default)]
    pub current_player: Option<usize>,
    #[serde(default)]
    pub initiative_visible: bool,
    #[serde(default)]
    pub health_visible: bool,
    #[serde(default)]
    pub enemy_health_visible: bool,
    #[serde(default = "default_true")]
    pub show_initiative_roll: bool,
    #[serde(default)]
    pub auto_hide_inactive: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            images: Vec::new(),
            current_player: None,
            initiative_visible: false,
            health_visible: false,
            enemy_health_visible: false,
            show_initiative_roll: true,
            auto_hide_inactive: false,
        }
    }
}

impl Campaign {
    /// Seed matching `defaultCampaing()` in `Console.svelte`.
    pub fn default_seed() -> Self {
        Self {
            players: vec![
                Combatant::new("Player 1", CombatantKind::Player, 3, DEFAULT_HEALTH),
                Combatant::new("Player 2", CombatantKind::Player, 2, DEFAULT_HEALTH),
                Combatant::new("Player 3", CombatantKind::Player, 1, DEFAULT_HEALTH),
            ],
            images: Vec::new(),
            current_player: None,
            initiative_visible: false,
            health_visible: false,
            enemy_health_visible: false,
            show_initiative_roll: true,
            auto_hide_inactive: false,
        }
    }

    pub fn normalize_dead(&mut self) -> bool {
        self.players.iter_mut().any(|p| p.normalize_dead())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_display_size")]
    pub display_size: f64,
    #[serde(default = "default_campaign_name")]
    pub current_campaign: String,
    #[serde(default = "default_campaigns")]
    pub campaigns: Vec<String>,
    #[serde(default)]
    pub campaign_data: std::collections::BTreeMap<String, Campaign>,
}

fn default_display_size() -> f64 {
    1.0
}

fn default_campaign_name() -> String {
    DEFAULT_CAMPAIGN_NAME.to_string()
}

fn default_campaigns() -> Vec<String> {
    vec![DEFAULT_CAMPAIGN_NAME.to_string()]
}

impl Default for AppState {
    fn default() -> Self {
        let mut campaign_data = std::collections::BTreeMap::new();
        campaign_data.insert(DEFAULT_CAMPAIGN_NAME.to_string(), Campaign::default_seed());
        Self {
            theme: Theme::System,
            display_size: 1.0,
            current_campaign: DEFAULT_CAMPAIGN_NAME.to_string(),
            campaigns: vec![DEFAULT_CAMPAIGN_NAME.to_string()],
            campaign_data,
        }
    }
}

impl AppState {
    /// Ensure campaigns list / current / data are consistent and normalize dead flags.
    pub fn normalize(&mut self) -> bool {
        let mut changed = false;

        if self.campaigns.is_empty() {
            self.campaigns.push(DEFAULT_CAMPAIGN_NAME.to_string());
            changed = true;
        }

        if self.current_campaign.is_empty()
            || !self.campaigns.contains(&self.current_campaign)
        {
            self.current_campaign = self.campaigns[0].clone();
            changed = true;
        }

        for name in self.campaigns.clone() {
            if !self.campaign_data.contains_key(&name) {
                self.campaign_data
                    .insert(name, Campaign::default_seed());
                changed = true;
            }
        }

        for campaign in self.campaign_data.values_mut() {
            if campaign.normalize_dead() {
                changed = true;
            }
        }

        changed
    }

    pub fn current_mut(&mut self) -> &mut Campaign {
        let name = self.current_campaign.clone();
        self.campaign_data
            .entry(name)
            .or_insert_with(Campaign::default_seed)
    }

    pub fn current(&self) -> Option<&Campaign> {
        self.campaign_data.get(&self.current_campaign)
    }

    /// Add a campaign seeded like `defaultCampaing()`. Rejects empty / duplicate names.
    pub fn add_campaign(&mut self, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() || self.campaigns.iter().any(|c| c == name) {
            return false;
        }
        self.campaigns.push(name.to_string());
        self.campaign_data
            .insert(name.to_string(), Campaign::default_seed());
        true
    }

    /// Switch `current_campaign` if the name exists in the list.
    pub fn set_current_campaign(&mut self, name: &str) -> bool {
        if !self.campaigns.iter().any(|c| c == name) {
            return false;
        }
        self.current_campaign = name.to_string();
        true
    }

    /// Rename/rekey the current campaign when the new name is unique.
    /// Parity with `saveSettings` in `Console.svelte`.
    pub fn rename_current_campaign(&mut self, new_name: &str) -> bool {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return false;
        }
        let old = self.current_campaign.clone();
        if new_name == old {
            return true;
        }
        if !self.campaigns.iter().any(|c| c == &old)
            || self.campaigns.iter().any(|c| c == new_name)
        {
            return false;
        }
        if let Some(data) = self.campaign_data.remove(&old) {
            self.campaign_data.insert(new_name.to_string(), data);
        } else {
            self.campaign_data
                .insert(new_name.to_string(), Campaign::default_seed());
        }
        for c in &mut self.campaigns {
            if *c == old {
                *c = new_name.to_string();
            }
        }
        self.current_campaign = new_name.to_string();
        true
    }
}

/// Title-case for campaign labels (parity with Svelte `toTitleCase`).
pub fn to_title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// JSON / form slug for a combatant kind (`player` / `npc` / `monster`).
pub fn kind_slug(kind: CombatantKind) -> &'static str {
    match kind {
        CombatantKind::Player => "player",
        CombatantKind::Npc => "npc",
        CombatantKind::Monster => "monster",
    }
}

/// Console list meta label (`PC` / `NPC` / `Monster`).
pub fn kind_label(kind: CombatantKind) -> &'static str {
    match kind {
        CombatantKind::Player => "PC",
        CombatantKind::Npc => "NPC",
        CombatantKind::Monster => "Monster",
    }
}

/// Default name when the add form leaves name empty: `New {TitleCase(slug)}`.
pub fn default_combatant_name(kind: CombatantKind) -> String {
    format!("New {}", to_title_case(kind_slug(kind)))
}

/// Resolve add-form name (trim; empty → default).
pub fn resolve_combatant_name(name: &str, kind: CombatantKind) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        default_combatant_name(kind)
    } else {
        trimmed.to_string()
    }
}

/// Accept JSON numbers or numeric strings (Tauri / InPlaceEdit sometimes stores strings).
fn deserialize_i32_lenient<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct I32Visitor;

    impl<'de> Visitor<'de> for I32Visitor {
        type Value = i32;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an integer or numeric string")
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<i32, E> {
            i32::try_from(v).map_err(E::custom)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<i32, E> {
            i32::try_from(v).map_err(E::custom)
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<i32, E> {
            Ok(v as i32)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<i32, E> {
            v.trim()
                .parse::<f64>()
                .map(|n| n as i32)
                .map_err(E::custom)
        }
    }

    deserializer.deserialize_any(I32Visitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_derived_from_health() {
        let mut c = Combatant::new("X", CombatantKind::Player, 1, 10);
        assert!(!c.dead);
        c.health = 0;
        assert!(c.normalize_dead());
        assert!(c.dead);
        c.health = 5;
        assert!(c.normalize_dead());
        assert!(!c.dead);
    }

    #[test]
    fn default_state_has_seed_campaign() {
        let state = AppState::default();
        assert_eq!(state.current_campaign, "default");
        let camp = state.current().unwrap();
        assert_eq!(camp.players.len(), 3);
        assert!(camp.images.is_empty());
        assert!(camp.show_initiative_roll);
        assert!(!camp.auto_hide_inactive);
    }

    #[test]
    fn lenient_i32_from_string() {
        let json = r#"{
            "id": "a",
            "name": "P",
            "initiative": "5",
            "health": "0",
            "max_health": "10"
        }"#;
        let c: Combatant = serde_json::from_str(json).unwrap();
        assert_eq!(c.initiative, 5);
        assert_eq!(c.health, 0);
        assert_eq!(c.max_health, 10);
    }

    #[test]
    fn add_campaign_rejects_empty_and_duplicate() {
        let mut state = AppState::default();
        assert!(!state.add_campaign(""));
        assert!(!state.add_campaign("default"));
        assert!(state.add_campaign("Family"));
        assert!(state.campaign_data.contains_key("Family"));
        assert_eq!(state.campaign_data["Family"].players.len(), 3);
    }

    #[test]
    fn rename_rekeys_without_data_loss() {
        let mut state = AppState::default();
        state.current_mut().players[0].name = "Hero".into();
        assert!(state.rename_current_campaign("Adventure"));
        assert_eq!(state.current_campaign, "Adventure");
        assert!(!state.campaign_data.contains_key("default"));
        assert_eq!(state.campaign_data["Adventure"].players[0].name, "Hero");
        assert_eq!(state.campaigns, vec!["Adventure".to_string()]);
        assert!(state.rename_current_campaign("Adventure")); // same name is a no-op success
        state.add_campaign("Other");
        assert!(!state.rename_current_campaign("Other")); // duplicate
    }

    #[test]
    fn set_current_campaign_switches() {
        let mut state = AppState::default();
        state.add_campaign("Family");
        assert!(state.set_current_campaign("Family"));
        assert_eq!(state.current_campaign, "Family");
        assert!(!state.set_current_campaign("missing"));
    }

    #[test]
    fn resolve_combatant_name_defaults() {
        assert_eq!(
            resolve_combatant_name("", CombatantKind::Player),
            "New Player"
        );
        assert_eq!(
            resolve_combatant_name("  ", CombatantKind::Npc),
            "New Npc"
        );
        assert_eq!(
            resolve_combatant_name("Goblin", CombatantKind::Monster),
            "Goblin"
        );
        assert_eq!(kind_label(CombatantKind::Player), "PC");
        assert_eq!(kind_label(CombatantKind::Npc), "NPC");
        assert_eq!(kind_label(CombatantKind::Monster), "Monster");
    }

    #[test]
    fn scene_image_name_from_path_uses_stem() {
        assert_eq!(
            SceneImage::name_from_path(std::path::Path::new("/tmp/Forest.png")),
            "Forest"
        );
        assert_eq!(
            SceneImage::name_from_path(std::path::Path::new("noext")),
            "noext"
        );
    }

    #[test]
    fn activate_scene_image_sets_single_active() {
        let mut images = vec![
            SceneImage::new("a", "/a.png"),
            SceneImage::new("b", "/b.png"),
        ];
        let id = images[1].id.clone();
        assert!(activate_scene_image(&mut images, &id));
        assert!(!images[0].active);
        assert!(images[1].active);
        assert_eq!(active_scene_image(&Campaign {
            images: images.clone(),
            ..Campaign::default()
        }).unwrap().id, id);
        assert!(!activate_scene_image(&mut images, "missing"));
    }
}