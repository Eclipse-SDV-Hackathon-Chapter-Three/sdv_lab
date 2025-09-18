use serde::{Serialize};
use serde_json;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use rand::Rng;
use zenoh::prelude::r#async::*;

#[derive(Serialize)]
struct ClockStatus {
    time: u64,
}

#[derive(Serialize)]
struct VelocityStatus {
    velocity: f64,
}

#[derive(Serialize)]
struct TargetSpeed {
    speed: f64,
}

#[derive(Serialize)]
struct EngageStatus {
    engaged: u8,
}

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::prelude::Config::default()).res().await.unwrap();

    let pub_clock = session.declare_publisher("vehicle/status/clock_status").res().await.unwrap();
    let pub_velocity = session.declare_publisher("vehicle/status/velocity_status").res().await.unwrap();
    let pub_target = session.declare_publisher("adas/cruise_control/target_speed").res().await.unwrap();
    let pub_engage = session.declare_publisher("adas/cruise_control/engage").res().await.unwrap();

    #[allow(unused_mut)]
    let mut engaged = 1;

    loop {
        let velocity = rand::rng().random_range(5.0..15.0);
        let target = rand::rng().random_range(10.0..20.0);

        // Getting system time as a timestamp in seconds
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let clock = ClockStatus { time: current_time };
        let vel = VelocityStatus { velocity };
        let tgt = TargetSpeed { speed: target };
        let eng = EngageStatus { engaged };

        // Payload needs to be formatted as a string
        pub_clock.put(serde_json::to_string(&clock).unwrap()).res().await.unwrap();
        pub_velocity.put(serde_json::to_string(&vel).unwrap()).res().await.unwrap();
        pub_target.put(serde_json::to_string(&tgt).unwrap()).res().await.unwrap();
        pub_engage.put(serde_json::to_string(&eng).unwrap()).res().await.unwrap();

        println!("Published: time={}, velocity={:.2}, target={:.2}, engaged={}", 
                current_time, velocity, target, engaged);

        // engaged = if engaged == 1 { 0 } else { 1 }; // toggle engagement for testing

        sleep(Duration::from_secs(2)).await;
    }
}
