---
title: Phase 5 — Scene images & presenter crossfade
type: port
date: 2026-08-02
phase: 5
status: planned
depends_on: [4]
---

# Phase 5 — Scene images & presenter crossfade

## Goal

Reach media feature parity: add/rename/activate scene images in the Console, and show the active image as a crossfading full-window background on the Presenter.

## Scope

**In**

- Image list in Console “Presenter & Media” section
- Multi-file open dialog with image filters
- Rename + set active (single active image)
- Persist filesystem paths in `SceneImage.path`
- Presenter dual-layer background with opacity crossfade
- Respect GNOME 50 **Reduced Motion** (skip or shorten fade)

**Out**

- Cloud/remote images
- Writing into Tauri `asset://` URLs
- Video / animated GIF special handling beyond what GDK loads by default

## Behavior parity

From `ImageList.svelte` + `Presenter.svelte`:

1. **Add Images** opens a file dialog, multiple selection.
2. Extensions (Tauri filter): avif, ico, jfif, svg, png, jpeg, jpg, webp, bmp, gif — support as many as GTK/GDK loaders allow; document any gaps (e.g. jfif alias).
3. Default name = file stem; `id` = UUID; `active = false` on add.
4. Clicking thumbnail sets that image active and clears others’ `active`.
5. Inline rename persists.
6. Presenter: find `images.iter().find(|i| i.active)`; preload then crossfade between two layers (~0.5s ease-in-out in web CSS).
7. Clearing active / no images → no background.

## Path storage (parallel transition)

- Store absolute paths initially (simplest).
- Prefer opening via `GtkFileDialog` so Flatpak document portal can grant access; persist portal-friendly paths / retain necessary permissions as required by sandbox (document in Phase 6 if portal bookmarks needed).
- Import from Tauri: map `fileUrl` only when a real path can be recovered; otherwise skip image entries.

## UI mapping

| Piece | Suggestion |
|---|---|
| File open | `GtkFileDialog::open_multiple` |
| Thumbnails | `GtkPicture` / `GdkTexture` scaled |
| Active state | CSS class or `GtkListBox` selection + explicit `active` flag in model |
| Rename | `GtkEditableLabel` or existing inline-edit pattern from Phase 3 |
| Presenter BG | Two stacked `GtkPicture` in a `GtkOverlay` / `GtkStack`-like custom box; animate `opacity` |

## Reduced Motion

Query `GtkSettings` / Accessibility reduced-motion (GNOME 50 setting). If reduced motion:

- Switch background instantly, or use a very short fade (≤100ms).

## Work items

1. Replace Presenter & Media stub with image list + Add Images.
2. Implement activate + rename + persist.
3. Implement Presenter dual-layer background + fade helper.
4. Wire active image changes through `StateStore` notifications.
5. Handle missing files gracefully (log; skip texture; don’t crash).
6. Manual check under Flatpak sandbox file access.

## Verification

- [ ] Add multiple images; thumbnails show
- [ ] Activating one updates Presenter background
- [ ] Switching active images crossfades (or instant with reduced motion)
- [ ] Rename persists across restart
- [ ] Missing file does not break Presenter list
- [ ] Tauri image flow still works with its own store

## Exit criteria

Media parity complete; GTK app matches Tauri Console+Presenter feature set for images.
