# ![App Icon](src-tauri/icons/Square71x71Logo.png) Initiative Tracker

Track initiative for your campaigns on a secondary monitor.

* Track initiative of players, monsters and NPCs
* Track players, monsters and NPCs health
* Display images to set the scene of the locations your players explore.
* Persist state of the game.
* Quickly switch between multiple campaigns easily.

![App Screenshot](.github/screenshot.png)


## Frontends

This repository ships **two frontends** in parallel during the GTK port:

| Frontend | Stack | Status | How to run |
|---|---|---|---|
| **Tauri** | Tauri 2 + Svelte | Production path today | `yarn tauri dev` / `yarn tauri build` |
| **GTK** | GTK4 + libadwaita (GNOME 50) | Parallel preview (feature parity) | see [`gtk/README.md`](gtk/README.md) |

Cutover / removal of Tauri is **not** part of the current plan set; see [`docs/plans/gtk/`](docs/plans/gtk/).

## Development

### Tauri / Svelte

1. Install [asdf](https://asdf-vm.com/).
2. Install tools
```bash
asdf install
```
3. Install dependencies
```bash
yarn install
```
4. Run development
```bash
yarn tauri dev
```
5. Production build
```bash
yarn tauri build
```

### GTK4 / libadwaita

```bash
cd gtk
meson setup build
meson compile -C build
./build/src/initiative-tracker-gtk
```

Full Flatpak packaging, sandbox notes, and shortcuts: [`gtk/README.md`](gtk/README.md).
