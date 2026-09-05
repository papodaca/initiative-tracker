project(
  'initiative-tracker-gtk',
  'rust',
  version: '0.1.0',
  meson_version: '>= 1.0.0',
  default_options: ['warning_level=2', 'werror=false'],
)

gnome = import('gnome')

application_id = 'im.apodaca.InitiativeTracker'

# Ensure Blueprint is available for UI files (Phase 0 stub + later phases).
blueprint_compiler = find_program('blueprint-compiler', required: true)

subdir('data')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
