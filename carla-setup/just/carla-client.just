#!/usr/bin/env just --justfile

client_host     := "ubuntu"
client_dir      := "~/carla-api"
client_cache    := "~/carlaCache"
client_width    := "640"
client_height   := "480"

python_api      := "PythonAPI"
python_examples := client_dir/python_api/examples
python_sparse   := "sparse-checkout set --no-cone " + python_api

examples        := "examples"
bridge_examples := source_directory() + "/../" + examples

vehicle_bp      := "vehicle.audi.etron"
vehicle_role    := "ego_vehicle"

example_auto            := "automatic_control_zenoh.py"
example_manual          := "manual_control_zenoh.py"
example_manual_sensors  := "manual_control_sensors.py"
zenoh_vehicle           := "zenoh_vehicle.py"

# Clone a specific version of the CARLA Python API
_clone_carla_api: (_check_host client_host)
  @rm -rf {{ client_dir }}
  @just _clone_repo_sparse {{ carla_repo }} {{ carla_release_tag }} {{ client_dir }}
  @cd {{ client_dir }} && git {{ python_sparse }}
  @cd {{ client_dir }} && git checkout

[group('Carla Client')]
[doc('Install the CARLA Python API')]
install-client: (_check_host client_host)
  # Clone carla
  @just _clone_carla_api
  # Pip install
  @pip install --user {{ python_deps }} carla=={{ carla_release }}

[group('Carla Client')]
[doc('Run automatic control with Zenoh')]
run-automatic host="127.0.0.1" port="2000": (_check_host client_host)
  # Copy script
  @cp -vf {{ bridge_examples }}/{{ example_auto }} {{ python_examples }}
  # Run client
  python3 {{ python_examples }}/{{ example_auto }} \
    --host {{ host }} \
    --port {{ port }} \
    --res {{ client_width }}x{{ client_height }} \
    --sync --loop

[group('Carla Client')]
[doc('Run manual control with Zenoh')]
run-manual router="127.0.0.1" host="127.0.0.1" port="2000": (_check_host client_host)
  # Copy scripts
  @cp -vf {{ bridge_examples }}/{{ zenoh_vehicle }} {{ python_examples }}
  @cp -vf {{ bridge_examples }}/{{ example_manual }} {{ python_examples }}
  # Run client
  python3 {{ python_examples }}/{{ example_manual }} \
    --host {{ host }} \
    --port {{ port }} \
    --res {{ client_width }}x{{ client_height }} \
    --filter "{{ vehicle_bp }}" \
    --rolename '{{ vehicle_role }}' \
    --router {{ router }}

[group('Carla Client')]
[doc('Run manual control with Sensors')]
run-manual-sensors router="127.0.0.1" host="127.0.0.1" port="2000": (_check_host client_host)
  # Copy scripts
  @cp -vf {{ bridge_examples }}/{{ zenoh_vehicle }} {{ python_examples }}
  @cp -vf {{ bridge_examples }}/{{ example_manual_sensors }} {{ python_examples }}
  # Run client
  python3 {{ python_examples }}/{{ example_manual_sensors }} \
    --host {{ host }} \
    --port {{ port }} \
    --res {{ client_width }}x{{ client_height }} \
    --filter "{{ vehicle_bp }}" \
    --rolename '{{ vehicle_role }}' \
    --router {{ router }}

[group('Carla Client')]
[doc('Uninstall CARLA Python API')]
[confirm("This action will REMOVE your CARLA folder and dependencies.\nAre you sure you want to continue? [y/N]")]
uninstall-client: (_check_host client_host)
  # Pip uninstall
  @pip uninstall carla {{ python_deps }} --yes
  # Remove carla
  @rm -rvf {{ client_cache }} {{ client_dir }}
