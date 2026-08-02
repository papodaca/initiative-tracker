# ![App Icon](src-tauri/icons/Square71x71Logo.png) Initiative Tracker

Track initiative for your campaigns on a secondary monitor.

* Track initiative of players, monsters and NPCs
* Track players, monsters and NPCs health
* Display images to set the scene of the locations your players explore.
* Persist state of the game.
* Quickly switch between multiple campaigns easily.

![App Screenshot](.github/screenshot.png)


## Development

This repository currently ships **two frontends** in parallel:

| Frontend | Stack | How to run |
|---|---|---|
| **Tauri** (production today) | Tauri 2 + Svelte | `yarn tauri dev` |
| **GTK** (port in progress) | GTK4 + libadwaita (GNOME 50) | see [`gtk/README.md`](gtk/README.md) |

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

### GTK4 / libadwaita

```bash
cd gtk
meson setup build
meson compile -C build
./build/src/initiative-tracker-gtk
```

Full Flatpak and packaging notes: [`gtk/README.md`](gtk/README.md).