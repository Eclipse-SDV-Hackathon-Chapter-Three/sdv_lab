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

use log::info;
use clap::Parser;
use up_transport_zenoh::{UPTransportZenoh, zenoh_config};
use up_rust::{LocalUriProvider, StaticUriProvider};
use zenoh::{Config};

use pid_controller::PIDController;
use uprotocol_handler::UProtocolHandler;

mod pid_controller;
mod uprotocol_handler;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "127.0.0.1")]
    host: String,
    #[clap(long, default_value_t = 2000)]
    port: u16,
    #[clap(long, default_value = "CruiseControl")]
    role: String,
    #[clap(long, default_value_t = 0.100)]
    delta: f64,
    #[clap(long, default_value = None)]
    router: Option<String>,
}

// Helper function to create a Zenoh configuration
pub(crate) fn get_zenoh_config() -> zenoh_config::Config {
    let args = Args::parse();

    let zenoh_string = if let Some(router) = &args.router {
        format!("{{ mode: 'peer', connect: {{ endpoints: [ 'tcp/{}:7447' ] }} }}", router)
    } else {
        "{ mode: 'peer' }".to_string()
    };

    let zenoh_config = Config::from_json5(&zenoh_string).expect("Failed to load Zenoh config");

    zenoh_config
}

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

    // Create a uProtocol URI provider for the PID controller
    // This defines the identity of this node in the uProtocol network
    let uri_provider = StaticUriProvider::new("CruiseControl", 0, 2);
    
    // Initialize uProtocol transport with Zenoh
    let transport = UPTransportZenoh::builder(uri_provider.get_authority())
        .expect("invalid authority name")
        .with_config(get_zenoh_config())
        .build()
        .await?;

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
