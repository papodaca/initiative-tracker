# Initiative Tracker (GTK)

GTK4 / libadwaita frontend for Initiative Tracker, targeting **GNOME 50**.
This lives alongside the existing Tauri/Svelte app; both are supported.

Application id: `im.apodaca.InitiativeTracker`  
Binary: `initiative-tracker-gtk`

## Native build (Meson + Cargo)

Requirements: Rust toolchain, GTK 4.22+, libadwaita 1.9+, Meson, Blueprint.

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
cargo run
```

## Flatpak (GNOME 50)

Requires `flatpak` and `flatpak-builder`, plus the GNOME 50 SDK:

```bash
flatpak install --user flathub org.gnome.Sdk//50 org.gnome.Platform//50 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08
```

Build and install from the `gtk/` directory:

```bash
cd gtk
flatpak-builder --user --install --force-clean flatpak-build \
  flatpak/im.apodaca.InitiativeTracker.json
flatpak run im.apodaca.InitiativeTracker
```

## Tauri frontend (unchanged)

From the repository root:

```bash
yarn install
yarn tauri dev
```

The Tauri identifier remains `im.apodaca.initiative-tracker`, so both apps can be installed side by side.
