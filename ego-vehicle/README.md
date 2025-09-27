# EgoVehicle Controllers

A collection of Rust-based ego vehicle controllers for CARLA simulation with different messaging protocols. This project provides two implementations for distributed vehicle control and monitoring in automotive software-defined vehicle (SDV) architectures.

## Implementations

### [uProtocol Controller](./uprotocol/)

- **Hybrid messaging**: uProtocol (automotive standard) + Zenoh for legacy support
- **Automotive compliance**: Implements standardized automotive communication patterns
- **Advanced integration**: Designed for modern software-defined vehicle architectures

### [Zenoh Controller](./zenoh/)

- **Traditional pub/sub**: Pure Zenoh messaging for distributed communication
- **Lightweight**: Simpler implementation focused on core functionality
- **Broad compatibility**: Works with existing Zenoh-based systems

## Common Features

- **CARLA Integration**: Direct connection to CARLA simulator
- **Dual Control Modes**: Manual control and autonomous cruise control
- **Real-time Status**: Vehicle velocity and simulation clock monitoring
- **Distributed Architecture**: Support for multi-node deployments

## Quick Start

1. **Start CARLA simulator**
2. **Choose your implementation**:

   ```bash
   # For automotive-standard messaging
   cd uprotocol && cargo run --release
   
   # For traditional pub/sub messaging  
   cd zenoh && cargo run --release
   ```

## Use Cases

- **Automotive R&D**: Testing vehicle control algorithms with industry-standard protocols
- **Simulation Integration**: Bridging CARLA with distributed control systems
- **Protocol Evaluation**: Comparing uProtocol vs traditional messaging approaches
- **Educational**: Learning automotive software architectures and communication patterns


For implementation-specific details, see the README.md files in each subdirectory:

- [Zenoh version Documentation](uprotocol/README.md)
- [uProtocol version Documentation](zenoh/README.md)
