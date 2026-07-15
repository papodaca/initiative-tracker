# Tailwind Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace Bootstrap 5.3.2 with Tailwind CSS v4 and add a system-aware dark-mode toggle synced across the Console and Presenter windows.

**Architecture:** Single `src/app.css` with `@import "tailwindcss"`, an attribute-based `@custom-variant dark`, and `@apply` component classes that keep the existing Bootstrap class names in templates. A new `src/theme.js` exposes pure helpers; the Console component owns preference state (stored in the existing Tauri store `state.theme`) and the Presenter applies theme on each `state-change` event it already listens to.

**Tech Stack:** Svelte 5 (runes), Vite 8, Tailwind v4 + `@tailwindcss/vite`, Tauri 2, FontAwesome (unchanged).

**No test framework in this project.** Each task ends with `yarn build` (and `yarn tauri dev` where noted) plus a commit. Final task adds an `rg` grep to confirm no Bootstrap references remain.

**Design doc:** `docs/plans/2026-07-14-tailwind-migration-design.md`

---

### Task 1: Install Tailwind, configure Vite, create app.css, swap entry imports

**Files:**
- Modify: `package.json` (add devDeps)
- Modify: `vite.config.js`
- Create: `src/app.css`
- Modify: `src/console.js:2`
- Modify: `src/presenter.js:2-4`

**Step 1: Add Tailwind v4 + Vite plugin**

Run:
```bash
yarn add -D tailwindcss @tailwindcss/vite
```
Expected: `package.json` gains `"tailwindcss"` and `"@tailwindcss/vite"` in `devDependencies`; `yarn.lock` updated.

**Step 2: Register the Tailwind Vite plugin**

Replace `vite.config.js` contents with:

```js
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  build: {
    rollupOptions: {
      input: {
        console: "./console.html",
        presenter: "./presenter.html",
      },
    }
  },
  plugins: [svelte(), tailwindcss()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

**Step 3: Create `src/app.css` with the dark variant and `@apply` component classes**

Create `src/app.css`:

```css
@import "tailwindcss";

/* Attribute-based dark variant driven from JS (data-theme="light|dark" on <html>),
   so the toggle can override prefers-color-scheme. */
@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *));

@layer base {
  body {
    @apply bg-slate-50 text-slate-900 dark:bg-slate-900 dark:text-slate-100;
    margin: 0;
  }
}

@layer components {
  .form-control {
    @apply rounded border border-slate-300 bg-white px-2 py-1 text-sm text-slate-900 outline-none
           dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100;
  }

  .btn {
    @apply inline-flex items-center rounded px-3 py-1.5 text-sm font-medium cursor-pointer
           border border-transparent;
  }
  .btn:disabled,
  .btn[disabled] {
    @apply opacity-50 cursor-not-allowed;
  }

  .btn-primary {
    @apply bg-blue-600 text-white hover:bg-blue-500;
  }
  .btn-success {
    @apply bg-green-600 text-white hover:bg-green-500;
  }
  .btn-danger {
    @apply bg-red-600 text-white hover:bg-red-500;
  }
  .btn-info {
    @apply bg-cyan-500 text-slate-900 hover:bg-cyan-400;
  }
  .btn-outline-danger {
    @apply border border-red-600 text-red-600 bg-transparent hover:bg-red-600 hover:text-white;
  }
  .btn-sm {
    @apply px-2 py-1 text-xs;
  }

  .list-group {
    @apply flex flex-col gap-1;
  }
  .list-group-item {
    @apply rounded border border-slate-200 bg-white px-3 py-2 text-slate-900
           dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100;
  }
  .list-group-item-action {
    @apply cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-700;
  }
  .list-group-item.active {
    @apply bg-blue-600 text-white border-blue-600 hover:bg-blue-600 dark:hover:bg-blue-600;
  }

  .text-danger {
    @apply text-red-600 dark:text-red-400;
  }
}
```

**Step 4: Swap the Bootstrap import in `console.js`**

Replace `src/console.js` with:

```js
import { mount } from 'svelte'
import './app.css'
import '@fortawesome/fontawesome-free/css/all.min.css'
import { applyTheme } from './theme'
import Console from './Console.svelte'

applyTheme('system')

const app = mount(Console, {
  target: document.getElementById('app')
})

export default app
```

Note: `./theme` is created in Task 2. The build in Step 6 will fail until Task 2 lands, so do not build between Task 1 and Task 2. (Alternatively, defer the `import { applyTheme }` line to Task 2 — but keeping it here is fine since Task 2 immediately follows.)

**Step 5: Swap the Bootstrap import in `presenter.js`**

Replace `src/presenter.js` with:

```js
import { mount } from 'svelte'
import './app.css'
import '@fortawesome/fontawesome-free/css/all.min.css'
import './presenter.css'
import { applyTheme } from './theme'
import Presenter from './Presenter.svelte'

applyTheme('system')

const app = mount(Presenter, {
  target: document.getElementById('app')
})

export default app
```

**Step 6: Do NOT build yet** — `src/theme.js` does not exist. Proceed directly to Task 2.

---

### Task 2: Add `src/theme.js` with pure theme helpers

**Files:**
- Create: `src/theme.js`

**Step 1: Create the module**

Create `src/theme.js`:

```js
// Pure theme helpers. Preference ("light" | "dark" | "system") is owned by the
// Console component and persisted on the existing Tauri store state as `state.theme`.
// The Presenter applies the effective theme on each `state-change` event.

export const getSystemTheme = () => {
  if (typeof window === "undefined" || !window.matchMedia) return "dark"
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

export const getEffectiveTheme = (preference) =>
  preference === "light" || preference === "dark" ? preference : getSystemTheme()

export const applyTheme = (preference) => {
  if (typeof document === "undefined") return
  document.documentElement.dataset.theme = getEffectiveTheme(preference)
}

// Subscribe to OS preference changes. Returns an unsubscribe function.
export const watchSystemTheme = (callback) => {
  if (typeof window === "undefined" || !window.matchMedia) return () => {}
  const mq = window.matchMedia("(prefers-color-scheme: dark)")
  const handler = () => callback(getSystemTheme())
  mq.addEventListener("change", handler)
  return () => mq.removeEventListener("change", handler)
}
```

**Step 2: Verify build**

Run:
```bash
yarn build
```
Expected: build succeeds for both `console` and `presenter` entry points. Tailwind classes are generated; the app may still look partly unstyled because templates still reference Bootstrap class names that are now defined via `@apply` (so it should actually look close to correct). No Bootstrap CSS is loaded.

If build fails on the `dark:` variant in `@apply`, confirm `@custom-variant dark` is placed immediately after `@import "tailwindcss";` and before `@layer components`.

**Step 3: Commit**

```bash
git add package.json yarn.lock vite.config.js src/app.css src/console.js src/presenter.js src/theme.js
git commit -m "Add Tailwind v4, replace Bootstrap CSS import, add theme helpers"
```

---

### Task 3: Wire theme into Console (load, toggle, system watch) and HTML

**Files:**
- Modify: `console.html:2`
- Modify: `src/Console.svelte`

**Step 1: Replace the HTML theme attribute**

In `console.html`, change line 2 from:
```html
<html lang="en" data-bs-theme="dark">
```
to:
```html
<html lang="en" data-theme="dark">
```
(The `dark` default prevents a light flash on first paint; `theme.js` corrects it on load.)

**Step 2: Add theme imports and state to `Console.svelte`**

In `src/Console.svelte`, in the `<script>` block, add to the existing imports (after the `./utils` import on line 10):

```js
  import { applyTheme, watchSystemTheme } from "./theme"
```

**Step 3: Add a `stopSystemWatcher` variable**

Just below `let appWindow` (around line 19), add:

```js
  let stopSystemWatcher = null
```

**Step 4: Initialize theme in `loadState`**

In the `loadState` function (around line 50-71), after the `if (state == null) state = {}` line, add a theme-default block alongside the existing `dislaySize`/`currentCampaign` defaults. Insert after the `if (state == null) state = {}` line:

```js
    let themeChanged = false
    if (state.theme == null) {
      state.theme = "system"
      themeChanged = true
    }
```

Then, at the end of `loadState`, after the existing `if (changed) broadcastState()` line, add:

```js
    if (themeChanged) broadcastState()
    applyTheme(state.theme)
    stopSystemWatcher?.()
    if (state.theme === "system") {
      stopSystemWatcher = watchSystemTheme(() => applyTheme("system"))
    }
```

**Step 5: Add the `cycleTheme` function**

Just above `const broadcastState = () => setStoreState(state)` (around line 199), add:

```js
  const cycleTheme = (_e) => {
    const order = ["system", "light", "dark"]
    const idx = order.indexOf(state.theme)
    const next = order[(idx + 1) % order.length]
    state = { ...state, theme: next }
    applyTheme(state.theme)
    stopSystemWatcher?.()
    stopSystemWatcher = state.theme === "system" ? watchSystemTheme(() => applyTheme("system")) : null
    broadcastState()
  }
```

**Step 6: Add the toggle button to the template**

In the template, add the toggle button as the first button on the top row (before the `Open Presenter` button, around line 247). Insert before `<button class="btn btn-primary" onclick={openPresenter} ...>`:

```svelte
<button class="btn btn-primary" onclick={cycleTheme} title="Theme: {state.theme}">
  {#if state.theme === "light"}
    <i class="fa-solid fa-sun"></i>
  {:else if state.theme === "dark"}
    <i class="fa-solid fa-moon"></i>
  {:else}
    <i class="fa-solid fa-circle-half-stroke"></i>
  {/if}&nbsp;{toTitleCase(state.theme)}
</button>
```

**Step 7: Verify build**

Run:
```bash
yarn build
```
Expected: build succeeds.

**Step 8: Manual check (optional, requires Tauri)**

Run `yarn tauri dev`. Confirm: toggle button cycles system → light → dark; the `<html data-theme>` attribute updates; on first launch with no stored preference the theme follows the OS. Skip if Tauri toolchain is unavailable — Task 6 includes the final manual check.

**Step 9: Commit**

```bash
git add console.html src/Console.svelte
git commit -m "Wire theme toggle into Console and follow system preference"
```

---

### Task 4: Apply theme in the Presenter window

**Files:**
- Modify: `presenter.html:2`
- Modify: `src/Presenter.svelte`

**Step 1: Replace the HTML theme attribute**

In `presenter.html`, change line 2 from:
```html
<html lang="en" data-bs-theme="dark">
```
to:
```html
<html lang="en" data-theme="dark">
```

**Step 2: Import `applyTheme` in `Presenter.svelte`**

In `src/Presenter.svelte`, add to the imports at the top of the `<script>` block (after the `./store` import on line 8):

```js
  import { applyTheme } from "./theme"
```

**Step 3: Apply theme on state change**

In the `incomingState` function (around line 14-20), add an `applyTheme` call. The function currently sets the body background image; add theme application at the end:

```js
  const incomingState = async (s) => {
    state = s
    applyTheme(state.theme)
    const currentImage = state[state.currentCampaign].images.find(i => i.active)
    if (currentImage) {
      document.body.setAttribute("style", `--bg-image: url('${currentImage.fileUrl}')`)
    }
  }
```

Note: `applyTheme` safely handles `undefined`/`null` (falls back to system), so no guard is needed even before state arrives. The `applyTheme('system')` call added in Task 1's `presenter.js` already sets a sensible default before mount.

**Step 4: Verify build**

Run:
```bash
yarn build
```
Expected: build succeeds.

**Step 5: Commit**

```bash
git add presenter.html src/Presenter.svelte
git commit -m "Apply synced theme in Presenter window on state-change"
```

---

### Task 5: Replace Bootstrap CSS vars in PlayerList

**Files:**
- Modify: `src/components/PlayerList.svelte:75-110`

**Step 1: Replace the `--bs-*` references**

In `src/components/PlayerList.svelte`, the scoped `<style>` block (lines 75-110) references `var(--bs-body-bg)` (line 92) and `var(--bs-active-bg)` (line 96). Replace those two declarations:

Change:
```css
  .list-group.initiative-list .list-group-item div.text {
    opacity: 0.9;
    background-color: var(--bs-body-bg);
  }
  .list-group.initiative-list .list-group-item.active div.text {
    opacity: 0.9;
    background-color: var(--bs-active-bg);
  }
```
to:
```css
  .list-group.initiative-list .list-group-item div.text {
    opacity: 0.9;
    background-color: theme(--color-slate-50);
  }
  :global([data-theme="dark"]) .list-group.initiative-list .list-group-item div.text {
    background-color: theme(--color-slate-800);
  }
  .list-group.initiative-list .list-group-item.active div.text {
    opacity: 0.9;
    background-color: theme(--color-blue-600);
  }
```

Rationale: these rules layer a translucent surface behind the active item's text. `theme(--color-slate-50)` / `theme(--color-slate-800)` mirror the body/surface token in light/dark; `theme(--color-blue-600)` matches the `.active` background set in `app.css`. `:global([data-theme="dark"])` is needed because Svelte scopes `.` selectors and the attribute lives on `<html>`. (`theme()` is Tailwind v4's function to read `@theme` tokens from inside CSS.)

**Step 2: Verify build**

Run:
```bash
yarn build
```
Expected: build succeeds. If `theme()` is unresolved, swap to literal hex values: `#f8fafc` (slate-50), `#1e293b` (slate-800), `#2563eb` (blue-600).

**Step 3: Commit**

```bash
git add src/components/PlayerList.svelte
git commit -m "Replace Bootstrap CSS vars with Tailwind tokens in PlayerList"
```

---

### Task 6: Remove Bootstrap and Popper, verify no references remain

**Files:**
- Modify: `package.json:14-15`

**Step 1: Remove the Bootstrap and Popper deps**

Run:
```bash
yarn remove bootstrap @popperjs/core
```
Expected: `package.json` no longer lists `bootstrap` or `@popperjs/core`; `yarn.lock` updated; `node_modules/bootstrap` and `node_modules/@popperjs/core` removed.

**Step 2: Confirm no stale references in source**

Run:
```bash
rg "bootstrap|bs-|popper" src --no-heading
```
Expected: no output. (FontAwesome `fa-` classes are unrelated and fine.)

If any hits appear, remove them before continuing.

**Step 3: Verify production build**

Run:
```bash
yarn build
```
Expected: build succeeds for both `console` and `presenter` chunks; no errors about unresolved `bootstrap` imports.

**Step 4: Manual verification (requires Tauri toolchain)**

Run `yarn tauri dev` and confirm:
- Console window: buttons (primary/success/danger/info), form controls, the player list, the image list, in-place edit inputs all render with Tailwind styling.
- Theme toggle button cycles system → light → dark; `<html data-theme>` updates; persists across restart; follows OS while in "system".
- Presenter window: renders; theme matches Console; toggling Console's theme updates Presenter live.

If Tauri is unavailable, at minimum run `yarn build` (done above) and `yarn vite preview` to smoke-check the built assets.

**Step 5: Commit**

```bash
git add package.json yarn.lock
git commit -m "Remove Bootstrap and Popper dependencies"
```

---

## Done criteria

- `yarn build` passes.
- `rg "bootstrap|bs-|popper" src` returns nothing.
- `bootstrap` and `@popperjs/core` absent from `package.json`.
- `data-theme="light|dark"` on `<html>` in both windows; Console toggle cycles and persists; Presenter follows.
- No white flash on launch (HTML default `data-theme="dark"`).

## Out of scope

- Pixel-perfect match to the previous Bootstrap look.
- Light-mode polish for the Presenter beyond the dark variant.
- Refactoring component structure or props.
- A persistent "system" indicator beyond the toggle's icon/label.
