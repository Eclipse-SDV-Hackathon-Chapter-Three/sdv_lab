# Troubleshooting

## If you're using Ubuntu 24.04

Be sure to run the script in:

```shell
./troubleshooting/setup-ubuntu-24.04-with-clang-12.sh
```

so that you can compile this project.

Make sure to open a new terminal or:

```shell
source ~/.bashrc
```

to make environment variables available.

tl;dr: The Rust binding to the CARLA Client API used in this program assumes clang-12 is available. The above script configures your environment correctly with clang-12 to be able to build this program.

## If you're using Ubuntu 22.04 and hitting compiler errors even still

Try to run this script to add additional compiler flags:

```shell
./troubleshooting/setup-compiler-flags.sh
```

Make sure to open a new terminal or:

```shell
source ~/.bashrc
```

to make environment variables available.
