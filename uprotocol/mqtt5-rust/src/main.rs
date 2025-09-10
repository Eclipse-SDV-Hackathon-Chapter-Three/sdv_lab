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

use std::{str::FromStr, time::SystemTime};

use backon::{ExponentialBuilder, Retryable};
use clap::{command, Parser};
use log::{error, info};
use up_rust::{UMessageBuilder, UPayloadFormat, UStatus, UTransport, UUri};
use up_transport_mqtt5::{Mqtt5Transport, Mqtt5TransportOptions, MqttClientOptions};


/// Publishes messages to a given topic using the MQTT 5 transport.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Command {
    /// The uProtocol topic to publish messages to.
    #[arg(value_name = "URI", env = "TOPIC", default_value = "up://threadx/000A/2/8001", value_parser = UUri::from_str)]
    topic: UUri,
}

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    let command = Command::parse();
    let authority = command.topic.authority_name.clone();

    let mqtt_client_options = MqttClientOptions {
        // broker_uri: "localhost:1883".to_string(),
        broker_uri: "5.196.78.28:1883".to_string(),
        ..Default::default()
    };
    let mqtt_transport_options = Mqtt5TransportOptions {
        mqtt_client_options,
        ..Default::default()
    };

    let client = Mqtt5Transport::new(mqtt_transport_options, authority).await?;

    (|| {
        info!("Connecting to broker...");
        client.connect()
    })
    .retry(ExponentialBuilder::default())
    .when(|err| {
        error!("Connection attempt failed: {err}");
        true
    })
    .await?;

    loop {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let payload_text = format!("Hello world - {current_time}");
        let message = UMessageBuilder::publish(command.topic.clone())
            .with_ttl(1000)
            .build_with_payload(
                payload_text.clone(),
                UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
            )
            .expect("Failed to build message");

        if let Err(e) = client.send(message).await {
            error!(
                "Failed to publish message [topic: {}]: {}",
                command.topic.to_uri(true),
                e
            );
        } else {
            info!(
                "Successfully published message [topic: {}] with payload: [{}]",
                command.topic.to_uri(true),
                payload_text
            );
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
