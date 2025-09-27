# PID Controller with uProtocol over Zenoh for Velocity Control

A real-time PID (Proportional-Integral-Derivative) controller system implemented in Rust, designed for velocity control applications using uProtocol over Zenoh middleware for distributed communication. The system implements cruise control functionality by computing acceleration commands to maintain desired vehicle speeds.

## Features

- **Real-time PID Control**: Classical PID algorithm with configurable gains (Kp, Ki, Kd)
- **uProtocol Integration**: Standards-compliant communication using uProtocol over Zenoh transport
- **Enable/Disable Control**: Runtime activation/deactivation of PID control
- **Data Logging**: Automatic storage of control data for analysis (JSON and text formats)
- **Robust Error Handling**: Graceful handling of communication and computation errors
- **Async/Await Support**: Modern Rust async programming with Tokio runtime

## System Architecture

The system consists of three main components:

1. **PIDController** (`pid_controller.rs`): Core PID algorithm implementation
2. **UProtocolHandler** (`uprotocol_handler.rs`): uProtocol communication layer managing subscriptions and publications
3. **Main Application** (`main.rs`): System orchestration and configuration

## uProtocol Topics

### Subscribed Topics (Inputs)

| Signal | Authority | UE ID | Version | Resource ID | URI | Payload Format | Example | Description |
|--------|-----------|-------|---------|-------------|-----|----------------|---------|-------------|
| clock_status | EGOVehicle | 0 | 2 | 0x8002 | `EGOVehicle/0/2/8002` | Text/JSON | `1234567890.123` or `{"time": 1234567890.123}` | System timestamp in seconds |
| curr_speed | EGOVehicle | 0 | 2 | 0x8001 | `EGOVehicle/0/2/8001` | Text/JSON | `65.5` or `{"velocity": 65.5}` | Current vehicle velocity (km/h) |
| cc_speed | AAOS | 0 | 2 | 0x8001 | `AAOS/0/2/8001` | Text/JSON | `70.0` or `{"speed": 70.0}` | Desired target velocity (km/h) |
| cc_engage | AAOS | 0 | 2 | 0x8002 | `AAOS/0/2/8002` | Text/JSON | `1` or `{"engaged": 1}` | Enable/disable PID control (0=off, 1=on) |

### Published Topics (Outputs)

| Signal | Authority | UE ID | Version | Resource ID | URI | Payload Format | Example | Description |
|--------|-----------|-------|---------|-------------|-----|----------------|---------|-------------|
| cc_throttle | CruiseControl | 0 | 2 | 0x8001 | `CruiseControl/0/2/8001` | Text | `0.5` | Computed acceleration command (m/s²) |

## Installation

### Prerequisites

- Rust 1.70+ with Cargo
- uProtocol Rust library
- Zenoh transport dependencies

### Setup

1. Build the project:

   ```bash
   cargo build --release
   ```

2. Install uProtocol publisher tool (optional, only used for functional testing):

   ```bash
   cargo install --path . --bin up_pub
   ```

## Usage

### Running the PID Controller

1. Start the PID controller system:

   ```bash
   RUST_LOG=info cargo run --bin pid_controller
   ```

   Or using the release build:

   ```bash
   RUST_LOG=info ./target/release/pid_controller
   ```

2. The system will start with PID **disabled** by default and register uProtocol listeners for incoming messages.

### Testing with Vehicle Simulator

Use the included `simulator` tool to send test messages and simulate vehicle behavior, the app will run in loop send uProtocol messages.
The tool is responsible for:

- Generate random values for target and current speed
- Generate activation signal to enable cruise control
- Generate clock signal
- Send the signals generated over the uProtocol

```bash
cargo run --bin simulator
```

Cruise Control activation/deactivation is hard-coded and enabled by default in order to have toggled operations uncomment line 130 of testing/simulator.rs file:

```rust
// engaged = if engaged == 1 { 0 } else { 1 };
```

### Testing and Debugging with uProtocol Publisher

Use the included `up_pub` tool to send test messages:

**Enable PID control:**

```bash
cargo run --bin up_pub -- args --uri "AAOS/0/2/8002" --payload "1" --format text
```

**Disable PID control:**

```bash
cargo run --bin up_pub -- args --uri "AAOS/0/2/8002" --payload "0" --format text
```

**Set target speed:**

```bash
cargo run --bin up_pub -- args --uri "AAOS/0/2/8001" --payload "70.0" --format text
```

**Publish current velocity:**

```bash
cargo run --bin up_pub -- args --uri "EGOVehicle/0/2/8001" --payload "65.5" --format text
```

**Publish timestamp:**

```bash
cargo run --bin up_pub -- args --uri "EGOVehicle/0/2/8002" --payload "$(date +%s.%3N)" --format text
```

### Interactive Testing

Use interactive mode for continuous testing:

```bash
cargo run --bin up_pub -- interactive
```

### Batch Testing from File

Create a JSON file with multiple messages and publish them:

```bash
cargo run --bin up_pub -- file --input test_messages.json
```

Example `test_messages.json`:

```json
[
  {
    "uri": "AAOS/0/2/8002",
    "payload": "1",
    "format": "text"
  },
  {
    "uri": "AAOS/0/2/8001", 
    "payload": "70.0",
    "format": "text"
  },
  {
    "uri": "EGOVehicle/0/2/8001",
    "payload": "65.5", 
    "format": "text"
  }
]
```

## Configuration

### PID Tuning Parameters

Default values in `main.rs`:

```rust
let kp = 0.125;    // Proportional gain
let ki = kp / 8.0; // Integral gain (0.015625)
let kd = kp / 10.0; // Derivative gain (0.0125)
```

Adjust these values based on your system's response characteristics:

- **Kp**: Increases response speed but may cause overshoot
- **Ki**: Eliminates steady-state error but may cause oscillation  
- **Kd**: Reduces overshoot and improves stability

### uProtocol Entity Configuration

The PID controller registers as a uProtocol entity:

```rust
let entity_uri = UUri::try_from_parts("CruiseControl", 0, 2, 0)?;
```

Modify the authority name, UE ID, or version as needed for your deployment.

## Output Files

When the system terminates (CTRL-C), it generates:

- `logs/desired_velocity.log`: Target velocity values over time
- `logs/current_velocity.log`: Actual velocity measurements  
- `logs/current_time.log`: Timestamp data
- `logs/acceleration.log`: PID controller output values
- `logs/pid_results.json`: Complete results in JSON format

## System Behavior

1. **Startup**: PID controller starts in **disabled** state
2. **Registration**: Registers uProtocol listeners for all input topics
3. **Enable**: Send `1` to engage topic to activate control
4. **Control Loop**: When enabled, computes acceleration based on velocity error
5. **Disable**: Send `0` to engage topic to deactivate (resets internal state)
6. **Shutdown**: CTRL-C saves data logs and shows results summary

## Message Formats

The system supports both text and JSON payload formats for backward compatibility:

**Text Format (Recommended):**
```
65.5
```

**JSON Format (Legacy):**
```json
{"velocity": 65.5}
{"time": 1234567890.123}
{"speed": 70.0}
{"engaged": 1}
```

## Logging

Enable detailed logging with environment variables:
```bash
RUST_LOG=debug cargo run --bin pid_controller
RUST_LOG=info cargo run --bin up_pub -- args --uri "AAOS/0/2/8001" --payload "70.0"
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

## Troubleshooting

### Common Issues

- **No acceleration output**: Ensure PID is enabled via engage topic (`AAOS/0/2/8002`)
- **Erratic behavior**: Check timestamp topic is publishing at sufficient rate
- **uProtocol connection failed**: Verify Zenoh router is running and accessible
- **Message parsing errors**: Check payload format matches expected text or JSON structure

### Debug Output

The system provides verbose logging:
```
INFO - PID => Kp=0.125, Ki=0.015625, Kd=0.0125
INFO - Timestamp subscriber registered
INFO - Received current velocity '65.50'
INFO - Publishing Acceleration: 0.5625
INFO - [INFO] PID controller ACTIVATED at 1640995200
```

### Testing uProtocol Connectivity

Verify uProtocol transport is working:
```bash
# In one terminal - start the PID controller
RUST_LOG=info cargo run --bin pid_controller

# In another terminal - send test message
cargo run --bin up_pub -- args --uri "EGOVehicle/0/2/8001" --payload "60.0" --format text
```

You should see message reception logs in the PID controller terminal.

## Development

### Building

```bash
# Debug build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run clippy lints
cargo clippy
```

### Dependencies

Key dependencies in `Cargo.toml`:

- `up-rust`: uProtocol core library
- `up-transport-zenoh`: Zenoh transport implementation
- `tokio`: Async runtime
- `serde`: Serialization framework
- `log`: Logging facade
