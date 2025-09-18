/********************************************************************************
 * Copyright (c) 2024 Contributors to the Eclipse Foundation
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

// AAOS
// publishes on [
//    AAOS/0/2/8001,
//    AAOS/0/2/8002,
//    AAOS/0/2/8003
// ]
// subscribes to [
//    EGOVehicle/0/2/8001
// ]

use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::time::SystemTime;
use up_rust::{UListener, UMessage, UMessageBuilder, UPayloadFormat, UStatus, UTransport, UUri};
use up_transport_mqtt5::{Mqtt5Transport, Mqtt5TransportOptions, MqttClientOptions};

// publish topics
const PUB_TOPIC_1: &str = "AAOS/0/2/8001";
const PUB_TOPIC_2: &str = "AAOS/0/2/8002";
const PUB_TOPIC_3: &str = "AAOS/0/2/8003";

// subscribe topics
const SUB_TOPIC_1: &str = "EGOVehicle/0/2/8001";

// authority of the entity itself
const ENTITY_AUTHORITY: &str = "AAOS";

struct PublishReceiver;

#[async_trait]
impl UListener for PublishReceiver {
    async fn on_receive(&self, msg: UMessage) {
        debug!("PublishReceiver: Received a message: {msg:?}");

        if let Some(payload) = msg.payload {
            info!("Message has payload: {payload:?}");
        } else {
            warn!("Message has no payload.")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    println!("\n*** Started AAOS...");

    let mqtt_client_options = MqttClientOptions {
        broker_uri: "5.196.78.28:1883".to_string(),
        ..Default::default()
    };

    let mqtt_transport_options = Mqtt5TransportOptions {
        mqtt_client_options,
        ..Default::default()
    };

    let authority = ENTITY_AUTHORITY.to_string();
    let client = Mqtt5Transport::new(mqtt_transport_options, authority.clone()).await?;

    // Connect to broker
    client.connect().await?;

    // TODO subscribe to the "subscribe topics"

    // TODO publish helloworld messages to every topic in the list of "publish topics" up top
    loop {
        for publish_topic in publish_topics {
            let topic = UUri::new().

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

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
