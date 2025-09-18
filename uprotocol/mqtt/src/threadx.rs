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

// Threadx
// publishes on [
//     Threadx/0/2/8001
// ]
// subscribes to []

use log::{error, info};
use std::time::SystemTime;
use up_rust::{UMessageBuilder, UPayloadFormat, UStatus, UTransport, UUri};
use up_transport_mqtt5::{Mqtt5Transport, Mqtt5TransportOptions, MqttClientOptions};

// publish topics
const PUB_TOPIC_1: &str = "Threadx/0/2/8001";

// no subscribe topics

// id of the entity itself
const ENTITY_ID: &str = "Threadx/0/2/0";

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    println!("\n*** Started mqtt5_publisher...");

    let mqtt_client_options = MqttClientOptions {
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

    // TODO publish helloworld messages to every topic in the list of "publish topics" up top (no need to subscribe to anything)

    let authorities = [PUB_TOPIC_AUTHORITY];

    let resource_ids = [PUB_TOPIC_RESOURCE_ID];

    loop {
        for authority_name in authorities {
            for resource_id in resource_ids {
                let source = UUri {
                    authority_name: authority_name.to_string(),
                    ue_id: PUB_TOPIC_UE_ID,
                    ue_version_major: PUB_TOPIC_UE_VERSION_MAJOR,
                    resource_id: resource_id,
                    ..Default::default()
                };

                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let payload_text = format!(
                    "Hello from '{}' - Resource: '0x{:X}' using {} - UTC: {current_time}",
                    authority_name,
                    resource_id,
                    std::any::type_name_of_val(&client)
                        .split("::")
                        .last()
                        .unwrap_or("Unknown")
                );

                let message = UMessageBuilder::publish(source.clone())
                    .with_ttl(1000)
                    .build_with_payload(payload_text.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
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

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
