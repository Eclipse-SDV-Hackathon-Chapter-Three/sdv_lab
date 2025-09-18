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
// publishes on []
// subscribes to [
//      AAOS/0/2/8001,
//      AAOS/0/2/8002,
//      AAOS/0/2/8003,
//      EGOVehicle/0/2/8001,
//      EGOVehicle/0/2/8002
// ]

use async_trait::async_trait;
use log::{debug, info, warn};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;
use zenoh::config::{Config, EndPoint};

// no publish topics

// subscribe topics
const SUB_TOPIC_1: &str = "AAOS/0/2/8001";
const SUB_TOPIC_2: &str = "AAOS/0/2/8002";
const SUB_TOPIC_3: &str = "AAOS/0/2/8003";
const SUB_TOPIC_4: &str = "EGOVehicle/0/2/8001";
const SUB_TOPIC_5: &str = "EGOVehicle/0/2/8002";

// id of the entity itself
const ENTITY_ID: &str = "CruiseControl/0/2/0";

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

    info!("Started zenoh_subscriber");

    let mut zenoh_config = Config::default();
    // Add the IPv4 endpoint to the Zenoh configuration
    zenoh_config
        .connect
        .endpoints
        .set(vec![
            EndPoint::from_str("tcp/127.0.0.1:7447").expect("Unable to set endpoint"),
        ])
        .expect("Unable to set Zenoh Config");

    let client_uri: String = (&subscriber_uuri()).into();
    let client: Arc<dyn UTransport> = Arc::new(
        UPTransportZenoh::new(zenoh_config, subscriber_uri)
            .await
            .unwrap(),
    );

    // TODO subscribe to the "subscribe topics" (no need to publish anything here)

    let source_filter = UUri::try_from_parts(
        PUB_TOPIC_AUTHORITY,
        PUB_TOPIC_UE_ID,
        PUB_TOPIC_UE_VERSION_MAJOR,
        PUB_TOPIC_RESOURCE_ID,
    )
    .unwrap();

    let publish_receiver: Arc<dyn UListener> = Arc::new(PublishReceiver);
    subscriber
        .register_listener(&source_filter, None, publish_receiver.clone())
        .await?;

    thread::park();
    Ok(())
}
