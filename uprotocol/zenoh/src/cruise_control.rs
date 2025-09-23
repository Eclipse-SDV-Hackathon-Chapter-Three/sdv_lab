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

// CruiseControl
// publishes on [
//      CruiseControl/0/2/8001
// ]
// subscribes to [
//     AAOS/0/2/8001,
//     AAOS/0/2/8002,
//     AAOS/0/2/8003,
//     EGOVehicle/0/2/8001,
//     EGOVehicle/0/2/8002,
//     Threadx/0/2/8001
// ]

use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::thread;
use std::{str::FromStr, time::SystemTime};
use up_rust::{UListener, UMessage, UMessageBuilder, UPayloadFormat, UStatus, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;
use zenoh::config::{Config, EndPoint};

// subscribe
const AAOS_AUTH: &str = "AAOS";
const EGO_AUTH: &str = "EGOVehicle";
const X_AUTH: &str = "Threadx";

const UEID: u32 = 0;
const VERSION: u8 = 2;

// authority of the entity itself
const CRUISE_AUTH: &str = "CruiseControl";

struct PublishReceiver;

#[async_trait]
impl UListener for PublishReceiver {
    async fn on_receive(&self, msg: UMessage) {
        debug!("PublishReceiver: Received a message: {msg:?}");

        if let Some(payload) = &msg.payload {
            let source = &msg.source().unwrap().to_uri(false);
            info!("Message has payload: {payload:?} and was published on {source}");
        } else {
            warn!("Message has no payload.");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    info!("Started cruise control");

    // --- Lists of pubish and subscribe topics ---
    let pub_topics =
        [UUri::try_from_parts(CRUISE_AUTH, UEID, VERSION, 0x8001).expect("Invalid UURI")];
    let sub_topics = [
        UUri::try_from_parts(AAOS_AUTH, UEID, VERSION, 0x8001).expect("Invalid UURI"),
        UUri::try_from_parts(AAOS_AUTH, UEID, VERSION, 0x8002).expect("Invalid UURI"),
        UUri::try_from_parts(AAOS_AUTH, UEID, VERSION, 0x8003).expect("Invalid UURI"),
        UUri::try_from_parts(EGO_AUTH, UEID, VERSION, 0x8001).expect("Invalid UURI"),
        UUri::try_from_parts(EGO_AUTH, UEID, VERSION, 0x8002).expect("Invalid UURI"),
        UUri::try_from_parts(X_AUTH, UEID, VERSION, 0x8001).expect("Invalid UURI"),
    ];
    // --- End of Lists of pubish and subscribe topics ---

    // --- Zenoh Transport Specific Stuff ---
    let mut zenoh_config = Config::default();
    // Add the IPv4 endpoint to the Zenoh configuration
    zenoh_config
        .connect
        .endpoints
        .set(vec![
            EndPoint::from_str("tcp/127.0.0.1:7447").expect("Unable to set endpoint"),
        ])
        .expect("Unable to set Zenoh Config");

    let client: Arc<dyn UTransport> = Arc::new(
        UPTransportZenoh::new(
            zenoh_config,
            UUri::try_from_parts(CRUISE_AUTH, UEID, VERSION, 0).expect("Invalid UURI"),
        )
        .await
        .unwrap(),
    );
    // --- End of Zenoh Transport Specific Stuff ---

    // --- Creation of Publishing Tasks ---
    for pub_topic in pub_topics {
        let client = client.clone();
        tokio::spawn(async move {
            loop {
                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let payload_text = format!(
                    "Hello from '{}' - Resource: '0x{:X}' using {} - UTC: {current_time}",
                    CRUISE_AUTH,
                    UEID,
                    std::any::type_name_of_val(&*client)
                        .split("::")
                        .last()
                        .unwrap_or("Unknown")
                );

                let message = UMessageBuilder::publish(pub_topic.clone())
                    .with_ttl(1000)
                    .build_with_payload(payload_text.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
                    .expect("Failed to build message");

                if let Err(e) = client.send(message).await {
                    error!(
                        "Failed to publish message payload: [{payload_text}] to source: [{}] : '{e}'",
                        pub_topic
                    );
                } else {
                    info!(
                        "Successfully published message payload: [{payload_text}] to source: [{}]",
                        pub_topic.to_uri(true)
                    );
                }

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });
    }
    // --- End of Creation of Publishing Tasks ---

    // --- Registration of subscription Receivers ---
    for sub_topic in sub_topics {
        let publish_receiver: Arc<dyn UListener> = Arc::new(PublishReceiver);
        client
            .register_listener(&sub_topic, None, publish_receiver.clone())
            .await
            .unwrap();
        info!("Subscribed to {}", sub_topic.to_string());
    }
    // --- End of Registration of subscription Receivers ---

    thread::park();
    Ok(())
}
