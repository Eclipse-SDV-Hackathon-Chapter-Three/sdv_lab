# PID Controller with Zenoh Pub/Sub for Velocity Control

A real-time PID (Proportional-Integral-Derivative) controller system designed for velocity control applications, using Zenoh middleware for distributed communication. The system implements cruise control functionality by computing acceleration commands to maintain desired vehicle speeds.

## Features

- **Real-time PID Control**: Classical PID algorithm with configurable gains (Kp, Ki, Kd)
- **Zenoh Integration**: Distributed pub/sub communication for real-time data exchange
- **Enable/Disable Control**: Runtime activation/deactivation of PID control
- **Data Logging**: Automatic storage of control data for analysis
- **Visualization**: Built-in plotting of velocity tracking and acceleration output
- **Robust Error Handling**: Graceful handling of communication and computation errors

## System Architecture

The system consists of three main components:

1. **PIDController** (`controller.py`): Core PID algorithm implementation
2. **ZenohHandler** (`zenoh_handler.py`): Communication layer managing pub/sub topics
3. **Main Application** (`main.py`): System orchestration and configuration

## Zenoh Topics

### Subscribed Topics (Inputs)

| Signal | Topic | Payload Type | Example | Description |
|--------|-------|--------------|---------|-------------|
| clock_status | `vehicle/status/clock_status` | float | `1234567890.123` | System timestamp in seconds |
| curr_speed | `vehicle/status/velocity_status` | float | `65.5` | Current vehicle velocity (km/h) |
| cc_speed | `adas/cruise_control/target_speed` | float | `70.0` | Desired target velocity (km/h) |
| cc_engage | `adas/cruise_control/engage` | boolean/string | `true`, `1`, `on` | Enable/disable PID control |

### Published Topics (Outputs)

| Signal | Topic | Payload Type | Example | Description |
|--------|-------|--------------|---------|-------------|
| cc_throttle | `control/command/actuation_cmd` | float | `0.5` [`0.0`, `1.0`] | Computed acceleration command (m/s²) |

## Installation

### Prerequisites

- Python 3.10+
- Zenoh Python library
- NumPy and Matplotlib for data visualization

### Setup

1. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

2. Install Zenoh command-line tools (optional, only used for functional testing)
   ```bash
   # Install Rust and Cargo (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

   # Install Zenoh CLI tools
   cargo install zenoh --features=unstable
   ```

## Usage

### Running the PID Controller

1. Start the PID controller system:
   ```bash
   python3 main.py
   ```

2. The system will start with PID **disabled** by default and is able to receive messages from zenoh network.
   Sending messages to the PID Controller subscribed topics could be used to test it:
   ```bash
   # Using zenoh command line tools (see 'Setup' section to get instructions on how to install it, if needed)
   z_put -k "adas/cruise_control/engage" -v "true"
   z_put -k "adas/cruise_control/target_speed" -v "70"
   z_put -k "vehicle/status/clock_status" -v "12345678.123"
   z_put -k "vehicle/status/velocity_status" -v "65.5"
   ```

### Control Commands

Enable PID control:
```bash
z_put -k "adas/cruise_control/engage" -v "true"
```

Disable PID control:
```bash
z_put -k "adas/cruise_control/engage" -v "false"
```

Set target speed:
```bash
z_put -k "adas/cruise_control/target_speed" -v "65.0"
```

## Configuration

### PID Tuning Parameters

Default values in `main.py`:
```python
Kp = 0.125    # Proportional gain
Ki = Kp / 8   # Integral gain (0.015625)
Kd = Kp / 10  # Derivative gain (0.0125)
```

Adjust these values based on your system's response characteristics:
- **Kp**: Increases response speed but may cause overshoot
- **Ki**: Eliminates steady-state error but may cause oscillation
- **Kd**: Reduces overshoot and improves stability

## Output Files

When the system terminates, it generates:

- `desired_velocity.log`: Target velocity values over time
- `current_velocity.log`: Actual velocity measurements
- `current_time.log`: Timestamp data
- `acceleration.log`: PID controller output values
- `results.png`: Visualization plot showing velocity tracking performance

## System Behavior

1. **Startup**: PID controller starts in **disabled** state
2. **Enable**: Send `true` to engage topic to activate control
3. **Control Loop**: When enabled, computes acceleration based on velocity error
4. **Disable**: Send `false` to engage topic to deactivate (resets internal state)
5. **Shutdown**: CTRL-C saves data logs and generates visualization

## Troubleshooting

### Common Issues

- **No acceleration output**: Ensure PID is enabled via engage topic
- **Erratic behavior**: Check timestamp topic is publishing at sufficient rate
- **ValueError on delta_time**: Verify clock_status topic provides monotonic timestamps

### Debug Output

The system provides verbose logging:
```
PID => Kp=0.125, Ki=0.015625, Kd=0.0125
Current timestamp subscriber started, waiting for data...
Received current velocity '65.5'
Publishing Acceleration: 0.5625
[INFO] PID controller ACTIVATED at 2025-09-25T10:30:45.123456
```
