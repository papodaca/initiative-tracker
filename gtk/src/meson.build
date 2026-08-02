pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()

# Compile Blueprint → GtkBuilder XML for GResource (loaded in later phases).
blueprints = custom_target(
  'blueprints',
  input: files('../data/ui/window.blp'),
  output: 'window.ui',
  command: [
    blueprint_compiler,
    'compile',
    '--output', '@OUTPUT@',
    '@INPUT@',
  ],
)

resources = gnome.compile_resources(
  'resources',
  '../data/resources.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  dependencies: blueprints,
  install: true,
  install_dir: pkgdatadir,
)

cargo_bin = find_program('cargo')
cargo_home = meson.project_build_root() / 'cargo-home'
cargo_target = meson.project_build_root() / 'cargo-target'
manifest = meson.project_source_root() / 'Cargo.toml'

if get_option('buildtype') == 'release'
  rust_target = 'release'
  cargo_mode = '--release'
else
  rust_target = 'debug'
  cargo_mode = ''
endif

cargo_script = '''
set -eu
export CARGO_HOME="$1"
manifest="$2"
target_dir="$3"
mode="$4"
out="$5"
bin_name="$6"
if [ -n "$mode" ]; then
  cargo build --manifest-path "$manifest" --target-dir "$target_dir" "$mode"
else
  cargo build --manifest-path "$manifest" --target-dir "$target_dir"
fi
cp "$target_dir/$7/$bin_name" "$out"
'''

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  depends: resources,
  command: [
    'sh', '-c', cargo_script,
    'sh',
    cargo_home,
    manifest,
    cargo_target,
    cargo_mode,
    '@OUTPUT@',
    meson.project_name(),
    rust_target,
  ],
)
