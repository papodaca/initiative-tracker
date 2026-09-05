# Initiative Tracker (GTK)

GTK4 / libadwaita frontend for Initiative Tracker, targeting **GNOME 50**.
This lives alongside the existing Tauri/Svelte app; both are supported.

Application id: `im.apodaca.InitiativeTracker`  
Binary: `initiative-tracker-gtk`  
Versioning: GTK preview shares `0.1.0` with a “GTK preview” note in AppStream until a tagged cutover.

## Native build (Meson + Cargo)

Distro packages (names vary): Rust toolchain, **GTK 4.22+**, **libadwaita 1.9+**, Meson, Ninja, Blueprint compiler, `desktop-file-utils`, `appstreamcli` (optional validation).

```bash
cd gtk
meson setup build
meson compile -C build
```

Run the compiled binary (no install required for a smoke test):

```bash
./build/src/initiative-tracker-gtk
```

Optional local install:

```bash
meson setup build --prefix=$HOME/.local
meson install -C build
initiative-tracker-gtk
```

Cargo-only (same binary, skips desktop/metainfo install):

```bash
cd gtk
cargo test
cargo run
```

## Flatpak (GNOME 50)

Requires `flatpak` and `flatpak-builder`, plus the GNOME 50 SDK:

```bash
flatpak install --user flathub org.gnome.Sdk//50 org.gnome.Platform//50 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08
```

Build and install from the repository root:

```bash
flatpak-builder --user --install --force-clean packaging/flatpak/build-dir \
  packaging/flatpak/im.apodaca.InitiativeTracker.json
flatpak run im.apodaca.InitiativeTracker
```

### Sandbox & scene images

The Flatpak finish-args intentionally omit broad home/Pictures access. **Add Images** uses `GtkFileDialog` (document portal); selected files are **copied** into:

`$XDG_DATA_HOME/im.apodaca.InitiativeTracker/images/`

so Presenter thumbnails keep working after restart. Paths imported from the Tauri store that point outside the sandbox may need to be re-added under Flatpak.

## Tauri frontend (unchanged)

From the repository root:

```bash
yarn install
yarn tauri dev
```

The Tauri identifier remains `im.apodaca.initiative-tracker`, so both apps can be installed side by side.

## Keyboard shortcuts (Console)

| Shortcut | Action |
|---|---|
| `Ctrl+N` | Next turn |
| `Ctrl+Shift+N` | Previous turn |
| `Ctrl+Shift+P` | Open Presenter |
| `Ctrl+Shift+F` | Toggle Presenter fullscreen |
| `Ctrl+Q` | Quit (saves state) |

Presenter: `F11` toggles fullscreen; `Esc` exits fullscreen.
