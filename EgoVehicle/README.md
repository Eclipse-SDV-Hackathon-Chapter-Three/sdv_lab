# EgoVehicle

## Building

The simplest way to build and run the EgoVehicle is to use the included scripts.

_Note_: It's assumed that Docker is installed.

### Build the Docker container

```shell
./build_docker_container.sh
```

### Enter the Docker container

You'll want to start an ssh-agent, which key to use will depend:

```shell
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519   # or whatever key you use for GitHub
ssh -T git@github.com       # should greet you on the host
```

```shell
./enter_docker_container.sh
```

Once inside of the docker container, you should be able to execute:

```shell
cargo build
```

and it ought to build!
