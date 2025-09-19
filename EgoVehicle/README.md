# EgoVehicle

## Building

The simplest way to build and run the EgoVehicle is to use the included scripts.

_Note_: It's assumed that Docker is installed.

### Build the Docker container

```shell
./build_docker_container.sh
```

### Enter the Docker container

```shell
./enter_docker_container.sh
```

Once inside of the docker container, you should be able to execute:

```shell
cargo build
```

and it ought to build!
