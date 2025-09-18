use serde::{Serialize};
use tokio::time::{sleep, Duration};
use rand::Rng;
use zenoh::prelude::r#async::*;

#[derive(Serialize)]
struct ClockStatus {
    time: f64,
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
    engaged: bool,
}

#[tokio::main]
async fn main() {
    let session = zenoh::open(zenoh::prelude::Config::default()).res().await.unwrap();

    let pub_clock = session.declare_publisher("vehicle/status/clock_status").res().await.unwrap();
    let pub_velocity = session.declare_publisher("vehicle/status/velocity_status").res().await.unwrap();
    let pub_target = session.declare_publisher("adas/cruise_control/target_speed").res().await.unwrap();
    let pub_engage = session.declare_publisher("adas/cruise_control/engage").res().await.unwrap();

    let mut time = 0.0;
    let mut engaged = true;

    loop {
        time += 0.1;
        let velocity = rand::thread_rng().gen_range(5.0..15.0);
        let target = rand::thread_rng().gen_range(10.0..20.0);

        let clock = ClockStatus { time };
        let vel = VelocityStatus { velocity };
        let tgt = TargetSpeed { speed: target };
        let eng = EngageStatus { engaged };

        pub_clock.put(serde_json::to_vec(&clock).unwrap()).res().await.unwrap();
        pub_velocity.put(serde_json::to_vec(&vel).unwrap()).res().await.unwrap();
        pub_target.put(serde_json::to_vec(&tgt).unwrap()).res().await.unwrap();
        pub_engage.put(serde_json::to_vec(&eng).unwrap()).res().await.unwrap();

        println!("Published: time={:.2}, velocity={:.2}, target={:.2}, engaged={}", time, velocity, target, engaged);

        //engaged = !engaged; // toggle engagement for testing
        sleep(Duration::from_secs(1)).await;
    }
}