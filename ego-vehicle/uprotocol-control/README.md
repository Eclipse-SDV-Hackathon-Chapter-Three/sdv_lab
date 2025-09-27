# EgoVehicle uProtocol Controller

A Rust-based ego vehicle controller for CARLA simulation that uses both uProtocol and Zenoh for distributed communication. This hybrid application bridges CARLA's vehicle simulation with standardized automotive messaging (uProtocol) and traditional pub/sub systems, enabling remote vehicle control and status monitoring in automotive software-defined vehicle architectures.

## Features

- **CARLA Integration**: Connects to CARLA simulator and controls ego vehicle actors
- **Hybrid Messaging**: Uses both uProtocol (automotive standard) and traditional Zenoh pub/sub
- **uProtocol Compliance**: Implements standardized automotive communication patterns
- **Dual Control Modes**: Supports both manual control and autonomous cruise control
- **Real-time Status**: Publishes vehicle clock and velocity status via uProtocol
- **Graceful Shutdown**: Handles Ctrl-C interruption cleanly

## Communication Architecture

### uProtocol Topics (Automotive Standard)

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
- uProtocol-compatible systems for automotive integration

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

### Message Flow

1. **Incoming Commands**: Received via uProtocol listeners with automatic deserialization
2. **Status Publishing**: Vehicle state published as uProtocol messages with proper formatting
3. **Legacy Support**: Manual control inputs still supported via traditional Zenoh topics

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

### 2. uProtocol Communication Layer

- Implements automotive-standard messaging patterns
- Handles structured message serialization/deserialization
- Provides standardized automotive service discovery
- Manages uProtocol listeners for incoming commands

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
