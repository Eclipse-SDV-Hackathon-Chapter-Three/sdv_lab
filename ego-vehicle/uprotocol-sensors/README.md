# EgoVehicle uProtocol Sensors (+ Controller)

A Rust-based ego vehicle controller for CARLA simulation that uses uProtocol-over-Zenoh for distributed communication. This hybrid application bridges CARLA's vehicle simulation with a service mesh (uProtocol) operating over a protocol (Zenoh), enabling remote vehicle control and status monitoring in automotive software-defined vehicle architectures.

uProtocol-over-Zenoh is also used to forward the sensor data from CARLA over to an application you will write which operates with this data to create an Advanced Driver Assistance System (ADAS) / Automated Driving (AD) feature.

## Features

- **CARLA Integration**: Connects to CARLA simulator and controls ego vehicle actors
- **Hybrid Messaging**: Uses both uProtocol (automotive standard) and traditional Zenoh pub/sub
- **uProtocol Compliance**: Implements standardized service mesh communication patterns
- **Dual Control Modes**: Supports both manual control and autonomous cruise control
- **Real-time Status**: Publishes vehicle clock and velocity status via uProtocol
- **Graceful Shutdown**: Handles Ctrl-C interruption cleanly

## Communication Architecture

### uProtocol Topics (for use in a Service Mesh architecture, abstracts out protocol details)

| Direction | Signal | Topic URI | Resource ID | Payload Signal | Description |
|-----------|--------|-----------|-------------|----------------|-------------|
| **Subscribe** | cc_throttle | `//CruiseControl/0/2/8001` | - | `0.7` | PID controller output for autonomous mode |
| **Subscribe** | cc_engage | `//AAOS/0/2/8002` | - | `1` | Cruise control engagement (0=manual, 1=autonomous) |
| **Publish** | curr_speed | `//EGOVehicle/0/2/8001` | 0x8001 | `45.2` | Vehicle velocity status in km/h |
| **Publish** | clock_status | `//EGOVehicle/0/2/8002` | 0x8002 | `123.456` | Simulation clock status in seconds |

### Traditional Zenoh Topics Subscription (Legacy Support to interactive with Python Carla Clients using Zenoh)

| Signal | Topic | Payload Example | Description |
|--------|-------|----------------|-------------|
| throttle_status | `vehicle/status/throttle_status` | `0.5` | Throttle input (0.0-1.0) for manual mode |
| steering_status | `vehicle/status/steering_status` | `-0.3` | Steering input (-1.0 to 1.0) for manual mode |
| brake_sensor | `vehicle/status/braking_status` | `0.2` | Brake input (0.0-1.0) for manual mode |

## Usage

### Prerequisites

- CARLA simulator running
- Rust toolchain installed
- Zenoh router (optional, for distributed setup)
- uProtocol-compatible systems for integration

### Command Line Arguments

```bash
cargo run --release -- [OPTIONS]
```

**Options:**

- `--host <HOST>`: CARLA server host (default: 127.0.0.1)
- `--port <PORT>`: CARLA server port (default: 2000)
- `--role <ROLE>`: Vehicle role name to control (default: ego_vehicle)
- `--delta <DELTA>`: Fixed delta seconds for simulation (default: 0.100)
- `--router <ROUTER>`: Zenoh router address for distributed mode (optional)

**Sensor Options**

- `--ego_vehicle_sensor_lane_invasion_role lane_invasion_1` (default: None, so no sensor)
- `--ego_vehicle_sensor_collision_role collision_1` (default: None, so no sensor)
- `--ego_vehicle_sensor_obstacle_detection_role obstacle_detection_1` (default: None, so no sensor)
- `--ego_vehicle_sensor_image_role front_camera` (default: None, so no sensor)
- `--ego_vehicle_sensor_radar_measurement_role front_radar` (default: None, so no sensor)
- `--ego_vehicle_sensor_lidar_measurement_role roof_lidar` (default: None, so no sensor)
- `--ego_vehicle_sensor_imu_measurement_role ego_imu` (default: None, so no sensor)

### Basic Usage

1. **Start CARLA simulator**
2. **Spawn an ego vehicle** with the specified role name in CARLA
3. **Run the controller**:

   ```bash
   # Local mode (peer-to-peer); no sensors
   cargo run --release

   # Local mode (peer-to-peer); add some sensors
   cargo run --release -- --ego_vehicle_sensor_lane_invasion_role lane_invasion_1 --ego_vehicle_sensor_image_role front_camera
   ```

### Detailed Example Usage

#### Terminal 1:

```shell
# navigate to carla-setup folder
# start the CARLA Server
just server-nvidia
```

#### Terminal 2:

```shell
# navigate to carla-setup folder
# start the CARLA Client configured with sensors
just manual-control-sensors
```

You should see a pygame window open showing the ego vehicle.

#### Terminal 3:

```shell
# start the the ego-vehicle proxy to collect sensors
cargo run --release -- --ego_vehicle_sensor_lane_invasion_role lane_invasion_1 --ego_vehicle_sensor_image_role front_camera
```

You should see the sensors configured be found and begin to publish in the terminal.

#### Terminal 4:

```shell
# navigate to ego-vehicle-sensor-mock folder
# start the ego-vehicle-sensor-subscriber to sanity check data is flowing
cargo run --bin ego-vehicle-sensor-mock -- ./sensor-uprotocol-configs.json5
```

You should any active sensors' data flow through in the terminal, usually in abbreviated form (e.g. not all camera pixels are logged).

### Adjusting CARLA Simulation and uprotocol-sensors for Different Sensor Configurations

While there are an array of sensors already configured, you may wish to, for example, a rear facing cameras or put another lidar sensor at a different angle.

Making these sorts of modifications is possible by investigating and modifying these files:
- `sdv_lab/carla-setup/examples/manual_control_sensors.py`
- `sdv_lab/ego-vehicle/uprotocol-sensor/src/main.rs`

If you'd like to test data flow from CARLA Server => `ego-vehicle/uprotocol-sensor` => `ego-vehicle-sensor-subscriber` for additional sensors or if you change the `role_name` from the default, you'll also need to modify:
- `ego-vehicle-sensor-mock/sensor-uprotocol-configs.json5` at a minimum

### Control Modes

#### Manual Mode (engage = 0)

- Vehicle responds to traditional Zenoh topics:
  - `vehicle/status/throttle_status`
  - `vehicle/status/steering_status`
  - `vehicle/status/braking_status`
- Direct control over vehicle actuators

#### Autonomous Mode (engage = 1)

- Vehicle responds to uProtocol actuation command: `//CruiseControl/0/2/8001`
- Positive values control throttle, negative values control braking
- Steering still controlled via Zenoh `steering_status` topic

## uProtocol Integration

### Entity Configuration

- **Authority**: `EGOVehicle`
- **Entity ID**: `0`
- **Entity Version**: `2`
- **Resource IDs**:
  - Velocity Status: `0x8001`
  - Clock Status: `0x8002`
  - LaneInvasionEvent: `0x8010`
  - CollisionEvent: `0x8011`
  - ObstacleDetectionEvent: `0x8012`
  - Image: `0x8013`
  - RadarMeasurement: `0x8014`
  - LidarMeasurement: `0x8015`
  - ImuMeasurement: `0x8016`

### Message Flow

1. **Incoming Commands**: Received via uProtocol listeners with automatic deserialization
2. **Status Publishing**: Vehicle state published as uProtocol messages with proper formatting
3. **Legacy Support**: Manual control inputs still supported via traditional Zenoh topics
4. **Outgoing Sensor Data**: Configured sensors are listened for over CARLA APIs and then forwarded via uProtocol-over-Zenoh

## Configuration

### Zenoh Configuration

- **Peer mode**: Direct peer-to-peer communication (default)
- **Router mode**: Connect to specified Zenoh router for distributed setup

### Vehicle Control Limits

- **Throttle**: 0.0 to 1.0
- **Steering**: -1.0 to 1.0 (left to right)
- **Braking**: 0.0 to 1.0

## Dependencies

- **up-rust**: uProtocol Rust SDK for automotive messaging
- **up-transport-zenoh**: uProtocol transport layer using Zenoh
- **carla**: CARLA Rust client library
- **zenoh**: Distributed pub/sub messaging
- **tokio**: Async runtime
- **async-trait**: Async trait support for uProtocol listeners
- **clap**: Command line argument parsing
- **log/pretty_env_logger**: Logging functionality

## Logging

If debug information is needed set the `RUST_LOG` environment variable to control log levels (info, debug, trace):

```bash
RUST_LOG=info cargo run --release  # Default recommended level
```

## Architecture

The application follows a hybrid event-driven architecture with four main components:

### 1. CARLA Interface Layer

- Connects to CARLA simulator via TCP
- Manages world synchronization and actor discovery
- Applies vehicle control commands
- Retrieves vehicle state information
- Retrieves configured sensor information

### 2. uProtocol Communication Layer

- Implements automotive-standard messaging patterns
- Handles structured message serialization/deserialization
- Manages uProtocol listeners for incoming commands
- Forwards retrieved sensor data via uProtocol-over-Zenoh

### 3. Traditional Zenoh Communication Layer

- Maintains backward compatibility with existing systems
- Handles legacy pub/sub messaging
- Supports manual control inputs
- Provides distributed communication capabilities

### 4. Control Logic Layer

- Processes incoming control commands from both protocols
- Implements dual control mode switching with uProtocol priority
- Enforces safety limits on actuator values
- Manages real-time control loop execution

## Error Handling

The application includes robust error handling for:

- **Connection failures**: Automatic retry for CARLA connection
- **Actor discovery**: Continuous polling until ego vehicle is found
- **Message parsing**: Graceful handling of malformed payloads (both protocols)
- **Control limits**: Automatic clamping of out-of-range values
- **uProtocol errors**: Proper error propagation and logging

## Performance Considerations

- **Polling intervals**: 1000ms for ego vehicle discovery, 1ms for publishing
- **Synchronization**: Uses CARLA's `wait_for_tick()` for frame synchronization
- **Delta time management**: Maintains consistent simulation timing
- **Async processing**: Non-blocking message handling with Tokio
- **Protocol priority**: uProtocol commands take precedence over Zenoh in autonomous mode

## Troubleshooting

### Common Issues

1. **"Waiting for the Ego Vehicle actor..."**
   - Ensure a vehicle with the specified role name exists in CARLA
   - Check that the role name matches exactly (case-sensitive)

2. **uProtocol connection issues**
   - Verify uProtocol transport initialization
   - Check entity and resource ID configurations
   - Ensure proper URI formatting

3. **Mixed protocol conflicts**
   - Verify control mode switching logic
   - Check message priority handling
   - Confirm proper listener registration

4. **"Waiting for actor with role_name=<ROLE_NAME_HERE>..."**
   - Have you given a different role_name for the sensor in `sdv_lab/carla-setup/examples/manual_control_sensors.py`?
   - Have you made sure the sensor is "turned on"? Certain sensors are not turned on by default:
      - To turn on the radar sensor, click on the pygame window and press `g`
      - The configuration at present allows either camera or lidar; you can toggle which vision sensor is being used via `tab` until you arrive at the desired one. Or -- you could modify `manual_control_sensors.py` to have both on at the same time.
