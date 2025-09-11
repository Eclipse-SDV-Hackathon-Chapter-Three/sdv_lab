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

use std::{time::SystemTime};
use log::{error, info};
use up_rust::{UMessageBuilder, UPayloadFormat, UStatus, UTransport, UUri};
use up_transport_mqtt5::{Mqtt5Transport, Mqtt5TransportOptions, MqttClientOptions};

// pub source UUri definitions -> up://threadx/A/2/8001
const PUB_TOPIC_AUTHORITY:          &str = "threadx";
const PUB_TOPIC_UE_ID:              u32  = 0x000A;
const PUB_TOPIC_UE_VERSION_MAJOR:   u32  = 2;
const PUB_TOPIC_RESOURCE_ID:        u32  = 0x8001;

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    println!("\n*** Started mqtt5_publisher...");

    let mqtt_client_options = MqttClientOptions {
        // broker_uri: "localhost:1883".to_string(),
        broker_uri: "5.196.78.28:1883".to_string(),
        ..Default::default()
    };
    
    let mqtt_transport_options = Mqtt5TransportOptions {
        mqtt_client_options,
        ..Default::default()
    };

    let authority = PUB_TOPIC_AUTHORITY.to_string();
    let client = Mqtt5Transport::new(mqtt_transport_options, authority.clone()).await?;

    // Connect to broker
    client.connect().await?;

    let resource_1 = PUB_TOPIC_RESOURCE_ID;
    let authority_name_1 = PUB_TOPIC_AUTHORITY;

    // One resource from multiple authorities
    // let authority_names = [authority_name_1, "Vehicle_a", "sensor_hub"];
    // let resource_ids = [resource_1, resource_1 + 1, resource_1 + 2];
    
    // Only one resource from one authority
    let authority_names = [authority_name_1];
    let resource_ids = [resource_1];

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        for (authority_name, resource_id) in authority_names.iter().zip(resource_ids.iter()) {
            let source = UUri {
                authority_name:     authority_name.to_string(),
                ue_id:              PUB_TOPIC_UE_ID,
                ue_version_major:   PUB_TOPIC_UE_VERSION_MAJOR,
                resource_id:        *resource_id,
                ..Default::default()
            };

            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();            
            let payload_text = format!("Hello from '{}' - Resource: '0x{:X}' - UTC: {}", 
                                                authority_name, resource_id, current_time);        
            
            let message = UMessageBuilder::publish(source.clone())
                .with_ttl(1000)
                .build_with_payload(
                    payload_text.clone(),
                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                )
                .expect("Failed to build message");

            if let Err(e) = client.send(message).await {
                error!(
                    "Failed to publish message payload: [{payload_text}] to source: [{}] : '{e}'",
                    source.to_uri(true)
                );
            } else {
                info!(
                    "Successfully published message payload: [{payload_text}] to source: [{}]",
                    source.to_uri(true)
                );
            }
        }
    }
}
