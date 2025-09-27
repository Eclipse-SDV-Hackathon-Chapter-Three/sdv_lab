#!/usr/bin/env just --justfile

server_host     := "ubuntu"
server_dir      := "~/carla-simulator"
server_width    := "800"
server_height   := "600"

carla_url       := "https://tiny.carla.org"
carla_download  := carla_url/"carla-" + replace(carla_release, ".", "-") + "-linux"
carla_file      := "~/CARLA_" + carla_release + ".tar.gz"
carla_md5       := "32fa681fb925cd62951a63977582ea8c"
carla_server    := server_dir/"CarlaUE4.sh"

# Download a specific release of the CARLA simulator
_download_carla: (_check_host server_host)
  @just _download_file {{ carla_download }} {{ carla_file }} {{ carla_md5 }}

# Extract the downloaded release
_extract_carla: (_check_host server_host)
  @just _extract_file {{ carla_file }} {{ server_dir }}

[group('Carla Server')]
[doc('Install the CARLA Simulator')]
install-server: (_check_host server_host)
  # Download carla
  @just _download_carla
  # Install carla
  @just _extract_carla

[group('Carla Server')]
[doc('Run CARLA off-screen using NVIDIA card')]
server-nvidia quality="Epic" port="2000": (_check_host server_host)
  # Run carla
  @{{ carla_server }} \
    -prefernvidia \
    -quality-level={{ quality }} \
    -carla-rpc-port={{ port }} \
    -RenderOffScreen


[group('Carla Server')]
[doc('Run CARLA in off-screen mode')]
server-offscreen quality="Epic" port="2000": (_check_host server_host)
  # Run carla
  @{{ carla_server }} \
    -quality-level={{ quality }} \
    -carla-rpc-port={{ port }} \
    -RenderOffScreen

[group('Carla Server')]
[doc('Run CARLA in windowed mode')]
server-windowed quality="Epic" port="2000": (_check_host server_host)
  # Run carla
  @{{ carla_server }} \
    -quality-level={{ quality }} \
    -carla-rpc-port={{ port }} \
    -windowed -ResX={{ server_width }} -ResY={{ server_height }}

[group('Carla Server')]
[doc('Uninstall CARLA Simulator')]
[confirm("This action will REMOVE your CARLA folder.\nAre you sure you want to continue? [y/N]")]
uninstall-server: (_check_host server_host)
  # Uninstall carla
  @rm -rvf {{ server_dir }}
