# Carla Simulator Setup and API/Libs build

## Prerequisites

- OS
  - Ubuntu 22.04


## Set Up Tools

```bash
sudo apt update
sudo apt install -y git python3-pip
```

```bash
pip install --user rust-just
```

### If you have troubles to install just in Ubuntu 22.04

#### Setting up Prebuilt-MPR

Run the following to set up the APT repository on your system:
```bash
wget -qO - 'https://proget.makedeb.org/debian-feeds/prebuilt-mpr.pub' | gpg --dearmor | sudo tee /usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg 1> /dev/null
echo "deb [arch=all,$(dpkg --print-architecture) signed-by=/usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg] https://proget.makedeb.org prebuilt-mpr $(lsb_release -cs)" | sudo tee /etc/apt/sources.list.d/prebuilt-mpr.list
sudo apt update
```

#### Install just using apt
```bash
sudo apt install just
```

## Set Up Environment

### CARLA Simulator

```bash
$ just
Available recipes:
    [Carla Build]
    build-libcarla                              # Build CARLA API (C++)
    make-carla                                  # Build CARLA API (Rust)

    [Carla Client]
    install-client                              # Install the CARLA Python API
    run-automatic host="127.0.0.1" port="2000"  # Run automatic control with Zenoh
    run-manual router="127.0.0.1" host="127.0.0.1" port="2000" # Run manual control with Zenoh
    uninstall-client                            # Uninstall CARLA Python API

    [Carla Server]
    install-server                              # Install the CARLA Simulator
    server-nvidia quality="Epic" port="2000"    # Run CARLA off-screen using NVIDIA card
    server-offscreen quality="Epic" port="2000" # Run CARLA in off-screen mode
    server-windowed quality="Epic" port="2000"  # Run CARLA in windowed mode
    uninstall-server                            # Uninstall CARLA Simulator

    [Utilities]
    check-host expected="ubuntu"                # Check current host
    fix-wsl                                     # Fix WSL permission issues
```

### Verify Environment

```bash
just check-host
```

### Troubleshooting

If you are using WSL, you may face the following error when running the `check-host` command.

```bash
$ just check-host
error: Recipe `_check_host` with shebang `#!/usr/bin/env -S bash -x` execution error: Permission denied (os error 13)
error: Recipe `check-host` failed on line 64 with exit code 1
```

To fix that, please run the following commands before proceeding.

```bash
just fix-wsl && source ~/.bashrc
```

Then run 'check-host' again and check if you get the same output below. If so, you are ready to proceed.

```bash
$ just check-host

You are running on host 'ubuntu'.
```

### CARLA Server

#### Install

```bash
just install-server
```

#### Test

```bash
just server-windowed
```

### CARLA Client

#### Install

```bash
just install-client
```

#### Test (Terminal #1)

```bash
just server-offscreen
```

#### Test (Terminal #2)

```bash
just run-automatic
```

### CARLA Build

# Build CARLA API (C++)

```bash
just build-libcarla
```
Built files will be available at 'carla-setup/localBuild' folder

# Build CARLA API (Rust)

```bash
just make-carla
```
Built files will be available at 'carla-setup/localBuild' folder