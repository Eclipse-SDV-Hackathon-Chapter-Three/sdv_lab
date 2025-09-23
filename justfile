#!/usr/bin/env·just·--justfile

# Ignore recipe lines beginning with '#'
set ignore-comments := true

# Optional imports
import? "./carla-setup/carla-build.just"
import? "./carla-setup/carla-rust.just"
import? "./carla-setup/carla-common.just"
import? "./carla-setup/carla-client.just"
import? "./carla-setup/carla-server.just"

# Mandatory imports
import "./just/utils.just"

# Main definitions
main_repo := "sdv_lab"
main_temp := main_repo/"temp"

# Default recipe to display help information
_default:
  @just --list
