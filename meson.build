
project('MXL Plyr',
        'rust',
        version: '3.0.0',
        meson_version: '>= 0.59.0',
        # license: 'GPL-3.0-or-later'
        default_options : ['buildtype=debug',
                           'rust_std=2021'
                          ]

)

i18n = import('i18n')
gnome = import('gnome')

datadir = get_option('prefix') / get_option('datadir')
iconsdir = datadir / 'icons'
plugins_install_dir = get_option('libdir') / 'gstreamer-1.0'
target_dir = 'target'

base_id = 'com.x-software.mxl'

cargo = find_program('cargo', required: true)

run_command('./scripts/setup-gdkpixbuf.sh', meson.project_build_root() / 'lib', check: true)
run_command('./scripts/setup-gdkpixbuf.sh', meson.project_build_root() / target_dir / 'lib', check: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
licensesdir = datadir / 'licenses'

package_name_full = 'MXL-Plyr'
message('Build package', package_name_full)
package_name = 'mxl-plyr'
package_id = 'mxl_plyr'

message('Building', package_name_full)

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>= 1.0.0')
dependency('gstreamer-1.0', version: '>= 1.0.0')

glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)

if host_machine.system() == 'linux'
  update_desktop_database = true
  gtk_update_icon_cache = true
else
  update_desktop_database = false
  gtk_update_icon_cache = false
endif

if get_option('profile') == 'development'
  gnome.post_install(
    glib_compile_schemas: true,
    gtk_update_icon_cache: gtk_update_icon_cache,
    update_desktop_database: update_desktop_database,
  )
endif

podir = meson.current_source_dir() / 'po'

application_id = '@0@.plyr'.format(base_id)

binary_output_file = 'mxl_plyr'

subdir('data' / 'resources')

third_party_license_file = '@0@_third_party_licenses.json'.format(binary_output_file)
run_command('cargo', 'bundle-licenses', '--format', 'json', '--output', third_party_license_file,
  check : true)
install_data(third_party_license_file, install_dir: licensesdir)

cargo_options = [ '--manifest-path', meson.current_source_dir() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / target_dir ]

if get_option('buildtype') == 'release'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: binary_output_file,
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    cargo, 'build',
    cargo_options,
    '&&', 'cp', target_dir / rust_target / binary_output_file, '@OUTPUT@',
  ]
)
