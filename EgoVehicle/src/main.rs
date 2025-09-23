//
// Copyright (c) 2025 The X-Verse <https://github.com/The-Xverse>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use carla::client::{ActorBase, Client};

use clap::Parser;
use log;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use zenoh::{bytes::Encoding, key_expr::KeyExpr, Config};

// General constants
const CLIENT_TIME_MS: u64 = 5_000;
const POLLING_EGO_MS: u64 = 1_000;
const WAITING_PUB_MS: u64 = 1;

// Vehicle control constants
const MIN_THROTTLE: f32 =  0.0;
const MIN_STEERING: f32 = -1.0;
const MIN_BRAKING:  f32 =  0.0;

const MID_STEERING: f32 = 0.0;

const MAX_THROTTLE: f32 = 1.0;
const MAX_STEERING: f32 = 1.0;
const MAX_BRAKING:  f32 = 1.0;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "127.0.0.1")]
    host: String,
    #[clap(long, default_value_t = 2000)]
    port: u16,
    #[clap(long, default_value = "ego_vehicle")]
    role: String,
    #[clap(long, default_value_t = 0.100)]
    delta: f64,
    #[clap(long, default_value = None)]
    router: Option<String>,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initiate logging
    pretty_env_logger::init();

    // Stop the program gracefully on Ctrl-C
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        log::warn!("Cancelled by user. Bye!");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Connect to the Carla Server
    log::info!("Connecting to the Carla Server at {}:{}...", args.host, args.port);

    let mut carla_client = Client::connect(&args.host, args.port, None);

    carla_client.set_timeout(Duration::from_millis(CLIENT_TIME_MS));

    // Configure Carla's World
    let mut carla_world = carla_client.world();
    let mut carla_settings = carla_world.settings();

    carla_settings.synchronous_mode = false;
    carla_settings.fixed_delta_seconds = Some(args.delta);

    carla_world.apply_settings(&carla_settings, Duration::from_millis(CLIENT_TIME_MS));

    log::info!(
        "World Settings: Synchronous mode: {}, Fixed delta seconds: {:?}",
         carla_settings.synchronous_mode, carla_settings.fixed_delta_seconds
     );

    // Wait for the Ego Vehicle actor
    let mut ego_vehicle_id: Option<u32> = None;

    while running.load(Ordering::SeqCst) && ego_vehicle_id.is_none() {
        log::info!("Waiting for the Ego Vehicle actor...");

        // Syncronize Carla's world
        let _ = carla_world.wait_for_tick();

        // Check if the Ego Vehicle actor exists in the world
        for actor in carla_world.actors().iter() {
            for attribute in actor.attributes().iter() {
                if attribute.id() == "role_name" && attribute.value_string() == args.role {
                    log::info!("Found '{}' actor with id: {}", args.role, actor.id());
                    ego_vehicle_id = Some(actor.id());
                    break;
                }
            }
        }

        // Sleep to avoid busy-waiting
        tokio::time::sleep(Duration::from_millis(POLLING_EGO_MS)).await;
    }

    // Set up Zenoh session, subscribers and publishers
    log::info!("Opening the Zenoh session...");

    let zenoh_string = if let Some(router) = &args.router {
        format!("{{ mode: 'peer', connect: {{ endpoints: [ 'tcp/{}:7447' ] }} }}", router)
    } else {
        "{ mode: 'peer' }".to_string()
    };

    let zenoh_config = Config::from_json5(&zenoh_string).expect("Failed to load Zenoh config");

    log::info!("Zenoh configuration: {:?}", zenoh_config);

    let zenoh_session = zenoh::open(zenoh_config).await.unwrap();

    // Subscribe topics
    let topic_throttle   = KeyExpr::new("vehicle/status/throttle_status").unwrap();
    let topic_steering   = KeyExpr::new("vehicle/status/steering_status").unwrap();
    let topic_braking    = KeyExpr::new("vehicle/status/braking_status").unwrap();
    let topic_actuation  = KeyExpr::new("control/command/actuation_cmd").unwrap();
    let topic_engage     = KeyExpr::new("adas/cruise_control/engage").unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_throttle);

    let mut _subscriber_throttle = zenoh_session.declare_subscriber(&topic_throttle).await.unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_steering);

    let mut _subscriber_steering = zenoh_session.declare_subscriber(&topic_steering).await.unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_braking);

    let mut _subscriber_braking = zenoh_session.declare_subscriber(&topic_braking).await.unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_actuation);

    let mut _subscriber_actuation = zenoh_session.declare_subscriber(&topic_actuation).await.unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_engage);

    let mut _subscriber_engage = zenoh_session.declare_subscriber(&topic_engage).await.unwrap();

    // Attach a callback to the subscriber to handle incoming messages
    let throttle_sts: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let throttle_sts_clone = throttle_sts.clone();

    tokio::spawn(async move {
        while let Ok(sample) = _subscriber_throttle.recv_async().await {
            // Receive the payload and convert it to a string
            let payload = sample
                .payload()
                .try_to_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|e| e.to_string());

            log::trace!("[from_zenoh] throttle_status : {}", payload);

            // Store the payload
            let mut data = throttle_sts_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    let steering_sts: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let steering_sts_clone = steering_sts.clone();

    tokio::spawn(async move {
        while let Ok(sample) = _subscriber_steering.recv_async().await {
            // Receive the payload and convert it to a string
            let payload = sample
                .payload()
                .try_to_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|e| e.to_string());

            log::trace!("[from_zenoh] steering_status : {}", payload);

            // Store the payload
            let mut data = steering_sts_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    let braking_sts: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let braking_sts_clone = braking_sts.clone();

    tokio::spawn(async move {
        while let Ok(sample) = _subscriber_braking.recv_async().await {
            // Receive the payload and convert it to a string
            let payload = sample
                .payload()
                .try_to_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|e| e.to_string());

            log::trace!("[from_zenoh] braking_sts : {}", payload);

            // Store the payload
            let mut data = braking_sts_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    let actuation_cmd: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let actuation_cmd_clone = actuation_cmd.clone();

    tokio::spawn(async move {
        while let Ok(sample) = _subscriber_actuation.recv_async().await {
            // Receive the payload and convert it to a string
            let payload = sample
                .payload()
                .try_to_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|e| e.to_string());

            log::trace!("[from_zenoh] actuation_cmd : {}", payload);

            // Store the payload
            let mut data = actuation_cmd_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    let engage: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some(0.to_string())));
    let engage_clone = engage.clone();

    tokio::spawn(async move {
        while let Ok(sample) = _subscriber_engage.recv_async().await {
            // Receive the payload and convert it to a string
            let payload = sample
                .payload()
                .try_to_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|e| e.to_string());

            log::trace!("[from_zenoh] engage : {}", payload);

            // Store the payload
            let mut data = engage_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    // Publish topics
    let topic_clock    = KeyExpr::new("vehicle/status/clock_status").unwrap();
    let topic_velocity = KeyExpr::new("vehicle/status/velocity_status").unwrap();

    log::info!("Declaring a Zenoh Publisher on '{topic_clock}'...");
    log::info!("Declaring a Zenoh Publisher on '{topic_velocity}'...");

    let publisher_clock = zenoh_session.declare_publisher(&topic_clock).await.unwrap();
    let publisher_velocity = zenoh_session.declare_publisher(&topic_velocity).await.unwrap();

    let topic_clock_str = topic_clock.to_string();
    let topic_velocity_str = topic_velocity.to_string();

    publisher_clock
        .matching_listener()
        .callback(move |matching_status| {
            if matching_status.matching() {
                log::info!("Publisher has at least one subscriber for '{}'.", topic_clock_str);
            } else {
                log::info!("Publisher has NO MORE subscribers for '{}'.", topic_clock_str);
            }
        })
        .background()
        .await
        .unwrap();

    publisher_velocity
        .matching_listener()
        .callback(move |matching_status| {
            if matching_status.matching() {
                log::info!("Publisher has at least one subscriber for '{}'.", topic_velocity_str);
            } else {
                log::info!("Publisher has NO MORE subscribers for '{}'.", topic_velocity_str);
            }
        })
        .background()
        .await
        .unwrap();

    // Main loop
    let mut last_time: f64 = 0.0;
    let attachment: Option<String> = None;

    while running.load(Ordering::SeqCst) {
        // Syncronize Carla's world and takes a snapshot of the current frame
        let snapshot = carla_world.wait_for_tick();
        let timestamp = snapshot.timestamp();
        let delta_time = timestamp.platform_timestamp - last_time;

        if delta_time < args.delta {
            let secs = args.delta - delta_time;
            log::debug!("[to_sleep] secs : {}", secs);
            tokio::time::sleep(Duration::from_secs_f64(secs)).await;
        }

        last_time = timestamp.platform_timestamp;

        let mut payload = format!("{}", timestamp.elapsed_seconds);

        log::debug!("[to_zenoh] clock_status : {}", payload);

        publisher_clock
            .put(payload)
            .encoding(Encoding::TEXT_PLAIN) // Optionally set the encoding metadata
            .attachment(attachment.clone()) // Optionally add an attachment
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(WAITING_PUB_MS)).await;

        // Control the Ego Vehicle
        if let Some(actor) = carla_world.actor(ego_vehicle_id.unwrap()) {
            if let Ok(ego_vehicle) = actor.into_kinds().try_into_vehicle() {
                let velocity = 3.6 * ego_vehicle.velocity().norm();

                payload = format!("{}", velocity);

                log::debug!("[to_zenoh] velocity_status : {}", payload);

                publisher_velocity
                    .put(payload)
                    .encoding(Encoding::TEXT_PLAIN) // Optionally set the encoding metadata
                    .attachment(attachment.clone()) // Optionally add an attachment
                    .await
                    .unwrap();

                let mut throttle: f32 = MIN_THROTTLE;
                let mut steer: f32 = MID_STEERING;
                let mut brake: f32 = MIN_BRAKING;

                let data_steering = steering_sts.lock().unwrap();
                if let Some(ref payload) = *data_steering {
                    if let Ok(val) = payload.parse::<f32>() {
                        steer = val.clamp(MIN_STEERING, MAX_STEERING);
                    }
                }

                log::debug!("[from_manual] steering_sts: {steer}");

                let data_engage = engage.lock().unwrap();
                if let Some(ref payload) = *data_engage {
                    if payload.to_lowercase() == "0" {
                        let data_throttle_sts = throttle_sts.lock().unwrap();
                        if let Some(ref payload) = *data_throttle_sts {
                            if let Ok(val) = payload.parse::<f32>() {
                                throttle = val;
                            }
                        }

                        let data_braking_sts = braking_sts.lock().unwrap();
                        if let Some(ref payload) = *data_braking_sts {
                            if let Ok(val) = payload.parse::<f32>() {
                                brake = val;
                            }
                        }

                        log::debug!("[from_manual] throttle_sts: {throttle}, braking_sts: {brake}");
                    } else {
                        let mut pid_output: f32 = 0.0;

                        let data_actuation_cmd = actuation_cmd.lock().unwrap();
                        if let Some(ref payload) = *data_actuation_cmd {
                            if let Ok(val) = payload.parse::<f32>() {
                                pid_output = val;
                            }
                        }

                        log::debug!("[from_pid] actuation_cmd: {pid_output}");

                        if pid_output >= 0.0 {
                            throttle = pid_output.clamp(MIN_THROTTLE, MAX_THROTTLE);
                        } else {
                            brake = pid_output.abs().clamp(MIN_BRAKING, MAX_BRAKING);
                        }
                    }
                }

                let mut control = ego_vehicle.control();

                control.throttle = throttle;
                control.steer = steer;
                control.brake = brake;


                log::debug!("[to_carla] throttle={}, steer={}, brake={}",
                    control.throttle,
                    control.steer,
                    control.brake);

                ego_vehicle.apply_control(&control);
            } else {
                log::error!("Ego Vehicle actor is not a Vehicle type!");
                running.store(false, Ordering::SeqCst);
            }
        } else {
            log::warn!("Ego Vehicle actor not found in the world anymore!");
        }
    }

    log::info!("Exiting the main loop. Bye!");
}
