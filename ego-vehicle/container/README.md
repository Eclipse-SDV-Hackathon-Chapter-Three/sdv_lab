# EgoVehicle

## Building

The simplest way to build and run the EgoVehicle is to use the included scripts.

_Note_: It's assumed that `Podman` is installed.

### Build for development

For development, you might want to build often until your source code builds successfully.

```shell
./make_container.sh
```

Next, enter the container.

You'll want to start an ssh-agent, which key to use will depend:

```shell
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519   # or whatever key you use for GitHub
ssh -T git@github.com       # should greet you on the host
```

```shell
./enter_container.sh
```

Once inside of the docker container, you should be able to execute:

```shell
cargo build
```

and it ought to build!

Use this mechanism until you have ready application and your build runs successful.

### Build for deployment

After completing your development, run the EgoVehicle as workload using Eclipse Ankaios.

```shell
./make_container.sh PROD
```

Next, if there is any existing `ego_vehicle` workload running delete it to deploy the new version afterwards:

```shell
ank delete workload ego_vehicle
```

Now, apply the Ankaios manifest again to run the whole scenario with your new `ego_vehicle` version:

```shell
ank apply ../ankaios/cruise_control.yaml
```

Check the logs:

```shell
ank logs -f ego_vehicle
```

