
# Shared Notebook Installation

This can be ignored for participants, since the shared notebooks should be already setup when the events starts. It is for internal documentation and in the case a shared notebook must be setup newly because of issues.

## Install CARLA

Please follow the guideline for CARLA Setup: [Carla Quick Start](https://carla.readthedocs.io/en/latest/start_quickstart/)
Be sure to use the version [0.9.15](https://github.com/carla-simulator/carla/releases/tag/0.9.15/)

## Install AAOS Digital Cluster and Sync the Project

### Install Android Studio

Start by downloading and installing the latest version of Android Studio:

🔗 [Download Android Studio](https://developer.android.com/studio)

Make sure the following components are also set up:
- Android SDK properly configured
- Git installed
- Access to the project source code
- A physical device or emulator available for testing

---

### Sync the Project

Once the project is open in Android Studio:

- Navigate to **File > Sync Project with Gradle Files**
- Wait for Gradle to resolve dependencies and build the project

This step ensures your environment is correctly configured and ready for development.

---

### Create a Virtual Device (Emulator)

If you don’t have access to a physical device, you can create an emulator to test the app:

1. Go to **Tools > Device Manager**
2. Click **Create Device**
3. Select a device model (e.g., Pixel Tablet) and click **Next**
4. Choose a system image (e.g., Android 15) and download it if needed
5. Click **Finish** to create the virtual device
6. Launch the emulator by clicking the **Play** icon next to the device

---

### Run the Application

With your device ready:

- Select the emulator or connected physical device from the device dropdown
- Click the **Run** button (▶️) or press `Shift + F10`
- The app will build and launch on the selected device


## Install Eclipse Ankaios

Install podman first:

```shell
sudo apt-get update -y
sudo apt-get install -y podman
```

Install Ankaios v0.6.0:

```shell
curl -sfL https://github.com/eclipse-ankaios/ankaios/releases/download/v0.6.0/install.sh | bash -s -- -v v0.6.0
```

Systemd unit files are automatically installed for `ank-server` and `ank-agent`.

Set a persistent startup-config for `ank-server` by editing `/etc/systemd/system/ank-server.service`:

```shell
sudo vi /etc/systemd/system/ank-server.service
```

Add the CLI argument `--startup-config /etc/ankaios/state.yaml`.

Reload systemd settings:

```shell
sudo systemctl daemon-reload
```

Copy paste the `state.yaml` content into `/etc/ankaios/state.yaml`:

```yaml
apiVersion: v0.1
workloads:
  mqtt_broker:
    runtime: podman
    agent: agent_A
    restartPolicy: NEVER
    configs:
      mqtt_conf: mqtt_config
    files:
      - mountPoint: /mosquitto/config/mosquitto.conf
        data: "{{mqtt_conf}}"
    runtimeConfig: |
      image: docker.io/library/eclipse-mosquitto:latest
      commandOptions: ["--network", "host", "-u", "mosquitto", "-v", "mosquitto_data:/mosquitto/data"]
  ustreamer:
    runtime: podman
    agent: agent_A
    restartPolicy: NEVER
    dependencies:
      mqtt_broker: ADD_COND_RUNNING
    configs:
      ustreamer_config: ustreamer_config
    files:
      - mountPoint: /app/config/CONFIG.json5
        data: "{{ustreamer_config.config}}"
      - mountPoint: /app/config/MQTT_CONFIG.json5
        data: "{{ustreamer_config.mqtt_config}}"
      - mountPoint: /app/config/ZENOH_CONFIG.json5
        data: "{{ustreamer_config.zenoh_config}}"
      - mountPoint: /app/config/subscription_data.json
        data: "{{ustreamer_config.subscription_data}}"
    runtimeConfig: |
      image: ghcr.io/eclipse-uprotocol/up-streamer-rust/configurable-streamer:main
      commandOptions: ["--network", "host", "-e", "RUST_LOG=debug,zenoh=debug"]
configs:
  mqtt_config: |
    allow_anonymous true
    listener 1883 0.0.0.0
    persistence true
    persistence_location /mosquitto/data/
  ustreamer_config:
    config: |
      {
          up_streamer_config: {
            // The message queue size of each route between endpoints within the UStreamer
            // Lower numbers mean that some messages will be dropped
            message_queue_size: 10000
          },
          streamer_uuri: {
            // Determines the authority_name of the host device
            // Used when initializing host transport
            authority: "streamer",
            // Determines the ue_id of the streamer
            // Used when initializing host transport
            ue_id: 78,
            // Determines the ue_version_major of the streamer
            // Used when initializing host transport
            ue_version_major: 1
          },
          usubscription_config: {
            // Lists the path to the subscription file when using static file
            file_path: "config/subscription_data.json"
          },
          transports: {
              zenoh: {
                  // Path to the zenoh config file
                  config_file: "config/ZENOH_CONFIG.json5",
                  // List of endpoints that use the zenoh transport
                  endpoints: [
                      {
                          // Authority of the entity that the endpoint represents
                          authority: "carla",
                          // Identifier of the endpoint
                          endpoint: "carla_endpoint",
                          // List of identifiers of all other endpoints that messages should be forwarded to
                          forwarding: [
                              "hpc_endpoint"
                          ]
                      },
                      {
                          authority: "hpc",
                          // Make sure that each endpoint has a unique identifier or the streamer will not start
                          endpoint: "hpc_endpoint",
                          // All endpoint identifiers listed here must also be defined in this config
                          forwarding: [
                              "carla_endpoint",
                          ]
                      }
                  ]
              },
              mqtt: {
                  // Same as for the zenoh section but for all MQTT5 based endpoints
                  config_file: "config/MQTT_CONFIG.json5",
                  endpoints: [
                      {
                          authority: "threadx",
                          endpoint: "threadx_endpoint",
                          forwarding: [
                              "carla_endpoint",
                              "hpc_endpoint"
                          ]
                      },
                  ]
              },
          }
      }
    mqtt_config: |
      {
        hostname: "127.0.0.1",
        port: 1883,
        max_buffered_messages: 100,
        max_subscriptions: 100,
        session_expiry_interval: 3600,
        username: "user"
      }
    zenoh_config: |
      {
        mode: "router",
        connect: {
          endpoints: ["tcp/0.0.0.0:7447"],
        },
        scouting: {
          multicast: {
            enabled: false,
          },
            gossip: {
              enabled: true,
            },
          },
        routing: {
          router: {
            peers_failover_brokering: true,
          },
          peer: {
            mode: "peer_to_peer",
          },
        },
      }
    subscription_data: |
      {
        "//threadx/000A/2/8001": ["//carla/000A/2/1234", "//hpc/000A/2/1234"],
        "//carla/000A/2/8001": ["//hpc/000A/2/1234"],
        "//hpc/000A/2/8001": ["//carla/000A/2/1234"]
      }
```

Start Ankaios Server and the Ankaios Agent:

```shell
sudo systemctl start ank-server ank-agent
```

Check the workload states (it might be take some time until the container images are downloaded and the workloads are up and running):

```shell
ank get workloads --watch
```

Press Ctrl+C if all workloads have reached the running state.

If you need to delete the workloads and start from scratch, just stopping the Ankaios Server and Ankaios agent does not delete the workloads, they would still be running (automotive use case!).

Instead you have to:

```shell
ank delete workloads ustreamer mqtt_broker
```

or 

```shell
ank apply -d /etc/ankaios/state.yaml
```

and then:

```shell
sudo systemctl stop ank-server ank-agent
```

Next time you execute `sudo systemctl start ank-server ank-agent` all workloads are deployed newly.

## Checking workload logs

```shell
ank logs -f <workload_name>
```

Replace workload_name with `ustreamer`, `mqtt_broker` or any other workload name you want to check logs from.
