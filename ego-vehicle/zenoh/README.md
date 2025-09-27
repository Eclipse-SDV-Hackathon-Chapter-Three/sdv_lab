# EgoVehicle Zenoh Controller

A Rust-based ego vehicle controller for CARLA simulation that uses Zenoh for distributed communication. This application bridges CARLA's vehicle simulation with a pub/sub messaging system, enabling remote vehicle control and status monitoring.

## Features

- **CARLA Integration**: Connects to CARLA simulator and controls ego vehicle actors
- **Zenoh Messaging**: Uses Zenoh pub/sub for distributed communication
- **Dual Control Modes**: Supports both manual control and autonomous cruise control
- **Real-time Status**: Publishes vehicle clock and velocity status
- **Graceful Shutdown**: Handles Ctrl-C interruption cleanly

## Zenoh Topics

### Subscribed Topics (Input)

| Signal | Topic | Payload Example | Description |
|-------|-------|----------------|-------------|
| throttle_status | `vehicle/status/throttle_status` | `0.5` | Throttle input (0.0-1.0) for manual mode |
| steering_status | `vehicle/status/steering_status` | `-0.3` | Steering input (-1.0 to 1.0) for manual mode |
| brake_sensor | `vehicle/status/braking_status` | `0.2` | Brake input (0.0-1.0) for manual mode |
| cc_throttle | `control/command/actuation_cmd` | `0.7` | PID controller output for autonomous mode |
| cc_engage | `adas/cruise_control/engage` | `1` | Cruise control engagement (0=manual, 1=autonomous) |

### Published Topics (Output)

| Signal | Topic | Payload Example | Description |
|-------|-------|----------------|-------------|
| clock_status | `vehicle/status/clock_status` | `123.456` | Simulation elapsed time in seconds |
| curr_speed | `vehicle/status/velocity_status` | `45.2` | Vehicle velocity in km/h |

## Usage

### Prerequisites

- CARLA simulator running
- Rust toolchain installed
- Zenoh router (optional, for distributed setup)

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

### Basic Usage

1. **Start CARLA simulator**
2. **Spawn an ego vehicle** with the specified role name in CARLA
3. **Run the controller**:

   ```bash
   # Local mode (peer-to-peer)
   cargo run --release
   
   # With custom CARLA settings
   cargo run --release -- --host 192.168.1.100 --port 2000 --role my_vehicle
   
   # With Zenoh router
   cargo run --release -- --router 192.168.1.200
   ```

### Control Modes

#### Manual Mode (engage = 0)

- Vehicle responds to `throttle_status`, `steering_status`, and `braking_status` topics
- Direct control over vehicle actuators

#### Autonomous Mode (engage = 1)

- Vehicle responds to `actuation_cmd` topic (PID controller output)
- Positive values control throttle, negative values control braking
- Steering still controlled via `steering_status`

## Configuration

### Zenoh Configuration

- **Peer mode**: Direct peer-to-peer communication (default)
- **Router mode**: Connect to specified Zenoh router for distributed setup

### Vehicle Control Limits

- **Throttle**: 0.0 to 1.0
- **Steering**: -1.0 to 1.0 (left to right)
- **Braking**: 0.0 to 1.0

## Dependencies

- **carla**: CARLA Rust client library
- **zenoh**: Distributed pub/sub messaging
- **tokio**: Async runtime
- **clap**: Command line argument parsing
- **log/pretty_env_logger**: Logging functionality

## Logging

```bash
cargo run   # Default recommended level
```

If debug information is needed set the `RUST_LOG` environment variable to control log levels (info, debug, trace):

```bash
RUST_LOG=info cargo run  # Default recommended level
```

## Architecture

The application follows an event-driven architecture with three main components:

### 1. CARLA Interface Layer

- Connects to CARLA simulator via TCP
- Manages world synchronization and actor discovery
- Applies vehicle control commands
- Retrieves vehicle state information

### 2. Zenoh Communication Layer

- Handles pub/sub messaging with configurable topology
- Manages topic subscriptions and publications
- Provides distributed communication capabilities
- Supports both peer-to-peer and router-based modes

### 3. Control Logic Layer

- Processes incoming control commands
- Implements dual control mode switching
- Enforces safety limits on actuator values
- Manages real-time control loop execution

## Error Handling

The application includes robust error handling for:

- **Connection failures**: Automatic retry for CARLA connection
- **Actor discovery**: Continuous polling until ego vehicle is found
- **Message parsing**: Graceful handling of malformed payloads
- **Control limits**: Automatic clamping of out-of-range values

## Performance Considerations

- **Polling intervals**: 1000ms for ego vehicle discovery, 1ms for publishing
- **Synchronization**: Uses CARLA's `wait_for_tick()` for frame synchronization
- **Delta time management**: Maintains consistent simulation timing
- **Async processing**: Non-blocking message handling with Tokio

## Troubleshooting

### Common Issues

1. **"Waiting for the Ego Vehicle actor..."**
   - Ensure a vehicle with the specified role name exists in CARLA
   - Check that the role name matches exactly (case-sensitive)

2. **Connection timeout to CARLA**
   - Verify CARLA is running and accessible
   - Check host/port configuration
   - Ensure firewall allows connections

3. **No Zenoh subscribers/publishers**
   - Verify Zenoh router is running (if using router mode)
   - Check network connectivity between nodes
   - Confirm topic names match exactly

### Debug Mode

Enable detailed logging for troubleshooting (info, debug, trace):

```bash
RUST_LOG=debug cargo run --release -- --host localhost --port 2000
```

### With Python Client

```python
import zenoh
import time

# Open Zenoh session
session = zenoh.open()

# Publish steering command
session.put("vehicle/status/steering_status", "0.2")

# Subscribe to velocity status
def listener(sample):
    print(f"Velocity: {sample.payload.decode()} km/h")

subscriber = session.declare_subscriber("vehicle/status/velocity_status", listener)
time.sleep(10)  # Listen for 10 seconds
```
