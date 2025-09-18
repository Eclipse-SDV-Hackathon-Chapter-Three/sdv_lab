use pid_controller::PIDController;
use zenoh_handler::ZenohHandler;
use zenoh::prelude::r#async::*;

mod pid_controller;
mod zenoh_handler;

#[tokio::main]
async fn main() {
    let kp = 0.125;
    let ki = kp / 8.0;
    let kd = kp / 10.0;

    println!("PID => Kp={}, Ki={}, Kd={}", kp, ki, kd);

    let pid = PIDController::new(kp, ki, kd);

    let session = zenoh::open(zenoh::prelude::Config::default()).res().await.unwrap();

    
    let mut handler = ZenohHandler::new(
        pid,
        session,
        "vehicle/status/clock_status".to_string(),
        "vehicle/status/velocity_status".to_string(),
        "adas/cruise_control/target_speed".to_string(),
        "adas/cruise_control/engage".to_string(),
        "control/command/actuation_cmd".to_string(),
    );


    handler.start().await;

    println!("PID controller running (CTRL-C to terminate)...");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
