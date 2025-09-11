/********************************************************************************
 * Copyright (c) 2023 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use std::{str, sync::Arc};

use async_trait::async_trait;
use log::info;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri};
use up_transport_mqtt5::{Mqtt5Transport, Mqtt5TransportOptions, MqttClientOptions};

// const WILDCARD_AUTHORITY:       &str = "*";         // any authority (service provider)
const WILDCARD_ENTITY_ID:       u32 = 0x0000_FFFF;  // any instance, any service
const WILDCARD_ENTITY_VERSION:  u32 = 0xFF;         // any version major
const WILDCARD_RESOURCE_ID:     u32 = 0xFFFF;       // any resource ID

struct PrintlnListener {}

#[async_trait]
impl UListener for PrintlnListener {
    async fn on_receive(&self, message: UMessage) {
        if let Some(msg_payload) = message.payload.clone() {
            if let Ok(msg_str) = str::from_utf8(&msg_payload) {
                info!(
                    "Received message payload: [{}] from source: [{}]",
                    msg_str,
                    message.source().expect("Failed to get source").to_uri(true)
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    println!("\n*** Started mqtt5_subscriber...");

    let mqtt_client_options = MqttClientOptions {
        broker_uri: "5.196.78.28:1883".to_string(),
        ..Default::default()
    };
    
    let mqtt_transport_options = Mqtt5TransportOptions {
        mqtt_client_options,
        ..Default::default()
    };

    // Use a generic authority for the client connection
    let client_authority = "subscriber".to_string();
    let client = Mqtt5Transport::new(mqtt_transport_options, client_authority).await?;

    // Connect to broker before registering listeners
    info!("Connecting to MQTT broker...");
    client.connect().await.map_err(|e| UStatus {
        code: up_rust::UCode::UNAVAILABLE.into(),
        message: Some(format!("Failed to connect: {}", e)),
        ..Default::default()
    })?;
    
    info!("Successfully connected to MQTT broker");

    let listener = Arc::new(PrintlnListener {});

    // Define multiple authorities to subscribe to
    // let authorities = ["threadx", "Vehicle_a", "sensor_hub"];
    
    // Only one authority to subscribe to
    let authorities = ["threadx"];

    // Subscribe to each authority
    for authority in authorities {
        let source_filter = UUri {
            authority_name:     authority.to_string(),
            ue_id:              WILDCARD_ENTITY_ID,
            ue_version_major:   WILDCARD_ENTITY_VERSION,
            resource_id:        WILDCARD_RESOURCE_ID,
            ..Default::default()
        };

        info!("Subscribing to authority: {} -> uURI: {}", authority, source_filter.to_uri(true));

        client
            .register_listener(&source_filter, None, listener.clone())
            .await?;
    }

    info!(
        "Successfully subscribed to {} authorit{}",
        authorities.len(),
        if authorities.len() > 1 { "ies" } else { "y" }
    );


    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
