#!/usr/bin/env just --justfile

carla_repo          := "https://github.com/carla-simulator/carla.git"

local_build          := main_repo/"carla-setup/localBuild"
carla_source        := "~"/local_build/"carla-src"
carla_build         := carla_source/"Build"

carla_release       := "0.9.15"
carla_release_tag   := carla_release + ".2"

python_deps         := "pygame numpy shapely networkx"

ego_vehicle_bp      := "vehicle.audi.etron"
