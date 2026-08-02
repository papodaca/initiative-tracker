application_id = 'im.apodaca.InitiativeTracker'

foreach size : ['32x32', '128x128', '512x512']
  install_data(
    'hicolor' / size / 'apps' / (application_id + '.png'),
    install_dir: get_option('datadir') / 'icons' / 'hicolor' / size / 'apps',
  )
endforeach

install_data(
  'hicolor' / 'scalable' / 'apps' / (application_id + '.svg'),
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps',
)

install_data(
  'hicolor' / 'symbolic' / 'apps' / (application_id + '-symbolic.svg'),
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'symbolic' / 'apps',
)
