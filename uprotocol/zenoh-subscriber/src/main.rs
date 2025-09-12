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

use async_trait::async_trait;
use log::{debug, info, warn};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;
use zenoh::config::{Config, EndPoint};

// const WILDCARD_AUTHORITY: &str      = "*";         // any authority (service provider)
// const WILDCARD_ENTITY_ID: u32       = 0xFFFF_FFFF;  // any instance, any service
// const WILDCARD_ENTITY_VERSION: u8   = 0xFF;         // any version major
// const WILDCARD_RESOURCE_ID: u16     = 0xFFFF;       // any resource ID

// topic uURI -> up://threadx/A/2/8001
const PUB_TOPIC_AUTHORITY: &str         = "threadx";

// topic uURI -> up://hpc/A/2/8001
const PUB_TOPIC_AUTHORITY_2: &str       = "hpc";

// topic uURI -> up://hpc/A/2/8001
// const PUB_TOPIC_AUTHORITY_3: &str       = "carla";

const PUB_TOPIC_UE_ID: u32              = 0x000A;
const PUB_TOPIC_UE_VERSION_MAJOR: u8    = 2;
const PUB_TOPIC_RESOURCE_ID: u16        = 0x8001;

const SUB_TOPIC_AUTHORITY: &str         = "carla";
const SUB_TOPIC_UE_ID: u32              = 0x5BB0;
const SUB_TOPIC_UE_VERSION_MAJOR: u8    = 1;

fn subscriber_uuri() -> UUri {
    UUri::try_from_parts(
        SUB_TOPIC_AUTHORITY,
        SUB_TOPIC_UE_ID,
        SUB_TOPIC_UE_VERSION_MAJOR,
        0,
    )
    .unwrap()
}

struct PublishReceiver;

#[async_trait]
impl UListener for PublishReceiver {
    async fn on_receive(&self, msg: UMessage) {
        debug!("PublishReceiver: Received a message: {msg:?}");

        if let Some(payload) = msg.payload {
            let uri_str = msg.attributes.unwrap().source.unwrap().to_uri(true);
            info!("Received message payload: [{payload:?}] from source: [{uri_str}]");
        } else {
            warn!("Message has no payload.")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();

    println!("\n*** Started zenoh_subscriber");

    let mut zenoh_config = Config::default();
    // Add the IPv4 endpoint to the Zenoh configuration
    zenoh_config
        .connect
        .endpoints
        .set(vec![
            EndPoint::from_str("tcp/127.0.0.1:7447").expect("Unable to set endpoint"),
        ])
        .expect("Unable to set Zenoh Config");

    let subscriber_uri: String = (&subscriber_uuri()).into();
    let subscriber: Arc<dyn UTransport> = Arc::new(
        UPTransportZenoh::new(zenoh_config, subscriber_uri)
            .await
            .unwrap(),
    );

    let publish_receiver: Arc<dyn UListener> = Arc::new(PublishReceiver);

    /*
     * Placeholder to allow the definition of multiple authorities to subscribe to
     * pub source UUri definitions:
     *  - "up://PUB_TOPIC_AUTHORITY/A/2/PUB_TOPIC_RESOURCE_ID"
     *  - "up://PUB_TOPIC_AUTHORITY_2/A/2/PUB_TOPIC_RESOURCE_ID"
     *  - "up://PUB_TOPIC_AUTHORITY_3/A/2/PUB_TOPIC_RESOURCE_ID"
     */
    // let authorities = [PUB_TOPIC_AUTHORITY, PUB_TOPIC_AUTHORITY_2, PUB_TOPIC_AUTHORITY_3];
    let authorities = [PUB_TOPIC_AUTHORITY, PUB_TOPIC_AUTHORITY_2];
    // let authorities = [PUB_TOPIC_AUTHORITY];

    // Subscribe to each authority
    for authority in authorities {
        let source_filter =UUri::try_from_parts(
            authority,
            PUB_TOPIC_UE_ID,
            PUB_TOPIC_UE_VERSION_MAJOR,
            PUB_TOPIC_RESOURCE_ID
        ).unwrap();

        subscriber
            .register_listener(&source_filter, None, publish_receiver.clone())
            .await
            .map(|_| {
                log::info!("Successfully subscribed to: {}", source_filter.to_uri(true));
            })
            .map_err(|e| {
                println!("Failed to subscribe to: {} - Error: {}", source_filter.to_uri(true), e);
                log::error!("Failed to subscribe to: {} - Error: {}", source_filter.to_uri(true), e);
                e
            })?;     
    }

    info!(
        "Successfully subscribed to {} topic{}",
        authorities.len(),
        if authorities.len() > 1 { "s" } else { "" }
    );
    
    thread::park();
    Ok(())
}
