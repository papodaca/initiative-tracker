# Design: Replace Bootstrap with Tailwind

**Date:** 2026-07-14
**Status:** Approved
**Goal:** Replace Bootstrap 5.3.2 with Tailwind CSS v4 in the Initiative Tracker Tauri/Svelte app, and add a system-aware dark-mode toggle that was not previously possible with the hardcoded `data-bs-theme="dark"`.

## Context

- Tauri 2 + Svelte 5 + Vite 8 desktop app with two windows: Console (`console.html` / `Console.svelte`) and Presenter (`presenter.html` / `Presenter.svelte`).
- ~684 lines of Svelte across 5 files: `Console.svelte`, `Presenter.svelte`, `components/{PlayerList,ImageList,InPlaceEdit}.svelte`.
- Bootstrap 5.3.2 loaded via CSS import in `console.js` and `presenter.js`. No Bootstrap JS, no `@popperjs/core` direct usage (peer dep only). `data-bs-theme="dark"` hardcoded on `<html>` in both HTML files; no toggle.
- State already syncs across windows via `emit('state-change', ...)` in `store.js` + `listen('state-change', ...)` in `Presenter.svelte`. Theme preference can piggyback on this.
- Bootstrap classes in use: `btn`, `btn-{primary,success,danger,info,sm,outline-danger}`, `form-control`, `list-group`, `list-group-item`, `list-group-item-action`, `active`, `text-danger`. CSS vars `--bs-body-bg`, `--bs-active-bg` used in `PlayerList.svelte`.
- No test framework; verification is build + manual.

## Decisions

- **Styling approach:** `@apply` component classes — keep existing class names in templates (e.g. `class="btn btn-primary"`), define them with `@apply` in CSS. Minimal template diff.
- **Palette:** Tailwind defaults (blue-600, green-600, red-600, cyan-500, slate-900). Not a pixel-match of Bootstrap.
- **Dark mode:** New feature. `data-theme="light|dark"` attribute on `<html>` driven by a `src/theme.js` module. Default follows `prefers-color-scheme`; toggle cycles `system → light → dark → system`. Preference persisted via the existing Tauri store and synced to the Presenter window through the existing `state-change` event.
- **Execution:** Big-bang single PR — install Tailwind, rewrite all 5 Svelte files' styling, add the toggle, remove Bootstrap, in one change set.

## Architecture

### Toolchain & CSS

- Add dev deps `tailwindcss` + `@tailwindcss/vite` (v4). v4 auto-detects content via the Vite module graph (no `content` array).
- Register `tailwindcss()` in `vite.config.js` `plugins` alongside `svelte()`.
- New `src/app.css`:
  - `@import "tailwindcss";`
  - `@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *));` — attribute-based dark variant driven from JS so the toggle can override the system.
  - `@layer base` — body/surface defaults for light + dark.
  - `@layer components` — `@apply` definitions for `.btn`, `.btn-*`, `.form-control`, `.list-group*`, `.list-group-item-action`, `.text-danger`, `.active`.
- Replace `import 'bootstrap/...'` in `console.js` and `presenter.js` with `import "./app.css"`. FontAwesome import unchanged.

### Theme module (`src/theme.js`)

- `getStoredTheme()` / `setStoredTheme("light"|"dark"|"system")` — read/write the preference field on the existing Tauri store state.
- `getSystemTheme()` → `window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"`, with a `change` listener used while preference is `"system"`.
- `getEffectiveTheme(preference)` → preference or system.
- `applyTheme(t)` → set `document.documentElement.dataset.theme = t`.
- `setTheme(preference)` → persist preference, apply effective theme, notify subscribers.
- Exposes a small reactive `theme` store (Svelte `$state`) for the toggle UI.
- On boot, `theme.js` applies the effective theme as early as possible to avoid a flash; the HTML files carry no theme attribute (or a neutral inline default).

### Cross-window sync

- Theme preference stored as `state.theme` inside the existing `state` object in `store.js` (alongside `currentCampaign`, `dislaySize`, etc.).
- `Console.svelte` toggle calls `setTheme` → updates `state.theme` → `broadcastState()` → `emit('state-change')`.
- `Presenter.svelte` already listens for `state-change`; on each event it calls `applyTheme(getEffectiveTheme(state.theme))`.
- Initial load: both windows read `state.theme` from store on mount and apply.

### Toggle UI

- One button in `Console.svelte` (sun/moon/auto icons via FontAwesome). Cycles `system → light → dark → system`. Shows the current mode in the icon/title.
- No toggle in `Presenter.svelte` (display-only window; follows Console).

### Component class mapping (`@apply`, Tailwind defaults)

| Bootstrap class | Tailwind `@apply` equivalent |
|---|---|
| `.form-control` | input base + `dark:` overrides |
| `.btn` | `inline-flex items-center rounded px-3 py-1.5 text-sm font-medium` + disabled |
| `.btn-primary` | `bg-blue-600 hover:bg-blue-500 ...` |
| `.btn-success` | `bg-green-600 hover:bg-green-500 ...` |
| `.btn-danger` | `bg-red-600 hover:bg-red-500 ...` |
| `.btn-info` | `bg-cyan-500 hover:bg-cyan-400 text-slate-900 ...` |
| `.btn-outline-danger` | border + text red, hover fill |
| `.btn-sm` | smaller padding/text |
| `.list-group` | `flex flex-col gap-1` |
| `.list-group-item` | surface bg + border + `dark:` overrides |
| `.list-group-item.active` | blue bg |
| `.list-group-item-action` | hover affordance |
| `.text-danger` | `text-red-600 dark:text-red-400` |

- `PlayerList.svelte` scoped `<style>` replaces `--bs-body-bg` / `--bs-active-bg` with new CSS vars (`--surface`, `--surface-active`) or `dark:` utilities.
- Templates keep their existing class names; only the CSS definitions change.

### Bootstrap removal

- Remove `bootstrap` and `@popperjs/core` from `package.json`.
- Remove the two `import 'bootstrap/...'` lines.
- Remove `data-bs-theme="dark"` from `console.html` and `presenter.html`.
- Remove `--bs-*` references in `PlayerList.svelte`.
- `yarn install` to update lockfile.

## Verification

No tests; verification is build + manual:
- `yarn build` passes for both entry points.
- `yarn tauri dev` runs; Console + Presenter render.
- Visual check: buttons, list, form controls, in-place edit, image list.
- Dark mode: toggle cycles correctly; first load with no stored pref follows system; preference persists across restart; Presenter syncs on toggle; system preference change updates while in `"system"` mode.
- `rg "bootstrap|bs-|popper" src` returns no hits.

## Out of scope

- Pixel-perfect match to the Bootstrap look.
- Light-mode polish for the Presenter window beyond what the dark variant provides.
- Refactoring component structure or props — class names and templates stay the same.
