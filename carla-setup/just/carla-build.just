#!/usr/bin/env just --justfile

build_host      := "ubuntu"
build_target    := "LibCarla.client.release"
build_release   := carla_build/"libcarla-client-build.release"
build_output    := build_release/"LibCarla/cmake/client/libcarla_client.a"

clang_version   := "12"
clang_package   := "clang-" + clang_version
clang_library   := "libclang-" + clang_version + "-dev"
clang_system    := "/usr/local/bin/clang"
clang_system_pp := "/usr/local/bin/clang++"
clang_target    := "/usr/bin/clang-" + clang_version
clang_target_pp := "/usr/bin/clang++-" + clang_version
clang_deps      := clang_package + " " + clang_library + " "

# From <https://github.com/jerry73204/carla-rust>
llvm_config     := "LLVM_CONFIG_PATH=/usr/bin/llvm-config-" + clang_version
libclang_path   := "LIBCLANG_PATH=/usr/lib/llvm-" + clang_version + "/lib"
libclang_static := "LIBCLANG_STATIC_PATH=" + libclang_path
target_clang    := "CLANG_PATH=" + clang_target
carla_src_dir   := "CARLA_DIR=" + carla_source

source_deps     := clang_deps + "autoconf build-essential cmake g++-12 libtool ninja-build"
source_patch    := replace(carla_source, "~/" + local_build + "/", "") + ".patch"

pip_list_user   := "/tmp/pip-list-user"

# Clone a specific version of the CARLA source code
_clone_carla_src: (_check_host build_host)
  @just _clone_repo {{ carla_repo }} {{ carla_release_tag }} {{ carla_source }}

# Install Python dependencies back
_install_python_deps: (_check_host build_host)
  #!/usr/bin/env -S bash -x
  set {{ no_errors }}
  had_deps=false
  python_deps="{{ python_deps }}"
  deps=($python_deps)
  for d in "${deps[@]}" ; do
    result=`cat {{ pip_list_user }} | grep $d || true`
    if [[ -n "$result" ]]; then
      had_deps=true
      break
    fi
  done
  if [[ $had_deps == true ]]; then
    pip install --user {{ python_deps }}
  fi

# Uninstall Python dependencies to avoid build issues
_uninstall_python_deps: (_check_host build_host)
  #!/usr/bin/env -S bash -x
  set {{ no_errors }}
  has_deps=false
  python_deps="{{ python_deps }}"
  deps=($python_deps)
  pip list --user > {{ pip_list_user }}
  for d in "${deps[@]}" ; do
    result=`cat {{ pip_list_user }} | grep $d || true`
    if [[ -n "$result" ]]; then
      has_deps=true
      break
    fi
  done
  if [[ $has_deps == true ]]; then
    pip uninstall {{ python_deps }} --yes
  fi

[group('Carla Build')]
[doc('Build CARLA API (C++)')]
build-libcarla: (_check_host build_host)
  # Avoid issues
  @just _uninstall_python_deps
  # Clone carla
  @if test ! -d {{ carla_source }}; then just _clone_carla_src; fi
  # Install dependencies
  @sudo apt install -y {{ source_deps }}
  # Ensure links
  @sudo unlink {{ clang_system }} || true
  @sudo ln -s {{ clang_target }} {{ clang_system }}
  @sudo unlink {{ clang_system_pp }} || true
  @sudo ln -s {{ clang_target_pp }} {{ clang_system_pp }}
  # Prepare bashrc
  @sed -i '/.*-{{ clang_version }}/d' {{ user_bashrc }}
  @echo "# Clang-{{ clang_version }}" >> {{ user_bashrc }}
  @echo "export {{ llvm_config }}" >> {{ user_bashrc }}
  @echo "export {{ libclang_path }}" >> {{ user_bashrc }}
  @echo "export {{ libclang_static }}" >> {{ user_bashrc }}
  @echo "export {{ target_clang }}" >> {{ user_bashrc }}
  # Update setup
  @cp {{ source_directory() }}/{{ source_patch }} {{ carla_source }}
  @cd {{ carla_source }} && git apply {{ source_patch }} || true
  @cd {{ carla_source }} && rm -rf .git
  # Build carla
  @. {{ user_bashrc }} && cd {{ carla_source }} && make {{ build_target }}
  # Show libcarla
  @just _list_file {{ build_output }}
  # Update bashrc
  @sed -i '/CARLA/d' {{ user_bashrc }}
  @echo "export {{ carla_src_dir }}" >> {{ user_bashrc }}
  @sed -i '${/^$/d;}' {{ user_bashrc }}
  # Restore dependencies
  @just _install_python_deps