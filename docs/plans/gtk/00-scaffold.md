---
title: Phase 0 — Scaffold & GNOME 50 baseline
type: port
date: 2026-08-02
phase: 0
status: done
depends_on: []
---

# Phase 0 — Scaffold & GNOME 50 baseline

## Goal

Stand up a parallel GTK4/libadwaita application that builds and runs on **GNOME 50**, without touching the Tauri/Svelte frontend.

## Scope

**In**

- New `gtk/` tree beside existing `src/` and `src-tauri/`
- Meson + Cargo project
- `AdwApplication` + empty `AdwApplicationWindow`
- Desktop entry, metainfo stub, app icon reuse/adapt from `src-tauri/icons`
- Flatpak manifest targeting GNOME **50** runtime/SDK
- Dev docs snippet: how to build GTK vs how to run Tauri

**Out**

- Any combat/presenter UI
- Changes to `package.json`, Svelte sources, or Tauri config
- Removing or renaming the Tauri app

## Target versions

| Crate / component | Version / feature |
|---|---|
| `gtk4` | 0.11.x, feature `gnome_50` (GTK 4.22, GIO 2.88) |
| `libadwaita` | 0.9.x, feature `v1_9` |
| Flatpak runtime | `org.gnome.Platform` // `50` (and matching Sdk) |
| Rust edition | 2021 (align with existing crate) or 2024 if toolchain allows |

Pin exact patch versions at scaffold time from crates.io; prefer stable libadwaita **1.9**, not 1.10 alpha.

## App identity

- Keep reverse-DNS family aligned with Tauri: `im.apodaca.initiative-tracker`
- GTK binary / Flatpak id recommendation: `im.apodaca.InitiativeTracker` (GNOME convention) **or** reuse `im.apodaca.initiative-tracker` for continuity — pick one at scaffold and use it consistently in `.desktop`, metainfo, `application_id`, and Flatpak `app-id`.
- Window title: `Initiative Tracker: Console` for the main window (Presenter added in Phase 4).

## Deliverables

```
gtk/
├── Cargo.toml
├── meson.build
├── meson.options          # optional
├── src/
│   ├── main.rs
│   ├── application.rs     # AdwApplication
│   └── window.rs          # AdwApplicationWindow shell
├── data/
│   ├── im.apodaca.InitiativeTracker.desktop.in
│   ├── im.apodaca.InitiativeTracker.metainfo.xml.in
│   ├── icons/...
│   └── resources.gresource.xml   # if using GResource / Blueprint
├── flatpak/
│   └── im.apodaca.InitiativeTracker.json
└── README.md              # GTK-specific build instructions
```

Blueprint toolchain wired in Meson even if Phase 0 UI is code-only or a single empty `.blp`.

## Work items

1. Create `gtk/` Cargo package `initiative-tracker-gtk` (name TBD; must not collide with Tauri package `initiative-tracker` in confusing ways — use explicit package name).
2. Add Meson project that invokes Cargo (pattern used by many GNOME Rust apps) or `cargo` with `meson` installing data files.
3. Implement minimal `Application::new` with correct `application_id` and `AdwApplicationWindow`.
4. Install `.desktop` + metainfo; verify they validate (`desktop-file-validate`, `appstreamcli validate` when available).
5. Add Flatpak manifest; build against GNOME 50.
6. Document in root `README.md` (additive section) or `gtk/README.md`:  
   - Tauri: `yarn tauri dev`  
   - GTK: `meson setup` / `flatpak-builder` commands  
   Emphasize **both are supported**.

## Parallel-frontend constraints

- Do not modify Tauri entrypoints.
- Do not claim the same D-Bus/app id in a way that breaks Flatpak vs Tauri side-by-side on one machine if both use identical ids — if conflict appears, give GTK a distinct id (`…Gtk` suffix) **only if required**, and document it. Prefer one canonical id for the GTK app as the future primary.

## Verification

- [x] `meson compile` (or cargo) produces a binary that opens an Adwaita window
- [ ] Flatpak build succeeds on GNOME 50 SDK — blocked locally: `flatpak-builder` not installed (needs package install); manifest is present at `gtk/flatpak/im.apodaca.InitiativeTracker.json`
- [x] `yarn tauri dev` still works unchanged (no edits under `src/`, `src-tauri/`, or `package.json`)
- [x] No deletions under `src/` or `src-tauri/`

## Exit criteria

Empty Adwaita app runs via native build and Flatpak; Tauri app untouched and still runnable.