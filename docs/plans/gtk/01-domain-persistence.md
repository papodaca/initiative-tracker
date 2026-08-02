---
title: Phase 1 — Domain model & persistence
type: port
date: 2026-08-02
phase: 1
status: planned
depends_on: [0]
---

# Phase 1 — Domain model & persistence

## Goal

Extract the campaign/combat domain (today implicit in Svelte) into a Rust module with JSON persistence and a best-effort importer for the Tauri store, so both frontends can be validated against the same behavioral contract.

## Scope

**In**

- Rust domain types matching current runtime state
- Load/save JSON under XDG data dir for the GTK app
- Dead derivation (`health <= 0`), initiative sort, turn wrap, long rest, clear monsters as pure functions
- Best-effort import from Tauri `.settings.dat` / exported JSON
- Unit tests for domain operations

**Out**

- GTK widgets beyond maybe a debug “state loaded” label
- Changing the Tauri store format in the Svelte app (read-compatible only)
- Presenter/console UI

## Canonical schema (GTK JSON)

Mirror existing semantics; fix naming going forward while mapping legacy keys on import:

```json
{
  "theme": "system",
  "display_size": 1.0,
  "current_campaign": "default",
  "campaigns": ["default"],
  "campaign_data": {
    "default": {
      "players": [
        {
          "id": "uuid",
          "name": "Player 1",
          "kind": "player",
          "initiative": 3,
          "health": 10,
          "max_health": 10,
          "active": false,
          "dead": false
        }
      ],
      "images": [
        {
          "id": "uuid",
          "name": "Tavern",
          "path": "/absolute/or/portable/path.png",
          "active": false
        }
      ],
      "current_player": null,
      "initiative_visible": false,
      "health_visible": false,
      "enemy_health_visible": false,
      "show_initiative_roll": true,
      "auto_hide_inactive": false
    }
  }
}
```

### Legacy Tauri mapping

| Tauri / Svelte | GTK JSON |
|---|---|
| `state.theme` | `theme` |
| `state.dislaySize` (typo preserved in Tauri) | `display_size` |
| `state.currentCampaign` | `current_campaign` |
| `state.campaigns` | `campaigns` |
| `state[campaignName]` object | `campaign_data[campaignName]` |
| `player.maxHealth` | `max_health` |
| `player.kind` | `kind` (`player` / `npc` / `monster`) |
| `image.fileUrl` (`asset://…`) | `path` when recoverable; else drop/skip with log |
| `showInitiativeRoll` | `show_initiative_roll` |
| `autoHideInactive` | `auto_hide_inactive` |
| `initiativeVisible` / `healthVisible` / `enemyHealthVisible` | snake_case equivalents |

Default campaign seed matches `defaultCampaing()` in `Console.svelte` (three sample players, empty images, flags as today).

## Module layout (suggested)

```
gtk/src/
├── domain/
│   ├── mod.rs
│   ├── state.rs          # AppState, Campaign, Combatant, SceneImage
│   ├── combat.rs         # next/prev/start/end, long_rest, clear_monsters, sort
│   └── visibility.rs     # presenter filter helpers
├── persistence/
│   ├── mod.rs
│   ├── json_store.rs     # XDG path, load/save atomic
│   └── tauri_import.rs   # .settings.dat / legacy JSON best-effort
└── ...
```

## Persistence rules

- Path: `$XDG_DATA_HOME/im.apodaca.InitiativeTracker/state.json` (or chosen app-id)
- Save: on every mutating store API call (debounce optional) + on shutdown
- Atomic write: temp file + rename
- On first run: try import from common Tauri store locations; else seed defaults
- Never write back into Tauri’s `.settings.dat` during the parallel transition (one-way import)

## StateStore API (UI-facing)

UI phases consume a small façade:

- `load() -> AppState`
- `save()`
- `with_mut(|state| …)` or command methods: `add_combatant`, `next_turn`, …
- Change notification: `glib::subclass` object with signals, or `Rc<RefCell<AppState>>` + callback list — pick one in implementation; prefer a `gio::ListModel`-friendly design where lists bind later

## Work items

1. Define serde types + defaults + dead normalization on load.
2. Implement combat helpers parity-tested against Svelte behavior.
3. Implement JSON store + XDG paths.
4. Implement Tauri importer (document what cannot be recovered, e.g. opaque asset URLs).
5. Add `#[cfg(test)]` coverage for sort, turn wrap, long rest, clear monsters, auto-hide filter, HP visibility display rules.

## Parallel-frontend constraints

- Tauri continues to own `.settings.dat`.
- GTK owns `state.json`.
- Optional later: a shared fixture file under `docs/plans/gtk/fixtures/` for manual comparison — not required to wire into Svelte.

## Verification

- [ ] Unit tests pass (`cargo test` in `gtk/`)
- [ ] Fresh install creates default campaign
- [ ] Import path documented; smoke with a copied Tauri store if available
- [ ] Tauri app persistence unchanged

## Exit criteria

Domain + persistence are usable by UI phases; behavioral helpers are covered by tests; Tauri remains authoritative for its own saves.
