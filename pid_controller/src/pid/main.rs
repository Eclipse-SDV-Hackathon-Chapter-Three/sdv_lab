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

use pid_controller::PIDController;
use uprotocol_handler::UProtocolHandler;
use up_transport_zenoh::UPTransportZenoh;
use up_rust::UUri;
use log::info;

mod pid_controller;
mod uprotocol_handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    info!("*** Started PID Controller with uProtocol");

    let kp = 0.125;
    let ki = kp / 8.0;
    let kd = kp / 10.0;

    println!("PID => Kp={}, Ki={}, Kd={}", kp, ki, kd);

    let pid = PIDController::new(kp, ki, kd);

    // Create entity URI for the PID controller
    let entity_uri = UUri::try_from_parts("CruiseControl", 0, 2, 0)?;
    let entity_uri_string: String = (&entity_uri).into();

    // Initialize uProtocol transport with Zenoh
    let transport = UPTransportZenoh::new(Default::default(), entity_uri_string).await?;

    let handler = UProtocolHandler::new(pid, transport)?;

    handler.start().await?;

    println!("PID controller running with uProtocol (CTRL-C to terminate)...");

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
