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

use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use rand::Rng;

use clap::Parser;
use log::{info, error};
use up_transport_zenoh::{UPTransportZenoh, zenoh_config};
use up_rust::{LocalUriProvider, StaticUriProvider, UUri, UMessageBuilder, UTransport, UPayloadFormat};
use zenoh::{Config};

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
    
    info!("*** Started uProtocol Publisher");

    // Create a uProtocol URI provider for the PID controller
    // This defines the identity of this node in the uProtocol network
    let uri_provider = StaticUriProvider::new("VehicleSimulator", 0, 2);
    
    // Initialize uProtocol transport with Zenoh
    let transport = UPTransportZenoh::builder(uri_provider.get_authority())
        .expect("invalid authority name")
        .with_config(get_zenoh_config())
        .build()
        .await?;


    // Create URIs for publishing according to the mapping table
    let clock_uri = UUri::try_from_parts("EGOVehicle", 0, 2, 0x8002)?;      // vehicle/status/clock_status
    let velocity_uri = UUri::try_from_parts("EGOVehicle", 0, 2, 0x8001)?;   // vehicle/status/velocity_status
    let target_uri = UUri::try_from_parts("AAOS", 0, 2, 0x8001)?;           // adas/cruise_control/target_speed
    let engage_uri = UUri::try_from_parts("AAOS", 0, 2, 0x8002)?;           // adas/cruise_control/engage

    info!("uProtocol Publisher initialized with URIs:");
    info!("  Clock: {}", String::from(&clock_uri));
    info!("  Velocity: {}", String::from(&velocity_uri));
    info!("  Target Speed: {}", String::from(&target_uri));
    info!("  Engage: {}", String::from(&engage_uri));

    #[allow(unused_mut)]
    let mut engaged = 1;

    loop {
        let velocity = rand::rng().random_range(5.0..15.0);
        let target = rand::rng().random_range(10.0..20.0);

        // Getting system time as a timestamp in seconds
        let current_time: f64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        // Publish current timestamp
        let clock_payload = format!("{}", current_time);
        let message = UMessageBuilder::publish(clock_uri.clone())
            .build_with_payload(clock_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
            .unwrap();
        
        if let Err(e) = transport.send(message).await {
            error!("Failed to publish clock: {}", e);
        } else {
            info!("Publishing clock timestamp: {}", clock_payload);
        }

        // Publish current velocity
        let velocity_payload = format!("{}", velocity);
        let message = UMessageBuilder::publish(velocity_uri.clone())
            .build_with_payload(velocity_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
            .unwrap();
        
        if let Err(e) = transport.send(message).await {
            error!("Failed to publish velocity: {}", e);
        } else {
            info!("Publishing velocity: {}", velocity_payload);
        }

        // Publish target speed
        let target_payload = format!("{}", target);
        let message = UMessageBuilder::publish(target_uri.clone())
            .build_with_payload(target_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
            .unwrap();
        
        if let Err(e) = transport.send(message).await {
            error!("Failed to publish target speed: {}", e);
        } else {
            info!("Publishing target speed: {}", target_payload);
        }

        // Publish engage status
        let engage_payload = format!("{}", engaged);
        let message = UMessageBuilder::publish(engage_uri.clone())
            .build_with_payload(engage_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
            .unwrap();
        
        if let Err(e) = transport.send(message).await {
            error!("Failed to publish engage status: {}", e);
        } else {
            info!("Publishing engage status: {}", engage_payload);
        }

        println!("Published uProtocol messages: time={:.4}, velocity={:.2}, target={:.2}, engaged={}", 
                current_time, velocity, target, engaged);

        // Uncomment to toggle engagement for testing
        // engaged = if engaged == 1 { 0 } else { 1 };

        sleep(Duration::from_secs(2)).await;
    }
}
