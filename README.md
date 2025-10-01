

# SDV Lab

Simulate a realistic, modular, and cyber-physical Software-Defined Vehicle (SDV) development environment where teams can design, implement, and demonstrate an SDV feature that
operates across a virtual platform.

SDV Lab is a collection of examples about how to use Eclipse projects and other Open Source projects, such as CARLA and Android, to rapidly develop realistic vehicle features, and then simulate and test them in a virtual environment.

![SDV Lab](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/assets/sdv_lab.png)

## ⚠️ Rust applications using uProtocol ⚠️

### tl;dr, do the following:

1 Modify your Cargo.toml like the following:

```
up-rust = { version = "=0.7.0" }
```

2 Run the following command on the command line, inside the folder for your Rust project:

```shell
cargo update -p up-rust --precise 0.7.0
```

3.a Run the following command on the command line, inside the folder for your Rust project:

```shell
cargo tree -p up-transport-zenoh
```

3.b Read through the output to confirm:

that you should see `up-transport-zenoh` using `up-rust` at `0.7.0`, not `0.7.1`.

4 When you commit to your own repo, ensure that you commit the lockfile changes:

```shell
git add Cargo.lock
```

### Why's this needed?

The [up-rust](https://crates.io/crates/up-rust/versions) crate got updated in its `0.7.1` release to more faithfully implement the uProtocol spec, specifically how `UUri`s are handled. See [here](https://github.com/eclipse-uprotocol/up-spec/blob/main/basics/uri.adoc#3-mapping-to-uri) for more details if interested where it's specified on what is legal in a `UUri`'s authority component.

The Rust build and packaging tool Cargo eagerly favors using newer releases.

In this case what's happening is that if you specify in your Cargo.toml:

```
up-rust = { version = "0.7.0" }
```

this will be eagerly upgraded to use `0.7.1`.

That's why we had step 1 up above.

Next, [`up-transport-zenoh`](https://crates.io/crates/up-transport-zenoh/0.8.0/dependencies) has its dependency on `up-rust` specified in the default way:

```
up-rust = { version = "0.7.0" }
```

which means that it's also going to try to use `0.7.1`.

By running the command in step 2 above we can also require all dependencies to use the `up-rust` release version `0.7.0`, including `up-transport-zenoh`.

## SDV Lab Framework

The SDV Lab repository is organized in folders, where you can find simple examples of components connected to implement unique vehicle functions. Inside the folders will you find README files with technical details about each example.

![SW Components samples](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/assets/SW_Components.png)
Each example is composed by 3 elements:
 - Software Component A: Usually CARLA or Android Automotive application (AAOS). This component will be deployed in the shared computers with GPU capabilities.
 - Software Component B: Application component developed in RUST or Python. This component can be developed and deployed in your own computer.
 - Communication Bus: Protocol channel where the message and signal between the 2 components will be exchanged. The following protocols are available: [uProtocol](https://github.com/eclipse-uprotocol), [MQTT5](https://github.com/eclipse-mosquitto/mosquitto) and [Zenoh](https://github.com/eclipse-zenoh/zenoh).

All Application Components and protocols bridges or brokers are containerized and managed using [Ankaios](https://github.com/eclipse-ankaios/ankaios). It means that you are able to run each example using simple Ankaios commands, as it will be described in the following sections.

## Infrastructure

For this event, each team will be provided with a dedicated Laptop (shared laptop) containing all the main SDV Lab components, which requires specific software and hardware configuration. Therefore, your main focus will be to develop applications to interact with those components.
![Infra](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/assets/infra.png)

Note that the Ankaios Architecture is based on 2 separate Nodes:

 - One Node dedicated to the Shared PC with "static" applications, such as MQTT Broker and uStreamer and,
 - Additional Nodes that will run in each User PC, where your application will be deployed.

![Ankaios Architecture](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/assets/Ankaios%20Arch.png)

Here is the list of applications provided by the Shared laptops:
 - CARLA Simulator 0.9.15
 - Android Studio
 - MQTT Mosquitto Broker
 - uStreamer (uProtocol)
 
 ## Installation

### User Notebook

On your notebook you will use [Eclipse Ankaios](https://eclipse-ankaios.github.io/ankaios/0.6) as embedded software orchestrator to start containerized workloads for this challenge. The example applications you can run are also managed by Ankaios.

#### Install Podman

As [Eclipse Ankaios](https://eclipse-ankaios.github.io/ankaios/0.6) as embedded software orchestrator is used to manage workloads inside the SDV Lab Challenge, you will need to install [Podman](https://docs.podman.io/en/latest/) as a container engine.

You can install Podman easily with the package manager like in Ubuntu:

```
sudo apt-get update
sudo apt-get -y install podman
```

Otherwise follow the official [Podman installation instructions](https://podman.io/docs/installation#installing-on-linux).

#### Install Eclipse Ankaios

Install [Eclipse Ankaios](https://eclipse-ankaios.github.io/ankaios/0.6) with a single curl according to the [Ankaios installation guide](https://eclipse-ankaios.github.io/ankaios/latest/usage/installation).

Follow the `Setup with script` section and install the version Eclipse Ankaios [v0.6.0](https://github.com/eclipse-ankaios/ankaios/releases/tag/v0.6.0).

**Note:** When using Ubuntu-24.04 disable AppArmor like described in the Ankaios installation guide.

The installation script will automatically create a systemd service file for the Ankaios server and an Ankaios agent.

### Shared Notebook

As an participant you can ignore this section. The shared notebooks were already setup and this is only linked to reinstall the services if the shared notebook has some troubles.
Look into the detailed [shared_notebooks.md](./shared_notebooks.md) about how to install the shared services on the shared notebooks from the hack coaches.

## If you're using Ubuntu 24.04

Be sure to run the script in:

```shell
./ego-vehicle/uprotocol-sensors/troubleshooting/setup-ubuntu-24.04-with-clang-12.sh
```

so that you're able to compile `ego-vehicle/uprotocol-sensors`.

## If using Ubuntu 22.04, but still hitting compilation errors

Consult [`ego-vehicle/uprotocol-sensors/README.md`](./ego-vehicle/uprotocol-sensors/README.md).

## Run

1. Start the required services on the shared notebook (CARLA, Ankaios cluster with communication service like MQTT, ...)
2. Run your applications using Ankaios on your notebook

### Run Ankaios

```shell
sudo systemctl start ank-server ank-agent
```

### Apply the workload example manifest

In each example workload folder you will find an Ankaios manifest which you can apply to your local running Ankaios cluster to start this example demo workload. Since the example workloads need to connect to services on the shared notebook, please find out the ip address of the shared notebook, connect your notebook to the Hack Wifi and replace remote address e.g. for MQTT broker to point to the external shared notebook NIC's ip address.

Navigate to the example workload folder you want to run and apply the manifest, example:

```shell
cd uprotocol/cruise-control-app
ank apply cruise-control-app.yaml
```

**Note:** If you want to remove all workloads specified in the Ankaios manifest for cleaning up you can simply add `-d` parameter to the `ank apply` like the following:
`ank apply -d cruise-control-app.yaml`. This might be helpful for incremental development, when you change the example code.

## Additional Ankaios commands

```
ank logs <workload_name> // Retrieve the logs from a workload
ank get state // Retrieve information about the current Ankaios system
ank get workloads // Information about the worloads running in the Ankaios system
```

## Workload starting other workloads

Inside [ankaios/example_workloads/README.md](./ankaios/example_workloads/README.md) there are two example workloads, one using the Ankaios Python SDK and the other one using the Ankaios Rust SDK, both using the [Ankaios Control Interface](https://eclipse-ankaios.github.io/ankaios/0.6/reference/control-interface/) to instruct Ankaios as a workload to start dynamically other workloads. This is common in the SDV world since workloads do not have to run always. Workloads can start other workloads or you can manage the Ankaios cluster also from within a workload. If your specific use case in the SDV Lab needs such feature, you can start with the example workloads there as a template and adapt it for your needs.

## MQTT Examples
### 1) Android AAOS + Python App
A Python-based MQTT implementation for publishing and subscribing to vehicle parameter data with intelligent Cruise Control status monitoring to an Android-based Panel Cluster.

 - [Link](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/tree/main/android_python)

#### Running the Example
 - Load Android project ( android_python/android/digital-cluster-app ) with Android Studio. Please follow the instructions of [shared_notebooks.md](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/shared_notebooks.md ) regarding Android Studio configuration.
 
 - Follow the instructions in the [README.md](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/android_python/python/README.md) file for running Ankaios.
 
 - If everything went well, you should see the Speed gauge from Android application increasing in a step of 5 km/h until reach 100 km/h, then decreasing.

### 2) ThreadX

Several examples about how to implement MQTT and uProtocol in Rust based on MXAZ3166 board. 
 - [Link](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/tree/main/android_treadx/threadx)

#### Running the Example
 - Load Android project (  android_treadx/android/digital-cluster-app ) with Android Studio. Please follow the instructions of [shared_notebooks.md](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/shared_notebooks.md ) regarding Android Studio configuration.

- Follow the [Quick Start guideline](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/android_treadx/threadx/README.md) to build and run ThreadX example.
 
## uProtocol Examples
### 1) Rust PID Controller
A real-time PID (Proportional-Integral-Derivative) controller system implemented in Rust, designed for velocity control applications using uProtocol over Zenoh middleware for distributed communication. The system implements cruise control functionality by computing acceleration commands to maintain desired vehicle speeds.

 - [Link](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/tree/main/pid_controller/rust-uprotocol)

#### Running the Example
Follow [README.md](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/pid_controller/rust-uprotocol/README.md) for all the instructions about how to run it using Ankaios.
 
### 2) Android AAOS + Rust App
The example uses Eclipse uProtocol to periodically publish the current operational status (e.g. current speed, engine temperature) and to expose an API endpoint for setting the target speed.

 - [Link](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/tree/add_rust_client_app/uprotocol/cruise-control-app)

#### Running the Example
 - Load Android project ( android_uprotocol/digital-cluster-app ) with Android Studio. Please follow the instructions of [shared_notebooks.md](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/shared_notebooks.md ) regarding Android Studio configuration.
 
 - Go the project folder:

 ```shell
cd uprotocol/cruise-control-app
```

 - Change the manifest file commandOptions with the Shared PC IP address for the MQTT broker (MQTT_BROKER_URI=mqtt://[MQTT IP]:1883)

 - Run with Ankaios:
```shell
ank apply cruise-control-app.yaml
```
### 3) CARLA Control & Sensors (uProtocol & Zenoh) --- Rust App + CARLA
A collection of Rust-based ego vehicle controllers for CARLA simulation with different messaging protocols. This project provides three implementations for distributed vehicle control and monitoring in automotive software-defined vehicle (SDV) architectures.

 - [Link](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/tree/main/ego-vehicle)

## Zenoh Examples
### 1) Python PID Controller
A real-time PID (Proportional-Integral-Derivative) controller system designed for velocity control applications, using Zenoh middleware for distributed communication. The system implements cruise control functionality by computing acceleration commands to maintain desired vehicle speeds.

 - [Link](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/tree/main/pid_controller/python-zenoh)
 
#### Running the Example
Follow [README.md](https://github.com/Eclipse-SDV-Hackathon-Chapter-Three/sdv_lab/blob/main/pid_controller/python-zenoh/README.md)

## SDV Lab - Challenge: Build Your Own ADAS or AD Feature

### 🧠 Challenge Overview

Your mission is to  **design and implement an Advanced Driver Assistance System (ADAS)**  or  **Autonomous Driving (AD)**  feature using the  **SDV Lab**  ecosystem.

You are free to choose any ADAS or AD functionality and bring it to life using Eclipse projects and others open-source tools and blueprints.

### 🎯 Objective

Create a  **functional prototype**  of an ADAS or AD feature that:
-   Uses SDV Lab framework, but you are free to choose any other Eclipse or Open-source solution.
-   Demonstrates innovation, usability, and integration.
-   Runs and test it in the simulated environment.
-   Deploy and automated your application by using Ankaios.

### 💡 Inspiration: ADAS & AD Feature Ideas

Here are some ideas to spark your creativity:
-   **Lane Detection & Departure Warning**
-   **Driver Monitoring System**  (drowsiness, distraction detection)
-   **Adaptive Cruise Control**
-   **Emergency Steering  & Emergency Braking**
-   **LiDAR-Camera Fusion for Object Detection**
-   **Traffic Sign Recognition**
-   **Autonomous Parking Assistant**

### 🧪 Evaluation Criteria

Your project will be judged based on:

-   **Usability**  – Is it intuitive and practical?
-   **Creativity**  – Is the idea novel or cleverly implemented?
-   **Technical Complexity**  – How sophisticated is the solution?
-   **Integration**  – How well does it use Eclipse solutions and blueprints?
-   **Completeness**  – Is it functional and demonstrable?

### 🧭 Getting Started

1.  Choose your ADAS or AD feature.
2.  Explore SDV Lab examples and Eclipse SDV Projects.
3.  Design and Prototype your solution.
4.  Create reasonable documentation of your team project and be ready for the presentation.

### 🏁 Final Notes

-   All code must be developed during the hackathon.
-   You can use simulators, virtual environments, and/or real hardware.
-   Coaches will be available to guide you through tooling and architecture.

## Prerequisites

- Linux Notebook Ubuntu 22.04/24.04 or WSL2 or a Linux VM
- Shared Notebook per team that you will get from the hack coaches





