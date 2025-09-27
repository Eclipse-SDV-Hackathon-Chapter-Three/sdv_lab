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

use std::str::FromStr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use clap::{Parser, Subcommand};
use log::{info, error, warn};
use up_transport_zenoh::{UPTransportZenoh, zenoh_config};
use up_rust::{UUri, UMessageBuilder, UTransport, UPayloadFormat};
use zenoh::Config;

#[derive(Parser, Debug)]
#[clap(author, version, about = "uProtocol Publisher - Send messages to multiple URIs", long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    
    #[clap(long, default_value = "127.0.0.1:7447", help = "Zenoh router endpoint")]
    endpoint: String,
    
    #[clap(long, default_value = "Publisher", help = "Publisher authority name")]
    authority: String,
    
    #[clap(
        long,
        default_value = "0x8001",
        help = "Publisher entity ID",
        value_parser = |s: &str| -> Result<u32, std::num::ParseIntError> {
            if s.starts_with("0x") || s.starts_with("0X") {
                u32::from_str_radix(&s[2..], 16)
            } else {
                s.parse::<u32>()
            }
        }
    )]
    entity_id: u32,
    
    #[clap(long, default_value_t = 2, help = "Publisher entity version major")]
    version_major: u8,
}


#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(
        about = "Publish messages to one or multiple URIs",
        long_about = "
Publish messages to one or multiple URIs

Usage example:
cargo run --bin up_pub -- args \\
  --uri \"EGOVehicle/0/2/8001\" --payload \"25.5\" \\
  --uri \"AAOS/0/2/8002\" --payload \"1\" \\
  --uri \"CruiseControl/0/2/8001\" --payload \"0.4\" \\
  --format text"
    )]
    Args {
        #[clap(long, help = "Target URI (format: authority/ue_id/ue_version/resource_id)", action = clap::ArgAction::Append)]
        uri: Vec<String>,
        
        #[clap(long, help = "Payload data to send", action = clap::ArgAction::Append)]
        payload: Vec<String>,
        
        #[clap(long, help = "Payload format: text, json, protobuf", action = clap::ArgAction::Append)]
        format: Vec<String>,
    },
    
    /// Publish from a JSON file
    File {
        #[clap(long, help = "Path to JSON file containing URI-payload pairs")]
        path: String,
    },
    
    /// Interactive mode - publish multiple messages with prompts
    Interactive,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageData {
    uri: String,
    payload: String,
    #[serde(default = "default_format")]
    format: String,
}

fn default_format() -> String {
    "text".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct MultipleMessages {
    messages: Vec<MessageData>,
}

#[derive(Debug)]
struct ParsedMessage {
    uri: String,
    payload: String,
    format: String,
}

fn parse_arguments(uris: Vec<String>, payloads: Vec<String>, formats: Vec<String>) -> Result<Vec<ParsedMessage>, String> {
    if uris.len() != payloads.len() {
        return Err(format!("Number of URIs ({}) must match number of payloads ({})", uris.len(), payloads.len()));
    }
    
    let mut messages = Vec::new();
    let default_format = "text".to_string();
    
    // Determine if we have a global format (last format applies to all)
    let global_format = if formats.len() == 1 && uris.len() > 1 {
        Some(formats[0].clone())
    } else {
        None
    };
    
    for (i, (uri, payload)) in uris.iter().zip(payloads.iter()).enumerate() {
        let format = if let Some(ref global_fmt) = global_format {
            // Use global format
            global_fmt.clone()
        } else if i < formats.len() {
            // Use individual format
            formats[i].clone()
        } else {
            // Use default format
            default_format.clone()
        };
        
        messages.push(ParsedMessage {
            uri: uri.clone(),
            payload: payload.clone(),
            format,
        });
    }
    
    Ok(messages)
}

fn parse_payload_format(format_str: &str) -> UPayloadFormat {
    match format_str.to_lowercase().as_str() {
        "json" => UPayloadFormat::UPAYLOAD_FORMAT_JSON,
        "protobuf" | "proto" => UPayloadFormat::UPAYLOAD_FORMAT_PROTOBUF,
        "text" | _ => UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
    }
}

// Helper function to create a Zenoh configuration (copied from simulator.rs)
fn get_zenoh_config(endpoint: &str) -> zenoh_config::Config {
    let zenoh_string = format!("{{ mode: 'peer', connect: {{ endpoints: [ 'tcp/{}' ] }} }}", endpoint);
    Config::from_json5(&zenoh_string).expect("Failed to load Zenoh config")
}

async fn create_transport(endpoint: &str, authority: &str, entity_id: u32, version_major: u8) -> Result<Arc<dyn UTransport>, Box<dyn std::error::Error>> {
    // Create publisher entity URI
    let publisher_uri = UUri::try_from_parts(authority, entity_id, version_major, 0)?;
    let publisher_uri_string: String = (&publisher_uri).into();

    info!("Initializing uProtocol transport with publisher URI: {}", publisher_uri_string);

    // Use the builder pattern like in simulator.rs
    let transport: Arc<dyn UTransport> = Arc::new(
        UPTransportZenoh::builder(authority)
            .expect("invalid authority name")
            .with_config(get_zenoh_config(endpoint))
            .build()
            .await?
    );

    Ok(transport)
}

async fn publish_message(transport: &Arc<dyn UTransport>, uri_str: &str, payload: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse URI - support both full URI format and parts format
    let uri = if uri_str.contains("://") {
        UUri::from_str(uri_str)?
    } else {
        // Parse as authority/ue_id/ue_version/resource_id
        let parts: Vec<&str> = uri_str.split('/').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid URI format. Expected: authority/ue_id/ue_version/resource_id, got: {}", uri_str).into());
        }
        
        let authority = parts[0];
        let ue_id: u32 = parts[1].parse()?;
        let ue_version: u8 = parts[2].parse()?;
        
        // Parse resource_id as hexadecimal if it starts with 0x, otherwise as decimal
        let resource_id: u16 = if parts[3].starts_with("0x") || parts[3].starts_with("0X") {
            u16::from_str_radix(&parts[3][2..], 16)?
        } else {
            // Try parsing as hex first (for values like 8001, 8002)
            u16::from_str_radix(parts[3], 16).unwrap_or_else(|_| {
                // If hex parsing fails, try decimal
                parts[3].parse().unwrap_or(0)
            })
        };
        
        UUri::try_from_parts(authority, ue_id, ue_version, resource_id)?
    };

    let payload_format = parse_payload_format(format);
    
    info!("Publishing to URI: {} with payload: {} (format: {})", String::from(&uri), payload, format);
    
    let message = UMessageBuilder::publish(uri)
        .build_with_payload(payload.to_string(), payload_format)?;
    
    if let Err(e) = transport.send(message).await {
        error!("Failed to publish message: {}", e);
        return Err(e.into());
    }
    
    info!("Successfully published message");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    let args = Args::parse();
    
    info!("*** Started uProtocol Publisher");

    // Create transport
    let transport = create_transport(&args.endpoint, &args.authority, args.entity_id, args.version_major).await?;
    
    match args.command {
        Commands::Args { uri, payload, format } => {
            if uri.is_empty() {
                return Err("At least one URI must be provided".into());
            }
            
            let messages = parse_arguments(uri, payload, format)?;
            
            info!("Publishing {} messages", messages.len());
            
            for msg in messages {
                if let Err(e) = publish_message(&transport, &msg.uri, &msg.payload, &msg.format).await {
                    error!("Failed to publish to {}: {}", msg.uri, e);
                } else {
                    println!("✓ Published to {}: {} ({})", msg.uri, msg.payload, msg.format);
                }
                
                // Small delay between messages
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        Commands::File { path } => {
            let file_content = std::fs::read_to_string(&path)?;
            let messages: MultipleMessages = serde_json::from_str(&file_content)?;
            
            info!("Publishing {} messages from file: {}", messages.messages.len(), path);
            
            for msg in messages.messages {
                if let Err(e) = publish_message(&transport, &msg.uri, &msg.payload, &msg.format).await {
                    error!("Failed to publish to {}: {}", msg.uri, e);
                } else {
                    println!("✓ Published to {}: {} ({})", msg.uri, msg.payload, msg.format);
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        
        Commands::Interactive => {
            info!("Interactive mode - Enter URI and payload pairs (Ctrl+C to exit)");
            
            loop {
                println!("\nEnter URI (authority/ue_id/ue_version/resource_id):");
                let mut uri_input = String::new();
                std::io::stdin().read_line(&mut uri_input)?;
                let uri = uri_input.trim();
                
                if uri.is_empty() {
                    warn!("Empty URI, skipping...");
                    continue;
                }
                
                println!("Enter payload:");
                let mut payload_input = String::new();
                std::io::stdin().read_line(&mut payload_input)?;
                let payload = payload_input.trim();
                
                println!("Enter format (text/json/protobuf) [default: text]:");
                let mut format_input = String::new();
                std::io::stdin().read_line(&mut format_input)?;
                let format = format_input.trim();
                let format = if format.is_empty() { "text" } else { format };
                
                if let Err(e) = publish_message(&transport, uri, payload, format).await {
                    error!("Failed to publish: {}", e);
                } else {
                    println!("✓ Message published successfully!");
                }
            }
        }
    }
    
    info!("uProtocol Publisher finished");
    Ok(())
}
