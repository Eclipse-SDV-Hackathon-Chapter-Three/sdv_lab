# **uprotocol**

all of the stuff from the magic realm of uprotocol

Right now there are just two components here but more will come.
The first is a uEntity called zenoh-subscriber, which uses a Zenoh network to subscribe to uMessages with a topic filter **up/threadx/000A/2/8001**. Whenever it gets a message it just prints the payload of it without any decoding or parsing.
The second is called the uStreamer. Its not technically here, but there is a reference docker-compose which pulls a "configurable" uStreamer from the uprotocol GHCR. You can also find reference configuration files here.

## **the threadX board**

If you want to use this example together with the rust-threadx setup then simply go to that repo, clone it and run the "network" example.
You will have to enter your WIFI credentials (search for __WIFI_SSID__ and __WIFI_PASSWORD__ in the network.rs example), and make sure that the UURI that the board publishes on matches the streamer setup.

## **zenoh-subscriber**

The subscriber is a very basic uEntity which uses the up-transport-zenoh-rust library to listen for uMessages with a selected topic filter.
### You can start it with
```bash
RUST_LOG=info cargo run
```

You might have to set RUST_LOG=info or trace to see something, without anyone sending uMessages it will not do anything. If the streamer is also running and you have flashed the threadx-rust network example onto one of the boards, you should start seeing temperatures being logged here.

## **mqtt5-rust**

This project provides publisher and subscriber examples as a basic uEntity which uses the up-transport-mqtt5-rust library to publish and subscribe uMessages in a specific source (both are prepared to handle multiple sources.see commented lines).

### You can start it with

#### publisher
```bash
RUST_LOG=info cargo run --bin mqtt5_publisher
```

#### subscriber
```bash
RUST_LOG=info cargo run --bin mqtt5_subscriber
```

You might have to set RUST_LOG=info or trace to see something, not setting the env var will not display any information in the terminal, however it will be working in the background until it's terminated.

This can be used as a starting point to develop your own uEntities publisher and subscriber, as well as simulates the threadx board messages sending, once that the publisher example is hard coding the same source uURI as the threadx board ("up://threadx/A/2/8001").

## **ustreamer**

To start the uStreamer in the example configuration just run

```bash
docker compose up
```

and watch the streamer being pulled and started.

If you are a sudo user, remember to use sudo before the docker command:
```bash
sudo docker compose up
```

The streamer has a fairly complex setup but the basics are already included here:

#### **CONFIG.json**
The highest level configuration file. Do not rename it as its specified like that in the entrypoint layer of the docker image.

The top part of the config is pretty boring as the streamer uURI is not actually doing anything at the moment, but its looking for it anyways so it must be included.

usubscription_config is what tracks which uEntity is subscribed where, so that the streamer does not forward pub/sub messages that no one is subscribed to. If you want to forward pub/sub messages then you must always manually add them here (although in the future this will be done automatically somehow).

The interesting parts are under the "transports" section. The streamer forwards messages based on "authority" which is the first segment of all uURIs. For each transport (for now zenoh and MQTT5) the streamer creates one "endpoint" for each authority and then creates a mapping of which authority should be forwarded to which other authority. In this example there are three endpoints: "carla", "hpc" and "threadx".
The mappings are:
- "carla_endpoint" -> "hpc_endpoint"
- "hpc_endpoint" -> "carla_endpoint
- "threadx_endpoint" -> "carla_endpoint", "hpc_endpoint"
So all in all hpc and carla are mutually interconnected and threadx is publishing to everyone but not receiving anything. If you are changing the mapping, make sure that each endpoint name is actually defined or the streamer will crash.

#### **subscribton_data.json**

The subscription info follows the following pattern:

"/threadx/000A/2/8001": ["//carla/000A/2/1234", "//hpc/000A/2/1234"]

where messages that get published by key-uEntities get forwarded to the the list of value-uEntities.
Any messages that get published from an entity that is not listed here will not be forwarded by the streamer even if there is a forwarding rule from its endpoint in the CONFIG.json5!

#### **MQTT5_CONFIG.json5**

Standard mqtt setup parameters. The most interesting one is the hostname of the MQTT broker. The one that is set up here by default is the free (and completely unsecured) test.mosquitto.org broker (which is a nice one if you dont want to navigate local networks)

#### **ZENOH_CONFIG.json5**

This config schema comes straight from how Zenoh expects it. I cant explain it all in detail here but it is explained in the Zenoh documentation somewhere.
