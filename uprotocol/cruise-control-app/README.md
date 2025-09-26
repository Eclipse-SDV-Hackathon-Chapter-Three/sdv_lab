# Cruise-Control Example App

This folder contains an example application that implements a simulated Cruise-Control component.

The example uses Eclipse uProtocol to periodically publish the current operational status (e.g. current speed, engine temperature) and to expose an API endpoint for setting the target speed.

## Getting Started

The example is implemented in Rust and therefore requires a [Rust toolchain to be installed](https://rustup.rs/) for building.

```bash
cargo build
```

The application supports using either Eclipse Zenoh or MQTT 5 for exchanging messages. The transport can be selected on the command line.

```bash
cargo run
```

will display all available command line options.

In order to enable informational log statements being printed to the console, the `RUST_LOG` environment variable can be used:

```bash
RUST_LOG=INFO cargo run
```

To enable debug logging for the app, use:

```bash
RUST_LOG=INFO,cruise_control_app=DEBUG cargo run
```

This will enable log statements regarding the sending of status messages.
