# SDV LAB

The SDV Lab – Rapid SDV Feature Development challenge invites everyone to step into the future of automotive innovation by building, integrating, and validating a feature for a Software-Defined Vehicle (SDV) within a realistic, modular, and cyber-physical simulation environment. This challenge is designed to simulate the complexities of modern vehicle software development, where embedded systems, middleware, orchestration, and simulation must work in harmony to deliver safe, intelligent, and responsive vehicle behavior.  
  
## The Challenge

**Design, implement, and showcase** a working SDV feature in the Automated Driving / ADAS domain on a virtual vehicle platform.

Build with modularity in mind, so your feature integrates smoothly into the broader SDV ecosystem.

Explore cross-domain interactions between software, simulated hardware components, and vehicle services.

This environment lets you **test ideas quickly**, iterate safely, and push the limits of what’s possible — without ever touching a physical car. The setup will mirror a real-world SDV stack, complete with simulated ECUs, sensors, and services, so you can focus on innovation instead of infrastructure.

## Architecture
![Cruise Control](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/architecture/sdv_lab_cc.png)
SDV Lab is an expansible framework where virtual ECUs (vECUs) from different technologies can be connected through an unified virtual bus based on [uProtocol](https://projects.eclipse.org/projects/automotive.uprotocol) (uStreamer).

Here we bring to you a simple use case of a Cruise Control feature, where you can use as a starting point to implement new features.

This use case is composed by the following main components:

 - [CARLA Simulator](https://carla.org/): open-source autonomous driving simulation platform that provides a high-fidelity, photorealistic environment for testing and developing self-driving vehicle technologies. Built on Unreal Engine, it supports a wide range of sensors (like cameras, LiDAR, radar, GPS) and allows users to create complex driving scenarios with dynamic weather, traffic, and pedestrian behavior. CARLA enables full control over vehicles and simulation parameters, making it ideal for SDV feature development.
 - **uStreamer** from the [uProtocol project](https://github.com/eclipse-uprotocol/up-streamer-rust) is a **generic, pluggable streaming component** designed to bridge communication between different transport layers in distributed systems. Written in Rust, it serves as a modular tool that can be integrated into various environments where message routing between protocols like SOME/IP, Zenoh, and MQTT5 is needed.
 - **Ankaios** is an open-source workload and container orchestration system developed specifically for automotive High Performance Computing (HPC) platforms. Ankaios provides a lightweight yet powerful solution for managing containerized and native applications across multiple nodes and virtual machines using a unified API.
 -  [Android Automotive OS (AAOS)](https://developer.android.com/training/cars/platforms/automotive-os) and [Cuttlefish](https://source.android.com/docs/devices/cuttlefish): **AAOS** is a version of Android tailored for in-vehicle infotainment systems. Unlike Android Auto, which mirrors a smartphone, AAOS runs natively on the car’s hardware and supports apps directly installed into the vehicle. **Cuttlefish** is a high-fidelity, configurable virtual Android device designed to emulate real hardware behavior for testing and development.
 - **PID Controller**: A simple PID (Proportional, Integral, Derivative) algorithm for controlling the required throttle to achieve the target speed defined by the cruise control feature. The code is implemented in RUST, using uProtocol to communicate with the vECUs.
 - **Ego Vehicle**: This component is responsible to interface with CARLA Client and create an abstration of a real vehicle, where sensors and actuators can be seemlessly implemented and accessed by the vECU networking. Ego Vehicle is also implemented in RUST and its based on uProtocol.
 - **ThreadX MXChip Board**: This board simulates a physical ECU in the SDV Lab, and operates with ThreadX Real-time operating system. The application implemented in this ECU has a simple mission: desengage the Cruise Control function whenever the pyshical button is pressed.
 

```
