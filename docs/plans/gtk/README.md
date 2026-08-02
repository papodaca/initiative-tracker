---
title: GTK4 / libadwaita Port — Overview
type: port
date: 2026-08-02
status: planned
target: GNOME 50
---

# GTK4 / libadwaita Port — Overview

## Goal

Port the Initiative Tracker **frontend** to GTK4 + libadwaita, targeting **latest GNOME 50**, with **full feature parity** to the existing Tauri/Svelte UI.

**Both frontends run in parallel during the transition.** The Tauri/Svelte app remains the production path until the GTK app reaches parity and is intentionally cut over. This plan set does **not** remove Tauri.

## Authority

| Source | Authoritative for |
|---|---|
| Existing Tauri/Svelte app (`src/`, `src-tauri/`) | Behavior, persistence schema semantics, dual-window UX |
| This plan set (`docs/plans/gtk/`) | GTK architecture, phase order, GNOME 50 targets |
| GNOME 50 / libadwaita HIG | Widget choice, theming, packaging conventions |

Where appearance conflicts with Adwaita norms, **Adwaita wins**. Where behavior conflicts, **Tauri feature parity wins**.

## Non-goals (transition)

- Removing the Tauri/Svelte stack (deferred until an explicit cutover plan).
- Pixel-matching the web CSS; adopt Adwaita patterns instead.
- Implementing Console drag-reorder (present in `PlayerList` but unused — `sortable={false}`).
- Short Rest (no UI today; only Long Rest).

## Current app snapshot

- **Stack:** Tauri 2 + Svelte 5 + Tailwind v4 + FontAwesome
- **Windows:** Console (`console.html`) + Presenter (`presenter.html`, label `presenter`)
- **IPC:** `state-change`, `set-fullscreen`, `fullscreen`
- **Persistence:** `@tauri-apps/plugin-store` → `.settings.dat`
- **Rust backend:** thin shell (`dialog` + `store` plugins only) — almost all logic is in Svelte

## Target stack (GNOME 50)

| Piece | Target |
|---|---|
| Platform | GNOME 50 (released; e.g. gnome-shell 50.x) |
| GTK | 4.22+ via `gtk4` crate **0.11.x**, feature `gnome_50` |
| libadwaita | 1.9.x via `libadwaita` crate **0.9.x**, feature `v1_9` (1.10 still alpha — avoid for ship) |
| UI | Blueprint (`.blp`) + gtk4-rs |
| Build | Meson + Cargo |
| Package | Flatpak first (`org.gnome.Platform` / SDK **50**), native meson path secondary |

## Parallel-frontend layout

```
initiative-tracker/
├── src/                    # Tauri/Svelte frontend (unchanged by default)
├── src-tauri/              # Tauri shell (unchanged by default)
├── gtk/                    # NEW: GTK4/libadwaita frontend + shared domain
│   ├── Cargo.toml
│   ├── meson.build
│   ├── src/                # Rust app
│   ├── data/               # desktop, metainfo, icons, gschema
│   └── flatpak/            # Flatpak manifest
└── docs/plans/gtk/         # This plan set
```

Shared domain logic lives under `gtk/` (or a small `crates/core` if extraction pays off). The Svelte app does **not** need to call into that crate during the transition; parity is behavioral, not binary-shared, unless a later phase extracts a common store format intentionally.

**Recommended shared contract during transition:** the JSON state schema (Phase 1), so GTK can import Tauri saves and both UIs can be validated against the same fixtures.

## Feature parity matrix

| Feature | Console | Presenter |
|---|---|---|
| Multi-campaign select / add / rename | ✓ | — |
| Settings: theme, display size, show init roll, auto-hide dead | ✓ | consumes |
| Presenter open / fullscreen / close (hide) | ✓ | ✓ (F11 / Esc) |
| Scene images: add, rename, activate, crossfade background | ✓ | ✓ |
| Add PC / NPC / Monster (name, init, max HP) | ✓ | — |
| Visibility: initiative, enemy HP, player HP | ✓ | ✓ |
| Combatant list: edit name/init/HP, delete, dead, active turn | ✓ | read-only + filters |
| Combat loop: prev / next / start / end | ✓ | shows active |
| Long rest (PC/NPC → max HP) | ✓ | — |
| Clear monsters | ✓ | — |
| Persist state across restarts | ✓ | ✓ |
| Light / dark / system theme | ✓ | ✓ |

## Phases

| Phase | Doc | Focus | Exit criteria |
|---|---|---|---|
| 0 | [00-scaffold.md](./00-scaffold.md) | Meson/Cargo/Flatpak shell on GNOME 50 | Empty Adwaita app runs beside Tauri |
| 1 | [01-domain-persistence.md](./01-domain-persistence.md) | Domain model, JSON store, Tauri import | State round-trips; tests pass ✅ |
| 2 | [02-console-shell.md](./02-console-shell.md) | Console chrome, campaigns, settings | Navigation/settings parity ✅ |
| 3 | [03-combat-loop.md](./03-combat-loop.md) | Combatants + combat actions | Full combat without Presenter ✅ |
| 4 | [04-presenter.md](./04-presenter.md) | Presenter window + live sync | Dual-window parity ✅ |
| 5 | [05-scene-images.md](./05-scene-images.md) | Images + crossfade | Media parity |
| 6 | [06-polish-packaging.md](./06-polish-packaging.md) | A11y, Flatpak, docs; **still keep Tauri** | Ship-ready GTK app in parallel |

Cutover / Tauri removal is **out of this plan set** and requires a separate plan after Phase 6.

## Decisions (locked)

1. **Parallel frontends** for the transition — do not delete Tauri/Svelte.
2. **Full feature parity** with the current Console + Presenter behavior.
3. **Target GNOME 50** with `gtk4` `gnome_50` + `libadwaita` `v1_9`.
4. **Blueprint + gtk4-rs** for UI.
5. **Flatpak-first** packaging.
6. **Best-effort import** of existing `.settings.dat` / mapped JSON when present.

## Risks

- **Wayland fullscreen / monitor selection:** Presenter is user-dragged to the TV then fullscreened — same practical UX as today.
- **Asset URLs:** Tauri `convertFileSrc` / `asset://` must become filesystem paths + `GtkPicture` / pixbuf.
- **Store format drift:** keep a documented schema and importers so Tauri and GTK do not diverge silently.
- **Reduced Motion (GNOME 50):** crossfade must respect the new accessibility setting.

## Verification philosophy

No test framework in the Tauri app today. GTK phases add **Rust unit tests** for domain logic (Phase 1+) and **manual smoke checklists** per phase against the parity matrix. Both `yarn tauri dev` and `meson compile` / Flatpak builds must remain viable throughout.