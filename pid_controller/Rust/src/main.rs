use pid_controller::PIDController;
use zenoh_handler::ZenohHandler;
use zenoh::prelude::r#async::*;
use log::info;

mod pid_controller;
mod zenoh_handler;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    info!("*** Started PID Controller");

    let kp = 0.125;
    let ki = kp / 8.0;
    let kd = kp / 10.0;

    println!("PID => Kp={}, Ki={}, Kd={}", kp, ki, kd);

    let pid = PIDController::new(kp, ki, kd);

    let session = zenoh::open(zenoh::prelude::Config::default()).res().await.unwrap();

    let handler = ZenohHandler::new(
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

    // Set up Ctrl+C handler
    let handler_clone = std::sync::Arc::new(handler);
    let handler_for_signal = handler_clone.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        
        println!("\nShutting down...");
        
        handler_for_signal.store_results();
        handler_for_signal.show_results();
        
        std::process::exit(0);
    });

    // Keep the main thread alive
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
