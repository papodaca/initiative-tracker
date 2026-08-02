---
title: Phase 4 — Presenter window & live sync
type: port
date: 2026-08-02
phase: 4
status: done
depends_on: [3]
---

# Phase 4 — Presenter window & live sync

## Goal

Add the player-facing Presenter window with live state sync from the Console, matching Tauri’s dual-window behavior (open / hide-close / fullscreen / F11 / Esc).

## Scope

**In**

- `PresenterWindow` as a second `GtkWindow` owned by the application
- Console controls: Open Presenter, Fullscreen / Exit Fullscreen, Close (hide)
- Live updates from `StateStore` (replacement for `state-change` events)
- Read-only initiative list with Presenter filters and HP rules
- Display size scaling
- Keyboard: F11 toggles fullscreen; Esc exits fullscreen when fullscreen

**Out**

- Scene background images / crossfade (Phase 5 — Presenter can show a blank/default background until then)
- Changing Tauri presenter IPC

## Window semantics (parity)

From `tauri.conf.json` + `Console.svelte` / `Presenter.svelte`:

| Behavior | Parity rule |
|---|---|
| Created at startup | May create lazily on first Open, or at app start hidden — either OK if Open shows it |
| Close button | Hide, do not destroy; Console “Close” hides |
| Closable | Prefer hide-on-close (`set_hide_on_close(true)` or connect `close-request`) |
| Fullscreen | Console toggles; Presenter can toggle via F11; notify Console button state |
| Parent | Tauri sets `parent: main`; GTK may set transient parent optionally — do not force same-monitor |

## Presenter content rules

When `initiative_visible` is false → show no list (background only later).

When true:

1. Players source = campaign players
2. If `auto_hide_inactive` → filter out `dead`
3. If `show_initiative_roll` is false → hide initiative badge/number
4. Apply HP visibility rules from Phase 3 (PC/NPC full HP vs monster damage-taken)
5. Scale typography/spacing by `display_size` (Tauri uses `font-size: {dislaySize}em` wrapper)

Active combatant visually emphasized; dead (if shown) struck through / dimmed like Console presenter mode (`initiative-list` opacity).

## Sync model

Replace Tauri events:

| Tauri | GTK |
|---|---|
| `emit('state-change', state)` | `StateStore` signal / callback → Presenter rebinds |
| `presenter.emit('set-fullscreen', …)` | Console calls `presenter.set_fullscreen(bool)` |
| `emit('fullscreen', { fullscreen })` | Presenter notifies Console (signal) to sync button label |

No JSON IPC bus required inside one process.

## UI mapping

| Piece | Suggestion |
|---|---|
| Presenter chrome | Minimal `GtkWindow`; optional undecorated when fullscreen only |
| List | Reuse row widget in read-only mode, or dedicated Presenter list |
| Console Presenter section | Enable/disable Open/Fullscreen/Close from `is_visible` / fullscreen flags |

## Work items

1. Add `PresenterWindow` type; register with `Application`.
2. Wire Console Presenter & Media actions (images still stub).
3. Subscribe Presenter to store updates; initial hydrate on show.
4. Implement fullscreen + key controllers.
5. Implement read-only list + filters + display_size CSS/provider or Pango scale.
6. Keep Console button disabled states in sync (`Open` disabled while visible, etc.).

## Wayland note

Do not promise “open fullscreen on monitor 2.” Document: drag Presenter to the player display, then Fullscreen — same as typical Tauri usage.

## Verification

- [x] Open Presenter from Console; hide via Close; Open again restores
- [x] Next Turn on Console updates Presenter without reopen
- [x] Visibility toggles and settings (show roll, auto-hide) reflect immediately
- [x] F11 / Esc / Console fullscreen controls agree on state
- [x] Display size changes update Presenter scale
- [x] Closing Console quits app cleanly (Presenter destroyed with app); no orphan windows
- [x] Tauri dual-window flow still works in parallel

## Exit criteria

Dual-window combat display parity achieved (minus scene backgrounds).

## Implementation notes (2026-08-02)

- `gtk/src/presenter_window.rs` — hide-on-close `AdwApplicationWindow`, `StateStore` subscribe, read-only list with `visible_combatants` / `presenter_hp_display`, F11/Esc, display-size CSS provider.
- Console Presenter & Media: Open / Fullscreen / Close with button sensitivity synced from Presenter show/hide + fullscreen notify.
- Scene backgrounds remain Phase 5; Presenter shows list-only until then.
- Wayland: drag Presenter to player display, then Fullscreen (documented in stub label).