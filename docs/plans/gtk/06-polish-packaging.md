---
title: Phase 6 — Polish, packaging & parallel ship
type: port
date: 2026-08-02
phase: 6
status: done
depends_on: [5]
---

# Phase 6 — Polish, packaging & parallel ship

## Goal

Make the GTK app ship-ready on **GNOME 50** Flatpak while **keeping the Tauri frontend** available for the remainder of the transition.

## Scope

**In**

- Accessibility pass (keyboard, screen reader labels, Reduced Motion already wired)
- Useful shortcuts (e.g. Next Turn) via `GtkShortcutController`
- Metainfo screenshots, release notes stub, icon completeness
- Flatpak hardening (portals for file open; minimal filesystem overrides)
- Root README: dual-frontend developer & user docs
- Full manual parity QA checklist
- Optional: CI job building the GTK Flatpak / meson tree without breaking Tauri CI

**Out**

- Removing `src/`, `src-tauri/`, yarn/vite (requires a **separate cutover plan**)
- Flathub submission paperwork (can be prepared here, submitted later)
- Feature work beyond the parity matrix

## Packaging

### Flatpak

- Runtime/SDK: `org.gnome.Platform` // `50`, `org.gnome.Sdk` // `50`
- Finish args: Wayland + fallback X11, dri, etc. as typical for GTK games/tools UIs
- Prefer **document portal** for images over broad `filesystem=home`
- AppStream metainfo validates; aligns with desktop entry `app-id`

### Native

- `meson install` installs binary, icons, desktop, metainfo
- Document distro build deps: `gtk4`, `libadwaita`, Blueprint compiler, Rust

## Polish checklist

- [x] All dialogs Escape-dismissible; focus order sane
- [x] Buttons have tooltips / `accessible-label` where icon-only
- [x] Presenter readable at `display_size` 1.0–5.0
- [x] No panic on empty campaigns / empty players
- [x] Shutdown saves state (parity with Tauri `onCloseRequested` → `saveStore`)
- [x] App switcher icon + title correct for Console vs Presenter windows

## Documentation

Update (additive):

- Root `README.md` — “Frontends” section: Tauri (current) + GTK (GNOME 50 preview/parallel)
- `gtk/README.md` — build, Flatpak, import notes
- `docs/plans/gtk/README.md` — mark phases complete as they land
- Changelog entry for the GTK preview if the project keeps one

## Full parity QA (manual)

Run against both apps where applicable; GTK must pass all:

1. Campaigns: add, switch, rename, persist
2. Settings: theme system/light/dark; display size; show init roll; auto-hide
3. Combatants: add PC/NPC/Monster; edit fields; delete; dead derivation
4. Visibility toggles affect Presenter only as in Tauri
5. Start / Next / Prev / End
6. Long Rest; Clear Monsters
7. Presenter open / fullscreen / close-hide / F11 / Esc
8. Images add / rename / activate / crossfade / reduced motion
9. Restart restores session
10. Best-effort Tauri store import (if fixture available)

## Parallel transition policy

Until cutover plan exists:

- Version numbering: either share `1.x` with a “GTK preview” note, or give GTK builds a clear `gtk` tag in Flatpak branch — decide at release time; document choice here when first tagged.
- Bugs may be fixed in one frontend only if the other is unaffected; **parity regressions** in GTK are Phase 6 blockers.
- Do not delete Tauri CI or scripts.

## Work items

1. Shortcuts + a11y pass.
2. Metainfo/screenshots/icons.
3. Flatpak manifest finalize + sandbox test for images.
4. README dual-frontend docs.
5. Execute full parity QA; file gaps as follow-ups only if non-blocking polish.
6. Optionally add CI workflow `gtk.yml` alongside existing Tauri workflows.

## Verification

- [x] Flatpak finish-args hardened (portal + app-data image copies; no home/Pictures overrides); CI builds Flatpak (`gtk.yml`). Local install needs `flatpak-builder`.
- [x] Parity QA checklist documented above (manual operator checklist; unit coverage for domain/store)
- [x] `yarn tauri build` / `yarn tauri dev` still documented and working
- [x] No removal of Tauri sources

## Exit criteria

GTK app is a viable parallel frontend with full feature parity on GNOME 50. Tauri remains. Cutover/removal is a **new plan**, not part of Phase 6.