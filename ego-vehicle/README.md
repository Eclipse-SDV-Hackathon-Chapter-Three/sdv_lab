# EgoVehicle Controllers

A collection of Rust-based ego vehicle controllers for CARLA simulation with different messaging protocols. This project provides three implementations for distributed vehicle control and monitoring in automotive software-defined vehicle (SDV) architectures.

## Contents

### [Container](./container/)

- **Clean build environment**: Container environment that can be used to build software

### [uProtocol Controller](./uprotocol-control/)

- **Service Mesh Showcase**: Eclipse uProtocol (service mesh communication abstraction) + Eclipse Zenoh (for underlying protocol)
- **Advanced integration**: Designed for modern software-defined vehicle architectures

### [Zenoh Controller](./zenoh-control/)

- **Traditional pub/sub**: Pure Zenoh messaging for distributed communication
- **Compatibility**: Works with existing Zenoh-based systems

### [uProtocol Sensors](./uprotocol-sensors/)

- **Service Mesh Showcase**: Eclipse uProtocol (service mesh communication abstraction) + Eclipse Zenoh (for underlying protocol)
- **Advanced integration**: Designed for modern software-defined vehicle architectures

## Common Features: uProtocol Controller & Zenoh Controller

- **CARLA Integration**: Direct connection to CARLA simulator
- **Dual Control Modes**: Manual control and autonomous cruise control
- **Real-time Status**: Vehicle velocity and simulation clock monitoring
- **Distributed Architecture**: Support for multi-node deployments

## Quick Start

1. **Start CARLA simulator**
2. **Choose your implementation**:

   ```bash
   # For service mesh messaging on top of Zenoh
   cd uprotocol && cargo run --release
   
   # For Zenoh directly messaging  
   cd zenoh && cargo run --release
   ```

## Use Cases

- **Automotive R&D**: Testing vehicle control algorithms with industry-standard protocols
- **Simulation Integration**: Bridging CARLA with distributed control systems
- **Protocol Evaluation**: Comparing uProtocol vs traditional messaging approaches
- **Educational**: Learning automotive software architectures and communication patterns


For implementation-specific details, see the README.md files in each subdirectory:

- [Zenoh version Documentation](zenoh/README.md)
- [uProtocol version Documentation](uprotocol/README.md)
