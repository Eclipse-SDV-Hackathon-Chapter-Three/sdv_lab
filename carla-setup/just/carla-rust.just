#!/usr/bin/env just --justfile

rust_host     := "ubuntu"
rust_env      := "$HOME/.cargo/env"

rust_dir      := "~"/local_build/"carla-rust"
rust_repo     := "https://github.com/jerry73204/carla-rust.git"
rust_tag      := "master"
rust_deps     := "libpng-dev libssl-dev pkg-config"
rust_patch    := replace(rust_dir, "~/" + local_build + "/", "") + ".patch"
rust_build    := "cargo build --release"
rust_lib      := rust_dir + "/target/release/libcarla.rlib"

recast_target := "recast-c" + clang_version + "-install"
recast_link   := "recast-0b13b0-c" + clang_version + "-install"

# Clone a specific version of the CARLA rust bridge
_clone_carla_rust: (_check_host rust_host)
  @just _clone_repo {{ rust_repo }} {{ rust_tag }} {{ rust_dir }}

# Install Rust Language tools
_install_rust: (_check_host rust_host)
  # Run rustup
  @curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

[group('Carla Build')]
[doc('Build CARLA API (Rust)')]
make-carla: (_check_host rust_host)
  # Clone carla
  @if test ! -d {{ rust_dir }}; then just _clone_carla_rust; fi
  # Install dependencies
  @sudo apt install -y {{ rust_deps }}
  # Ensure libcarla
  @just build-libcarla
  # Ensure links
  @cd {{ carla_build }} && unlink {{ recast_link }} || true
  @cd {{ carla_build }} && ln -s {{ recast_target }} {{ recast_link }}
  # Ensure rust
  @just _install_rust
  # Apply patch
  @cp {{ source_directory() }}/{{ rust_patch }} {{ rust_dir }}
  @cd {{ rust_dir }} && git apply {{ rust_patch }} || true
  @cd {{ rust_dir }} && rm -rf .git
  # Build carla-rust
  @. {{ user_bashrc }} && . {{ rust_env }} && cd {{ rust_dir }} && {{ rust_build }}
  # Show libcarla
  @just _list_file {{ rust_lib }}
