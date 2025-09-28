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

use async_trait::async_trait;
use carla::client::{ActorBase, Client};
use carla::sensor::data::{
    CollisionEvent, Image as ImageEvent, ImuMeasurement as ImuMeasurementEvent, LaneInvasionEvent,
    LidarMeasurement as LidarMeasurementEvent, ObstacleDetectionEvent,
    RadarMeasurement as RadarMeasurementEvent,
};
use carla_data_serde::{
    CollisionEventSerDe, ImageEventSerBorrowed, ImuMeasurementSerDe, LaneInvasionEventSerDe,
    LidarMeasurementSerBorrowed, ObstacleDetectionEventSerDe, RadarMeasurementSerBorrowed,
};
use clap::Parser;
use ego_vehicle::args::Args;
use ego_vehicle::helpers::setup_sensor_with_transport;
use ego_vehicle::sensors::{
    CollisionFactory, ImageFactory, ImuMeasurementFactory, LaneInvasionFactory,
    LidarMeasurementFactory, ObstacleDetectionFactory, RadarMeasurementFactory,
};
use log;
use serde_json;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use up_rust::{
    LocalUriProvider, StaticUriProvider, UListener, UMessage, UMessageBuilder, UPayloadFormat,
    UTransport, UUri,
};
use up_transport_zenoh::UPTransportZenoh;
use up_transport_zenoh::zenoh_config;
use zenoh::{Config, key_expr::KeyExpr};

// General constants
const CLIENT_TIME_MS: u64 = 5_000;
const POLLING_EGO_MS: u64 = 1_000;
const WAITING_PUB_MS: u64 = 1;
// Vehicle control constants
const MIN_THROTTLE: f32 = 0.0;
const MIN_STEERING: f32 = -1.0;
const MIN_BRAKING: f32 = 0.0;
const MID_STEERING: f32 = 0.0;
const MAX_THROTTLE: f32 = 1.0;
const MAX_STEERING: f32 = 1.0;
const MAX_BRAKING: f32 = 1.0;

// uProtocol resource IDs
const RESOURCE_VELOCITY_STATUS: u16 = 0x8001;
const RESOURCE_CLOCK_STATUS: u16 = 0x8002;
// uProtocol resource IDs for sensors
const RESOURCE_LANE_INVASION_SENSOR: u16 = 0x8010;
const RESOURCE_COLLISION_SENSOR: u16 = 0x8011;
const RESOURCE_OBSTACLE_DETECTION_SENSOR: u16 = 0x8012;
const RESOURCE_IMAGE_SENSOR: u16 = 0x8013;
const RESOURCE_RADAR_SENSOR: u16 = 0x8014;
const RESOURCE_LIDAR_SENSOR: u16 = 0x8015;
const RESOURCE_IMU_SENSOR: u16 = 0x8016;

// Helper function to create a Zenoh configuration
pub(crate) fn get_zenoh_config() -> zenoh_config::Config {
    let args = Args::parse();

    let zenoh_string = if let Some(router) = &args.router {
        format!(
            "{{ mode: 'peer', connect: {{ endpoints: [ 'tcp/{}:7447' ] }} }}",
            router
        )
    } else {
        "{ mode: 'peer' }".to_string()
    };

    let zenoh_config = Config::from_json5(&zenoh_string).expect("Failed to load Zenoh config");

    zenoh_config
}

// Listener for actuation command - implements the UListener trait for uProtocol
struct ActuationListener {
    data: Arc<Mutex<Option<String>>>, // Shared data structure to store the latest actuation command
}

#[async_trait]
impl UListener for ActuationListener {
    async fn on_receive(&self, msg: UMessage) {
        if let Some(payload) = msg.payload {
            // Convert the binary payload to a string
            let value =
                String::from_utf8(payload.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string());
            log::trace!("[from_uprotocol] actuation_cmd : {}", value);

            // Update the shared data structure with the new value
            // This is where the lock is acquired and the data is updated
            let mut data = self.data.lock().unwrap();
            *data = Some(value);
            // Lock is released when data goes out of scope
        }
    }
}

// Listener for engage status - implements the UListener trait for uProtocol
struct EngageListener {
    data: Arc<Mutex<Option<String>>>, // Shared data structure to store the latest engage status
}

#[async_trait]
impl UListener for EngageListener {
    async fn on_receive(&self, msg: UMessage) {
        if let Some(payload) = msg.payload {
            // Convert the binary payload to a string
            let value =
                String::from_utf8(payload.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string());
            log::trace!("[from_uprotocol] engage : {}", value);

            // Update the shared data structure with the new value
            // This is where the lock is acquired and the data is updated
            let mut data = self.data.lock().unwrap();
            *data = Some(value);
            // Lock is released when data goes out of scope
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // -- Parse command line arguments --
    let args = Args::parse();

    // Initiate logging
    pretty_env_logger::init();

    // Stop the program gracefully on Ctrl-C
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        log::warn!("Cancelled by user. Bye!");
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // -- CARLA configuration --

    // Connect to the Carla Server
    log::info!(
        "Connecting to the Carla Server at {}:{}...",
        args.host,
        args.port
    );

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
        carla_settings.synchronous_mode,
        carla_settings.fixed_delta_seconds
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
                if attribute.id() == "role_name"
                    && attribute.value_string() == args.ego_vehicle_role
                {
                    log::info!(
                        "Found '{}' actor with id: {}",
                        args.ego_vehicle_role,
                        actor.id()
                    );
                    ego_vehicle_id = Some(actor.id());
                    break;
                }
            }
        }

        // Sleep to avoid busy-waiting
        tokio::time::sleep(Duration::from_millis(POLLING_EGO_MS)).await;
    }

    // Initialize uProtocol logging
    UPTransportZenoh::try_init_log_from_env();

    // Create a uProtocol URI provider for this vehicle
    // This defines the identity of this node in the uProtocol network
    let uri_provider = StaticUriProvider::new("EGOVehicle", 0, 2);

    // Create the uProtocol transport using Zenoh as the underlying transport
    let transport: Arc<dyn UTransport> = Arc::new(
        UPTransportZenoh::builder(uri_provider.get_authority())
            .expect("invalid authority name")
            .with_config(get_zenoh_config())
            .build()
            .await?,
    );

    // Create shared data structures for uProtocol subscribers
    // These will store the latest values received from uProtocol messages
    let actuation_cmd: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let engage: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some(0.to_string())));

    // Register the actuation command listener with uProtocol
    // This listener will be called when messages matching the filter are received
    let actuation_filter = UUri::from_str("//CruiseControl/0/2/8001")?;
    log::info!(
        "Registering actuation command listener [filter: {}]",
        actuation_filter.to_uri(false)
    );
    transport
        .register_listener(
            &actuation_filter,
            None,
            Arc::new(ActuationListener {
                data: actuation_cmd.clone(),
            }),
        )
        .await?;

    // Register the engage listener with uProtocol
    // This listener will be called when messages matching the filter are received
    let engage_filter = UUri::from_str("//AAOS/0/2/8002")?;
    log::info!(
        "Registering engage listener [filter: {}]",
        engage_filter.to_uri(false)
    );
    transport
        .register_listener(
            &engage_filter,
            None,
            Arc::new(EngageListener {
                data: engage.clone(),
            }),
        )
        .await?;

    // Create topics for publishing uProtocol messages
    let clock_topic = uri_provider.get_resource_uri(RESOURCE_CLOCK_STATUS);
    let velocity_topic = uri_provider.get_resource_uri(RESOURCE_VELOCITY_STATUS);

    // Set up Zenoh session for traditional Zenoh subscribers
    let zenoh_session = zenoh::open(get_zenoh_config()).await.unwrap();

    // Define Zenoh topics to subscribe to
    let topic_throttle = KeyExpr::new("vehicle/status/throttle_status").unwrap();
    let topic_steering = KeyExpr::new("vehicle/status/steering_status").unwrap();
    let topic_braking = KeyExpr::new("vehicle/status/braking_status").unwrap();

    // Create Zenoh subscribers
    log::info!("Declaring Subscriber on '{}'...", &topic_throttle);
    let mut _subscriber_throttle = zenoh_session
        .declare_subscriber(&topic_throttle)
        .await
        .unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_steering);
    let mut _subscriber_steering = zenoh_session
        .declare_subscriber(&topic_steering)
        .await
        .unwrap();

    log::info!("Declaring Subscriber on '{}'...", &topic_braking);
    let mut _subscriber_braking = zenoh_session
        .declare_subscriber(&topic_braking)
        .await
        .unwrap();

    // Create shared data structures for Zenoh subscribers
    let throttle_sts: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let throttle_sts_clone = throttle_sts.clone();

    // Spawn a task to handle throttle status messages from Zenoh
    tokio::spawn(async move {
        while let Ok(sample) = _subscriber_throttle.recv_async().await {
            // Receive the payload and convert it to a string
            let payload = sample
                .payload()
                .try_to_string()
                .map(|s| s.to_string())
                .unwrap_or_else(|e| e.to_string());

            log::trace!("[from_zenoh] throttle_status : {}", payload);

            // Store the payload in the shared data structure
            let mut data = throttle_sts_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    // Spawn a task to handle steering status messages from Zenoh
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

            // Store the payload in the shared data structure
            let mut data = steering_sts_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    // Spawn a task to handle braking status messages from Zenoh
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

            // Store the payload in the shared data structure
            let mut data = braking_sts_clone.lock().unwrap();
            *data = Some(payload);
        }
    });

    // -- Set up Sensor for Lane Invasion -- (generic)
    let (_lane_comms, _ego_vehicle_sensor_lane_invasion_id, _lane_sensor_keepalive) =
        if let Some(ego_vehicle_sensor_lane_invasion_role) =
            args.ego_vehicle_sensor_lane_invasion_role
        {
            let uuri = uri_provider.get_resource_uri(RESOURCE_LANE_INVASION_SENSOR);

            // Encoder: LaneInvasionEvent -> Vec<u8>
            let encode = |evt: LaneInvasionEvent| {
                let serde_evt: LaneInvasionEventSerDe = evt.into();
                serde_json::to_vec(&serde_evt).map_err(|e| e.into())
            };

            let (_lane_comms, lane_actor_id, _lane_sensor_keepalive) = setup_sensor_with_transport(
                &carla_world,
                &running,
                &ego_vehicle_sensor_lane_invasion_role,
                "lane_invasion_sensor",
                POLLING_EGO_MS,
                LaneInvasionFactory,
                uuri,
                encode,
                UPayloadFormat::UPAYLOAD_FORMAT_JSON,
                Arc::clone(&transport),
            )
            .await
            .expect("Unable to set up lane sensor with transport");

            let _ego_vehicle_sensor_lane_invasion_id = Some(lane_actor_id);

            (
                Some(_lane_comms),
                Some(_ego_vehicle_sensor_lane_invasion_id),
                Some(_lane_sensor_keepalive),
            )
        } else {
            (None, None, None)
        };

    // -- Set up Sensor for Collision -- (generic)
    let (_collision_comms, _ego_vehicle_sensor_collision_id, _collision_sensor_keepalive) =
        if let Some(ego_vehicle_sensor_collision_role) = args.ego_vehicle_sensor_collision_role {
            let uuri = uri_provider.get_resource_uri(RESOURCE_COLLISION_SENSOR);

            // Encoder: CollisionEvent -> Vec<u8>
            let encode = |evt: CollisionEvent| {
                let serde_evt: CollisionEventSerDe = evt.into();
                serde_json::to_vec(&serde_evt).map_err(|e| e.into())
            };

            let (_collision_comms, collision_actor_id, _collision_sensor_keepalive) =
                setup_sensor_with_transport(
                    &carla_world,
                    &running,
                    &ego_vehicle_sensor_collision_role,
                    "collision_sensor",
                    POLLING_EGO_MS,
                    CollisionFactory,
                    uuri,
                    encode,
                    UPayloadFormat::UPAYLOAD_FORMAT_JSON,
                    Arc::clone(&transport),
                )
                .await
                .expect("Unable to set up collision sensor with transport");

            let _ego_vehicle_sensor_collision_id = Some(collision_actor_id);

            (
                Some(_collision_comms),
                Some(_ego_vehicle_sensor_collision_id),
                Some(_collision_sensor_keepalive),
            )
        } else {
            (None, None, None)
        };

    // -- Set up Sensor for Obstacle Detection -- (generic)
    let (
        _obstacle_detection_comms,
        _ego_vehicle_sensor_obstacle_detection_id,
        _obstacle_detection_sensor_keepalive,
    ) = if let Some(ego_vehicle_sensor_obstacle_detection_role) =
        args.ego_vehicle_sensor_obstacle_detection_role
    {
        let uuri = uri_provider.get_resource_uri(RESOURCE_OBSTACLE_DETECTION_SENSOR);

        // Encoder: ObstacleDetectionEvent -> Vec<u8>
        let encode = |evt: ObstacleDetectionEvent| {
            let serde_evt: ObstacleDetectionEventSerDe = evt.into();
            serde_json::to_vec(&serde_evt).map_err(|e| e.into())
        };

        let (
            _obstacle_detection_comms,
            obstacle_detection_actor_id,
            _obstacle_detection_sensor_keepalive,
        ) = setup_sensor_with_transport(
            &carla_world,
            &running,
            &ego_vehicle_sensor_obstacle_detection_role,
            "obstacle_detection_sensor",
            POLLING_EGO_MS,
            ObstacleDetectionFactory,
            uuri,
            encode,
            UPayloadFormat::UPAYLOAD_FORMAT_JSON,
            Arc::clone(&transport),
        )
        .await
        .expect("Unable to set up obstacle detection sensor with transport");

        let _ego_vehicle_sensor_obstacle_detection_id = Some(obstacle_detection_actor_id);

        (
            Some(_obstacle_detection_comms),
            Some(_ego_vehicle_sensor_obstacle_detection_id),
            Some(_obstacle_detection_sensor_keepalive),
        )
    } else {
        (None, None, None)
    };

    // -- Set up Sensor for Image -- (generic)
    let (_image_comms, _ego_vehicle_sensor_image_id, _image_sensor_keepalive) =
        if let Some(ego_vehicle_sensor_image_role) = args.ego_vehicle_sensor_image_role {
            let uuri = uri_provider.get_resource_uri(RESOURCE_IMAGE_SENSOR);

            // Encoder: ImageEvent -> Vec<u8> (borrow-only)
            let encode = |evt: ImageEvent| {
                // Borrow the event so the payload can serialize without copying the image buffer
                let serde_evt: ImageEventSerBorrowed<'_> = (&evt).into();
                serde_json::to_vec(&serde_evt)
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
            };

            let (_image_comms, image_actor_id, _image_sensor_keepalive) =
                setup_sensor_with_transport(
                    &carla_world,
                    &running,
                    &ego_vehicle_sensor_image_role,
                    "image_sensor",
                    POLLING_EGO_MS,
                    ImageFactory,
                    uuri,
                    encode,
                    UPayloadFormat::UPAYLOAD_FORMAT_JSON,
                    Arc::clone(&transport),
                )
                .await
                .expect("Unable to set up obstacle detection sensor with transport");

            let _ego_vehicle_sensor_image_id = Some(image_actor_id);

            (
                Some(_image_comms),
                Some(_ego_vehicle_sensor_image_id),
                Some(_image_sensor_keepalive),
            )
        } else {
            (None, None, None)
        };

    // -- Set up Sensor for RadarMeasurement -- (generic)
    let (
        _radar_measurement_comms,
        _ego_vehicle_sensor_radar_measurement_id,
        _radar_measurement_sensor_keepalive,
    ) = if let Some(ego_vehicle_sensor_radar_measurement_role) =
        args.ego_vehicle_sensor_radar_measurement_role
    {
        let uuri = uri_provider.get_resource_uri(RESOURCE_RADAR_SENSOR);

        // Encoder: RadarMeasurementEvent -> Vec<u8>
        let encode = |evt: RadarMeasurementEvent| {
            // Borrowed, zero-copy serializer
            let serde_evt: RadarMeasurementSerBorrowed<'_> = (&evt).into();
            serde_json::to_vec(&serde_evt)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        };

        let (
            _radar_measurement_comms,
            radar_measurement_actor_id,
            _radar_measurement_sensor_keepalive,
        ) = setup_sensor_with_transport(
            &carla_world,
            &running,
            &ego_vehicle_sensor_radar_measurement_role,
            "radar_measurement_sensor",
            POLLING_EGO_MS,
            RadarMeasurementFactory,
            uuri,
            encode,
            UPayloadFormat::UPAYLOAD_FORMAT_JSON,
            Arc::clone(&transport),
        )
        .await
        .expect("Unable to set up obstacle detection sensor with transport");

        let _ego_vehicle_sensor_radar_measurement_id = Some(radar_measurement_actor_id);

        (
            Some(_radar_measurement_comms),
            Some(_ego_vehicle_sensor_radar_measurement_id),
            Some(_radar_measurement_sensor_keepalive),
        )
    } else {
        (None, None, None)
    };

    // -- Set up Sensor for LidarMeasurement -- (generic)
    let (
        _lidar_measurement_comms,
        _ego_vehicle_sensor_lidar_measurement_id,
        _lidar_measurement_sensor_keepalive,
    ) = if let Some(ego_vehicle_sensor_lidar_measurement_role) =
        args.ego_vehicle_sensor_lidar_measurement_role
    {
        let uuri = uri_provider.get_resource_uri(RESOURCE_LIDAR_SENSOR);

        // Encoder: LidarMeasurementEvent -> Vec<u8> (borrow-only)
        let encode = |evt: LidarMeasurementEvent|
            -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
        {
            let serde_evt: LidarMeasurementSerBorrowed<'_> = (&evt).into();
            serde_json::to_vec(&serde_evt)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        };

        let (
            _lidar_measurement_comms,
            lidar_measurement_actor_id,
            _lidar_measurement_sensor_keepalive,
        ) = setup_sensor_with_transport(
            &carla_world,
            &running,
            &ego_vehicle_sensor_lidar_measurement_role,
            "lidar_measurement_sensor",
            POLLING_EGO_MS,
            LidarMeasurementFactory,
            uuri,
            encode,
            UPayloadFormat::UPAYLOAD_FORMAT_JSON,
            Arc::clone(&transport),
        )
        .await
        .expect("Unable to set up lidar measurement sensor with transport");

        let _ego_vehicle_sensor_lidar_measurement_id = Some(lidar_measurement_actor_id);

        (
            Some(_lidar_measurement_comms),
            Some(_ego_vehicle_sensor_lidar_measurement_id),
            Some(_lidar_measurement_sensor_keepalive),
        )
    } else {
        (None, None, None)
    };

    // -- Set up Sensor for ImuMeasurement -- (generic)
    let (
        _imu_measurement_comms,
        _ego_vehicle_sensor_imu_measurement_id,
        _imu_measurement_sensor_keepalive,
    ) = if let Some(ego_vehicle_sensor_imu_measurement_role) =
        args.ego_vehicle_sensor_imu_measurement_role
    {
        let uuri = uri_provider.get_resource_uri(RESOURCE_IMU_SENSOR);

        // Encoder: ImuMeasurementEvent -> Vec<u8> (borrow-only)
        let encode = |evt: ImuMeasurementEvent| {
            let serde_evt: ImuMeasurementSerDe = evt.into();
            serde_json::to_vec(&serde_evt).map_err(|e| e.into())
        };

        let (_imu_measurement_comms, imu_measurement_actor_id, _imu_measurement_sensor_keepalive) =
            setup_sensor_with_transport(
                &carla_world,
                &running,
                &ego_vehicle_sensor_imu_measurement_role,
                "imu_measurement_sensor",
                POLLING_EGO_MS,
                ImuMeasurementFactory,
                uuri,
                encode,
                UPayloadFormat::UPAYLOAD_FORMAT_JSON,
                Arc::clone(&transport),
            )
            .await
            .expect("Unable to set up imu measurement sensor with transport");

        let _ego_vehicle_sensor_imu_measurement_id = Some(imu_measurement_actor_id);

        (
            Some(_imu_measurement_comms),
            Some(_ego_vehicle_sensor_imu_measurement_id),
            Some(_imu_measurement_sensor_keepalive),
        )
    } else {
        (None, None, None)
    };

    // Main loop
    let mut last_time: f64 = 0.0;

    // Main loop
    while running.load(Ordering::SeqCst) {
        // Synchronize Carla's world and take a snapshot of the current frame
        let snapshot = carla_world.wait_for_tick();
        let timestamp = snapshot.timestamp();
        let delta_time = timestamp.platform_timestamp - last_time;

        if delta_time < args.delta {
            let secs = args.delta - delta_time;
            log::debug!("[to_sleep] secs : {}", secs);
            tokio::time::sleep(Duration::from_secs_f64(secs)).await;
        }

        last_time = timestamp.platform_timestamp;

        // Publish clock status via uProtocol
        let clock_payload = format!("{}", timestamp.elapsed_seconds);
        log::debug!("[to_uprotocol] clock_status : {}", clock_payload);

        let clock_message = UMessageBuilder::publish(clock_topic.clone())
            .build_with_payload(clock_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)?;
        transport.send(clock_message).await?;

        tokio::time::sleep(Duration::from_millis(WAITING_PUB_MS)).await;

        // Control the Ego Vehicle
        if let Some(actor) = carla_world.actor(ego_vehicle_id.unwrap()) {
            if let Ok(ego_vehicle) = actor.into_kinds().try_into_vehicle() {
                // Calculate and publish velocity
                let velocity = 3.6 * ego_vehicle.velocity().norm();
                let velocity_payload = format!("{}", velocity);

                // Publish velocity via uProtocol
                log::debug!("[to_uprotocol] velocity_status : {}", velocity_payload);
                let velocity_message = UMessageBuilder::publish(velocity_topic.clone())
                    .build_with_payload(
                        velocity_payload.clone(),
                        UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                    )?;
                transport.send(velocity_message).await?;

                // Initialize control values
                let mut throttle: f32 = MIN_THROTTLE;
                let mut steer: f32 = MID_STEERING;
                let mut brake: f32 = MIN_BRAKING;

                // Get steering value (Zenoh)
                {
                    let data_steering = steering_sts.lock().unwrap();
                    if let Some(ref payload) = *data_steering {
                        if let Ok(val) = payload.parse::<f32>() {
                            steer = val.clamp(MIN_STEERING, MAX_STEERING);
                        }
                    }
                }

                log::debug!("[from_manual] steering_sts: {steer}");

                // Check engage status (prioritize uProtocol)
                let engage_mode = {
                    let data_engage = engage.lock().unwrap();
                    if let Some(ref payload) = *data_engage {
                        payload.to_lowercase() != "0" // true for automatic mode, false for manual
                    } else {
                        false // default to manual mode
                    }
                };

                if !engage_mode {
                    // Manual mode - use throttle and brake from Zenoh
                    {
                        let data_throttle_sts = throttle_sts.lock().unwrap();
                        if let Some(ref payload) = *data_throttle_sts {
                            if let Ok(val) = payload.parse::<f32>() {
                                throttle = val
                            }
                        }
                    }

                    {
                        let data_braking_sts = braking_sts.lock().unwrap();
                        if let Some(ref payload) = *data_braking_sts {
                            if let Ok(val) = payload.parse::<f32>() {
                                brake = val
                            }
                        }
                    }

                    log::debug!("[from_manual] throttle_sts: {throttle}, braking_sts: {brake}");
                } else {
                    // Automatic mode - use PID output from actuation command
                    // Prioritize uProtocol actuation command
                    let mut pid_output: f32 = 0.0;

                    {
                        // scope blocking to release lock after checking the value
                        let data_actuation_cmd = actuation_cmd.lock().unwrap();
                        if let Some(ref payload) = *data_actuation_cmd {
                            if let Ok(val) = payload.parse::<f32>() {
                                pid_output = val;
                            }
                        }
                    }

                    log::debug!("[from_pid] actuation_cmd: {pid_output}");

                    if pid_output >= 0.0 {
                        throttle = pid_output.clamp(MIN_THROTTLE, MAX_THROTTLE);
                    } else {
                        brake = pid_output.abs().clamp(MIN_BRAKING, MAX_BRAKING);
                    }
                }

                // Apply control to the vehicle
                let mut control = ego_vehicle.control();

                control.throttle = throttle;
                control.steer = steer;
                control.brake = brake;

                log::debug!(
                    "[to_carla] throttle={}, steer={}, brake={}",
                    control.throttle,
                    control.steer,
                    control.brake
                );

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

    // Return success when the program exits
    Ok(())
}
