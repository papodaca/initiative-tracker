---
title: Phase 2 — Console shell (campaigns & settings)
type: port
date: 2026-08-02
phase: 2
status: planned
depends_on: [0, 1]
---

# Phase 2 — Console shell (campaigns & settings)

## Goal

Build the Console window chrome with Adwaita patterns: campaign switching, add-campaign, and settings — matching Tauri Console information architecture, not its CSS.

## Scope

**In**

- `AdwApplicationWindow` Console layout: header, scrollable body stub, footer stub
- Campaign dropdown + add-campaign dialog
- Settings dialog (theme, display size, campaign rename, show initiative roll, auto-hide inactive)
- Wire to `StateStore` from Phase 1
- Theme via `AdwStyleManager` (system / light / dark)

**Out**

- Combatant rows and combat actions (Phase 3)
- Presenter window (Phase 4)
- Image gallery (Phase 5)

## UI mapping

| Tauri / Svelte | GTK / libadwaita |
|---|---|
| Header bar | `AdwToolbarView` + `AdwHeaderBar` |
| Campaign `<select>` | `GtkDropDown` or `AdwComboRow` in header |
| Add campaign button | Header button → `AdwAlertDialog` / `AdwDialog` + entry |
| Settings button | Header button → `AdwPreferencesDialog` or `AdwDialog` |
| Theme select | `AdwComboRow` → `AdwStyleManager::set_color_scheme` |
| Display size range | `GtkScale` (1.0–5.0, step 0.1) — stored for Presenter |
| Show Initiative Roll | `AdwSwitchRow` |
| Auto-Hide Inactive Turns | `AdwSwitchRow` |
| Campaign name field | `AdwEntryRow`; rename rekeys campaign like `saveSettings` in Console |
| Collapsible drawers (shell only) | `AdwPreferencesGroup` sections or `GtkExpander` placeholders labeled for later phases |

## Behavior parity

From `Console.svelte` / `SettingsOverlay.svelte` / `AddCampaignDialog.svelte`:

1. Switching `current_campaign` persists immediately (Tauri `broadcastState` on change).
2. Add campaign: reject empty / duplicate names; seed with default campaign contents; close dialog.
3. Settings save:
   - Rename current campaign if new name unique and old name exists in list (rekey map + campaigns vec).
   - Persist theme + display size + campaign flags.
   - Apply color scheme; when `system`, follow OS (Adw handles this).
4. Settings / add-campaign dismiss on cancel; Escape closes dialogs.

## Blueprint suggestion

- `window.blp` — Console shell
- `dialogs/settings.blp`
- `dialogs/add_campaign.blp`

Keep logic in Rust controllers; Blueprint for structure.

## Work items

1. Lay out Console shell with placeholder groups: “Presenter & Media”, “Add Combatant”, visibility row stub, list stub, footer stub.
2. Implement campaign dropdown bound to `campaigns` + `current_campaign`.
3. Implement add-campaign dialog.
4. Implement settings dialog + rename rekey.
5. Connect `AdwStyleManager` to `theme` field on load and on save.
6. Persist all mutations through Phase 1 store.

## Parallel-frontend constraints

- Do not change Svelte settings UX.
- Visual language is Adwaita; labels/copy should stay recognizable (“Show Initiative Roll”, “Auto-Hide Inactive Turns”, etc.).

## Verification

- [ ] Create two campaigns, switch, restart GTK app — selection and data survive
- [ ] Rename campaign rekeys without data loss
- [ ] Theme system/light/dark matches desktop expectation
- [ ] Display size persists (even if Presenter not yet reading it)
- [ ] Tauri Console still behaves as before

## Exit criteria

Campaign + settings flows reach feature parity; Console shell ready for combat list integration.
