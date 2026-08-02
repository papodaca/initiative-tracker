desktop_file = configure_file(
  input: 'im.apodaca.InitiativeTracker.desktop.in',
  output: 'im.apodaca.InitiativeTracker.desktop',
  copy: true,
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = configure_file(
  input: 'im.apodaca.InitiativeTracker.metainfo.xml.in',
  output: 'im.apodaca.InitiativeTracker.metainfo.xml',
  copy: true,
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test(
    'Validate appstream file',
    appstreamcli,
    args: ['validate', '--no-net', '--explain', appstream_file],
  )
endif

subdir('icons')
