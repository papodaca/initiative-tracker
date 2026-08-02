use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;
use serde::Deserialize;
use serde_json::Value;

use crate::domain::{
    AppState, Campaign, Combatant, CombatantKind, SceneImage, Theme, DEFAULT_CAMPAIGN_NAME,
};

#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub source: Option<PathBuf>,
    pub skipped_images: Vec<String>,
    pub notes: Vec<String>,
}

/// Common locations for the Tauri `@tauri-apps/plugin-store` file.
pub fn tauri_candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut push_under = |root: PathBuf| {
        paths.push(root.join("im.apodaca.initiative-tracker/.settings.dat"));
        paths.push(root.join("im.apodaca.InitiativeTracker/.settings.dat"));
        paths.push(root.join("com.tauri.dev/.settings.dat"));
    };

    if let Some(base) = directories::BaseDirs::new() {
        push_under(base.data_dir().to_path_buf());
    }
    if let Ok(home) = std::env::var("HOME") {
        push_under(PathBuf::from(home).join(".local/share"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        push_under(PathBuf::from(xdg));
    }

    paths.sort();
    paths.dedup();
    paths
}

/// Best-effort import from a Tauri store file or a bare exported state JSON.
pub fn import_tauri_state(path: &Path) -> Result<(AppState, ImportReport), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut report = ImportReport {
        source: Some(path.to_path_buf()),
        ..Default::default()
    };

    let state_value = if let Some(s) = value.get("state") {
        s.clone()
    } else if value.get("campaigns").is_some() || value.get("currentCampaign").is_some() {
        report
            .notes
            .push("Treated file as bare exported state JSON (no top-level \"state\" key).".into());
        value
    } else {
        return Err("Unrecognized Tauri store shape (missing \"state\")".into());
    };

    let legacy: LegacyState =
        serde_json::from_value(state_value).map_err(|e| format!("legacy parse: {e}"))?;
    let state = map_legacy(legacy, &mut report);
    Ok((state, report))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyState {
    #[serde(default)]
    theme: Option<String>,
    /// Typo preserved in Tauri saves.
    #[serde(default, alias = "dislaySize", alias = "displaySize")]
    dislay_size: Option<f64>,
    #[serde(default)]
    current_campaign: Option<String>,
    #[serde(default)]
    campaigns: Option<Vec<String>>,
    /// Remaining keys are campaign objects keyed by name.
    #[serde(flatten)]
    rest: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyCampaign {
    #[serde(default)]
    players: Vec<LegacyCombatant>,
    #[serde(default)]
    images: Vec<LegacyImage>,
    #[serde(default)]
    current_player: Option<usize>,
    #[serde(default)]
    initiative_visible: Option<bool>,
    #[serde(default)]
    health_visible: Option<bool>,
    #[serde(default)]
    enemy_health_visible: Option<bool>,
    #[serde(default)]
    show_initiative_roll: Option<bool>,
    #[serde(default)]
    auto_hide_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyCombatant {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default, deserialize_with = "de_i32_opt")]
    initiative: Option<i32>,
    #[serde(default, deserialize_with = "de_i32_opt")]
    health: Option<i32>,
    #[serde(default, deserialize_with = "de_i32_opt")]
    max_health: Option<i32>,
    #[serde(default)]
    active: Option<bool>,
    #[serde(default)]
    dead: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyImage {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    file_url: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    active: Option<bool>,
}

fn de_i32_opt<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<Value>::deserialize(deserializer)?;
    Ok(match v {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_i64().map(|i| i as i32).or_else(|| {
            n.as_f64().map(|f| f as i32)
        }),
        Some(Value::String(s)) => s.trim().parse::<f64>().ok().map(|f| f as i32),
        Some(_) => None,
    })
}

fn map_legacy(legacy: LegacyState, report: &mut ImportReport) -> AppState {
    let mut state = AppState::default();

    state.theme = match legacy.theme.as_deref() {
        Some("light") => Theme::Light,
        Some("dark") => Theme::Dark,
        _ => Theme::System,
    };
    if let Some(size) = legacy.dislay_size {
        state.display_size = size;
    }

    let campaigns = legacy
        .campaigns
        .unwrap_or_else(|| vec![DEFAULT_CAMPAIGN_NAME.to_string()]);
    state.campaigns = campaigns.clone();
    state.current_campaign = legacy
        .current_campaign
        .unwrap_or_else(|| campaigns.first().cloned().unwrap_or_else(|| DEFAULT_CAMPAIGN_NAME.to_string()));

    let reserved = ["theme", "dislaySize", "displaySize", "currentCampaign", "campaigns"];
    state.campaign_data.clear();

    for name in &campaigns {
        if let Some(val) = legacy.rest.get(name) {
            match serde_json::from_value::<LegacyCampaign>(val.clone()) {
                Ok(lc) => {
                    state
                        .campaign_data
                        .insert(name.clone(), map_campaign(lc, report));
                }
                Err(e) => {
                    report
                        .notes
                        .push(format!("Failed to parse campaign \"{name}\": {e}; seeding default"));
                    state
                        .campaign_data
                        .insert(name.clone(), Campaign::default_seed());
                }
            }
        } else {
            report
                .notes
                .push(format!("Campaign \"{name}\" missing from store; seeding default"));
            state
                .campaign_data
                .insert(name.clone(), Campaign::default_seed());
        }
    }

    // Also pick up campaign objects present under rest but not listed (defensive).
    for (key, val) in &legacy.rest {
        if reserved.contains(&key.as_str()) || state.campaign_data.contains_key(key) {
            continue;
        }
        if val.is_object() {
            if let Ok(lc) = serde_json::from_value::<LegacyCampaign>(val.clone()) {
                if !state.campaigns.contains(key) {
                    state.campaigns.push(key.clone());
                }
                state
                    .campaign_data
                    .insert(key.clone(), map_campaign(lc, report));
            }
        }
    }

    state.normalize();
    let _ = reserved; // silence if unused in some builds
    state
}

fn map_campaign(lc: LegacyCampaign, report: &mut ImportReport) -> Campaign {
    let mut players: Vec<Combatant> = lc
        .players
        .into_iter()
        .map(|p| {
            let max_health = p.max_health.unwrap_or(crate::domain::DEFAULT_HEALTH);
            let health = p.health.unwrap_or(max_health);
            let mut c = Combatant {
                id: p
                    .id
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                name: p.name.unwrap_or_else(|| "Unnamed".into()),
                kind: parse_kind(p.kind.as_deref()),
                initiative: p.initiative.unwrap_or(0),
                health,
                max_health,
                active: p.active.unwrap_or(false),
                dead: p.dead.unwrap_or(false),
            };
            c.normalize_dead();
            c
        })
        .collect();

    // Keep imported order; GTK sort is explicit when editing.
    let _ = &mut players;

    let images = lc
        .images
        .into_iter()
        .filter_map(|img| map_image(img, report))
        .collect();

    Campaign {
        players,
        images,
        current_player: lc.current_player,
        initiative_visible: lc.initiative_visible.unwrap_or(false),
        health_visible: lc.health_visible.unwrap_or(false),
        enemy_health_visible: lc.enemy_health_visible.unwrap_or(false),
        show_initiative_roll: lc.show_initiative_roll.unwrap_or(true),
        auto_hide_inactive: lc.auto_hide_inactive.unwrap_or(false),
    }
}

fn parse_kind(kind: Option<&str>) -> CombatantKind {
    match kind.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("npc") => CombatantKind::Npc,
        Some("monster") => CombatantKind::Monster,
        _ => CombatantKind::Player,
    }
}

fn map_image(img: LegacyImage, report: &mut ImportReport) -> Option<SceneImage> {
    let path = if let Some(p) = img.path.filter(|p| !p.is_empty()) {
        p
    } else if let Some(url) = img.file_url.as_deref() {
        match recover_path_from_file_url(url) {
            Some(p) => p,
            None => {
                report.skipped_images.push(format!(
                    "{} ({})",
                    img.name.as_deref().unwrap_or("?"),
                    url
                ));
                return None;
            }
        }
    } else {
        report.skipped_images.push(
            img.name
                .clone()
                .unwrap_or_else(|| "<unnamed image>".into()),
        );
        return None;
    };

    Some(SceneImage {
        id: img
            .id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        name: img.name.unwrap_or_else(|| {
            Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image")
                .to_string()
        }),
        path,
        active: img.active.unwrap_or(false),
    })
}

/// Recover a filesystem path from Tauri `convertFileSrc` URLs or plain paths.
///
/// Recoverable:
/// - `asset://localhost/%2Fhome%2F…` → `/home/…`
/// - `file:///home/…`
/// - absolute paths starting with `/`
///
/// Not recoverable (skipped): other schemes, relative opaque URLs.
pub fn recover_path_from_file_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    if url.starts_with('/') {
        return Some(url.to_string());
    }
    if let Some(rest) = url.strip_prefix("file://") {
        let decoded = percent_decode_str(rest).decode_utf8().ok()?.into_owned();
        return Some(decoded);
    }
    // asset://localhost/<percent-encoded-path>
    if let Some(rest) = url
        .strip_prefix("asset://localhost/")
        .or_else(|| url.strip_prefix("https://asset.localhost/"))
    {
        let decoded = percent_decode_str(rest).decode_utf8().ok()?.into_owned();
        if decoded.starts_with('/') {
            return Some(decoded);
        }
        // Sometimes the encoded form already includes leading %2F only.
        if !decoded.is_empty() {
            return Some(if decoded.starts_with('/') {
                decoded
            } else {
                format!("/{decoded}")
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn recover_asset_localhost_url() {
        let url = "asset://localhost/%2Fhome%2Fusr1%2FPictures%2Ffoo.jpg";
        assert_eq!(
            recover_path_from_file_url(url).as_deref(),
            Some("/home/usr1/Pictures/foo.jpg")
        );
    }

    #[test]
    fn skip_opaque_url() {
        assert!(recover_path_from_file_url("blob:xyz").is_none());
    }

    #[test]
    fn import_real_tauri_store_if_present() {
        let path = directories::BaseDirs::new()
            .map(|b| {
                b.data_dir()
                    .join("im.apodaca.initiative-tracker/.settings.dat")
            })
            .expect("xdg");
        if !path.is_file() {
            eprintln!("skip: no Tauri store at {path:?}");
            return;
        }
        let (state, report) = import_tauri_state(&path).expect("import real store");
        assert!(!state.campaigns.is_empty());
        assert!(state.campaign_data.contains_key(&state.current_campaign));
        eprintln!(
            "imported {:?} campaigns={}, skipped_images={}",
            report.source,
            state.campaigns.len(),
            report.skipped_images.len()
        );
    }

    #[test]
    fn import_tauri_shaped_json() {
        let json = r#"{
          "state": {
            "theme": "dark",
            "dislaySize": 1.7,
            "currentCampaign": "Family",
            "campaigns": ["default", "Family"],
            "default": {
              "players": [
                {"id": "1", "name": "P1", "initiative": "5", "health": "0", "maxHealth": 10, "kind": "player"}
              ],
              "images": [
                {"id": "i1", "name": "Art", "fileUrl": "asset://localhost/%2Ftmp%2Fa.png", "active": true},
                {"id": "i2", "name": "Bad", "fileUrl": "blob:nope", "active": false}
              ],
              "showInitiativeRoll": false,
              "autoHideInactive": true,
              "initiativeVisible": true
            },
            "Family": {
              "players": [],
              "images": [],
              "enemyHealthVisible": true
            }
          }
        }"#;
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "it-import-test-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }
        let (state, report) = import_tauri_state(&path).unwrap();
        assert_eq!(state.theme, Theme::Dark);
        assert!((state.display_size - 1.7).abs() < f64::EPSILON);
        assert_eq!(state.current_campaign, "Family");
        let default = state.campaign_data.get("default").unwrap();
        assert_eq!(default.players.len(), 1);
        assert!(default.players[0].dead);
        assert_eq!(default.players[0].initiative, 5);
        assert_eq!(default.images.len(), 1);
        assert_eq!(default.images[0].path, "/tmp/a.png");
        assert!(!default.show_initiative_roll);
        assert!(default.auto_hide_inactive);
        assert_eq!(report.skipped_images.len(), 1);
        let _ = std::fs::remove_file(&path);
    }
}