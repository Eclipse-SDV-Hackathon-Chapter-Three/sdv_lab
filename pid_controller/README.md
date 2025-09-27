# PID Controller Implementations for Velocity Control

A collection of real-time PID (Proportional-Integral-Derivative) controller implementations for velocity control applications, featuring two different communication middleware approaches: Zenoh and uProtocol over Zenoh.

## Project Overview

This repository contains two complete PID controller implementations designed for cruise control functionality:

- **Python + Zenoh**: Direct Zenoh pub/sub communication
- **Rust + uProtocol**: Standards-compliant uProtocol over Zenoh transport

Both implementations compute acceleration commands to maintain desired vehicle speeds using classical PID control algorithms.

For implementation-specific details, see the README.md files in each subdirectory:

- [Python + Zenoh Documentation](python-zenoh/README.md)
- [Rust + uProtocol Documentation](rust-uprotocol/README.md)

## Project Structure

```
.
├── README.md                           # Main project documentation and overview
├── python-zenoh/                       # Python implementation using Zenoh pub/sub
│   ├── README.md                       # Python-specific documentation
│   ├── __init__.py                     # Python package initialization file
│   ├── controller.py                   # Core PID controller algorithm implementation
│   ├── main.py                         # Python application entry point and configuration
│   ├── requirements.txt                # Python dependencies (zenoh, numpy, matplotlib)
│   └── zenoh_handler.py               # Zenoh communication layer and pub/sub management
└── rust-uprotocol/                    # Rust implementation using uProtocol over Zenoh
    ├── README.md                       # Rust-specific documentation
    ├── Cargo.lock                      # Rust dependency lock file (auto-generated)
    ├── Cargo.toml                      # Rust project configuration and dependencies
    └── src/                            # Rust source code directory
        ├── main.rs                     # Rust application entry point (PID controller)
        ├── pid_controller.rs           # Core PID algorithm implementation in Rust
        ├── testing/                    # Testing utilities and tools
        │   ├── simulator.rs            # Vehicle data simulator for testing
        │   └── uprotocol_pub.rs        # uProtocol message publisher tool
        └── uprotocol_handler.rs        # uProtocol communication layer and message handling
```

## Features Comparison

| Feature | Python + Zenoh | Rust + uProtocol |
|---------|----------------|------------------|
| **Language** | Python 3.10+ | Rust 1.70+ |
| **Communication** | Direct Zenoh pub/sub | uProtocol over Zenoh |
| **Standards Compliance** | Zenoh native | uProtocol compliant |
| **Memory Safety** | Runtime checks | Compile-time guarantees |
| **Async Support** | Threading | Tokio async/await |
| **Data Logging** | Text files + PNG plots | JSON + text files |
| **Testing Tools** | Manual publishers | Built-in `up_pub` tool |

## Communication Protocols

### Python + Zenoh Topics

### Subscribed Topics (Inputs)

| Signal | Topic | Payload | Description |
|--------|-------|---------|-------------|
| clock_status | `vehicle/status/clock_status` | `1234567890.123` | System timestamp in seconds |
| curr_speed | `vehicle/status/velocity_status` | `65.5` | Current velocity (km/h) |
| cc_speed | `adas/cruise_control/target_speed` | `70.0` | Target velocity (km/h) |
| cc_engage | `adas/cruise_control/engage` | `true`/`false` | Enable/disable control |

### Published Topics (Outputs)

| Signal | Topic | Payload | Description |
|--------|-------|---------|-------------|
| cc_throttle | `control/command/actuation_cmd` | `0.5` | Acceleration output |

### Rust + uProtocol URIs

### Subscribed Topics (Inputs)

| Signal | URI | Payload | Description |
|--------|-----|---------|-------------|
| clock_status | `EGOVehicle/0/2/8002` | `1234567890.123` | System timestamp in seconds |
| curr_speed | `EGOVehicle/0/2/8001` | `65.5` | Current velocity (km/h) |
| cc_speed | `AAOS/0/2/8001` | `70.0` | Target velocity (km/h) |
| cc_engage | `AAOS/0/2/8002` | `1`/`0` | Enable/disable control |


### Published Topics (Outputs)

| Signal | URI | Payload | Description |
|--------|-----|---------|-------------|
| cc_throttle | `CruiseControl/0/2/8001` | `0.5` | Acceleration output |

## Quick Start

### Python Zenoh Implementation

```bash
cd python-zenoh
pip install -r requirements.txt
python3 main.py
```

### Rust uProtocol Implementation

```bash
cd rust-uprotocol
cargo build --release
RUST_LOG=info cargo run --bin pid_controller
```

## Testing

### Python Zenoh

#### Starting Application

**Terminal 1 - Run the app:**

```bash
cd python-zenoh
python3 main.py
```

#### Send Test Data

**Terminal 2 - Test using z_put:**

```bash
z_put -k "adas/cruise_control/engage" -v "true"
z_put -k "adas/cruise_control/target_speed" -v "70.0"
z_put -k "vehicle/status/velocity_status" -v "65.5"
```

### Rust uProtocol

#### Starting Application

**Terminal 1 - Run the app:**

```bash
cd rust-uprotocol
RUST_LOG=info cargo run --bin pid_controller
```

#### Send Test Data

**Terminal 2 - Test using up_pub:**

```bash
cd rust-uprotocol
cargo run --bin up_pub -- args --uri "AAOS/0/2/8002" --payload "1" --format text
cargo run --bin up_pub -- args --uri "AAOS/0/2/8001" --payload "70.0" --format text
cargo run --bin up_pub -- args --uri "EGOVehicle/0/2/8001" --payload "65.5" --format text
```

## Prerequisites

### Common Requirements

- Zenoh router (optional, uses peer-to-peer by default)
- Network connectivity between components

### Python Implementation

- Python 3.10+
- eclipse-zenoh package
- NumPy and Matplotlib for visualization

### Rust Implementation  

- Rust 1.70+ with Cargo
- uProtocol Rust libraries
- Tokio async runtime

## Configuration

Both implementations use the same PID tuning parameters by default:

```text
Kp = 0.125     # Proportional gain
Ki = 0.015625  # Integral gain (Kp/8)
Kd = 0.0125    # Derivative gain (Kp/10)
```

If needed, modify these values in:

- Python: `python-zenoh/main.py`
- Rust: `rust-uprotocol/src/main.rs`

## Output and Logging

### Python Implementation

- Generates `*.log` files and `results.png` plot
- Real-time console output with timestamps

### Rust Implementation

- Generates structured logs in `logs/` directory
- JSON format results in `logs/pid_results.json`
- Comprehensive statistics summary


## Troubleshooting

### Common Issues

**No communication between implementations:**

- Ensure both use compatible Zenoh configurations
- Check network connectivity and firewall settings
- Verify topic/URI naming conventions

**Performance issues:**

- Python: Check for blocking operations in main thread
- Rust: Monitor async task scheduling and resource usage

**Data format mismatches:**

- Both implementations support text payloads
- Rust also supports JSON for backward compatibility

### Debug Logging

**Python:**

```bash
# Enable verbose output in code or use print statements
python3 main.py
```

**Rust:**

```bash
# Set log level via environment variable
RUST_LOG=debug cargo run --bin pid_controller
```

## Related Projects

- [Eclipse Zenoh](https://zenoh.io/) - Zero Overhead Network Protocol
- [uProtocol](https://github.com/eclipse-uprotocol) - Universal Protocol for IoT/Automotive
- [up-rust](https://github.com/eclipse-uprotocol/up-rust) - uProtocol Rust Implementation

