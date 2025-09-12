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

/*!
This example illustrates how uProtocol's Transport Layer API can be used to publish
messages to a topic using the Zenoh transport.

This example works in conjunction with the `subscriber`, which should be started in
another terminal first.
*/

use std::{str::FromStr, time::SystemTime};
use log::{error, info};
use up_rust::{UMessageBuilder, UPayloadFormat, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;
use zenoh::config::{Config, EndPoint};

// topic uURI -> up://hpc/A/2/8001
const PUB_TOPIC_AUTHORITY: &str         = "hpc";

// topic uURI -> up://threadx/A/2/8001
const PUB_TOPIC_AUTHORITY_2: &str       = "threadx";

// topic uURI -> up://carla/A/2/8001
// const PUB_TOPIC_AUTHORITY_3: &str       = "carla";

const PUB_TOPIC_UE_ID: u32              = 0x000A;
const PUB_TOPIC_UE_VERSION_MAJOR: u8    = 2;
const PUB_TOPIC_RESOURCE_ID: u32        = 0x8001;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("\n*** Started zenoh_publisher...");

    let mut zenoh_config = Config::default();
    // Add the IPv4 endpoint to the Zenoh configuration
    zenoh_config
        .connect
        .endpoints
        .set(vec![
            EndPoint::from_str("tcp/127.0.0.1:7447").expect("Unable to set endpoint"),
        ])
        .expect("Unable to set Zenoh Config");

    let uri_provider = UUri {
        authority_name: PUB_TOPIC_AUTHORITY.to_string(),
        ue_id: PUB_TOPIC_UE_ID,
        ue_version_major: PUB_TOPIC_UE_VERSION_MAJOR as u32,
        resource_id: 0,
        ..Default::default()
    };
    let client = UPTransportZenoh::new(zenoh_config, uri_provider.to_uri(false))
        .await
        .unwrap();

    /*
     * Possible combinations of different authorities and resource_ids to publish to
     * |----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
     * |                                                                        pub source UUri definitions                                                                                     |
     * |----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
     * | "up://PUB_TOPIC_AUTHORITY/A/2/PUB_TOPIC_RESOURCE_ID"   |   "up://PUB_TOPIC_AUTHORITY/A/2/(PUB_TOPIC_RESOURCE_ID + 1)"  |   "up://PUB_TOPIC_AUTHORITY/A/2/(PUB_TOPIC_RESOURCE_ID + 2)"  |
     * | "up://PUB_TOPIC_AUTHORITY_2/A/2/PUB_TOPIC_RESOURCE_ID" |   "up://PUB_TOPIC_AUTHORITY_2/A/2/(PUB_TOPIC_RESOURCE_ID + 1)"|   "up://PUB_TOPIC_AUTHORITY_2/A/2/(PUB_TOPIC_RESOURCE_ID + 2)"|
     * | "up://PUB_TOPIC_AUTHORITY_3/A/2/PUB_TOPIC_RESOURCE_ID" |   "up://PUB_TOPIC_AUTHORITY_3/A/2/(PUB_TOPIC_RESOURCE_ID + 1)"|   "up://PUB_TOPIC_AUTHORITY_3/A/2/(PUB_TOPIC_RESOURCE_ID + 2)"|
     * |----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
     */

    /*
     * Placeholder to allow the definition of multiple authorities to publish to
     */
    // let authorities = [PUB_TOPIC_AUTHORITY, PUB_TOPIC_AUTHORITY_2, PUB_TOPIC_AUTHORITY_3];
    // let authorities = [PUB_TOPIC_AUTHORITY, PUB_TOPIC_AUTHORITY_2];
    let authorities = [PUB_TOPIC_AUTHORITY];

    /*
     * Placeholder to allow the definition of multiple resource_ids to publish to
     */
    // let resource_ids = [PUB_TOPIC_RESOURCE_ID, PUB_TOPIC_RESOURCE_ID + 1, PUB_TOPIC_RESOURCE_ID + 2];
    // let resource_ids = [PUB_TOPIC_RESOURCE_ID, PUB_TOPIC_RESOURCE_ID + 1];
    let resource_ids = [PUB_TOPIC_RESOURCE_ID];

    loop {
        for authority_name in authorities {
            for resource_id in resource_ids {
                let source = UUri {
                    authority_name:     authority_name.to_string(),
                    ue_id:              PUB_TOPIC_UE_ID,
                    ue_version_major:   PUB_TOPIC_UE_VERSION_MAJOR as u32,
                    resource_id:        resource_id,
                    ..Default::default()
                };

                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();            
                let payload_text = format!("Hello from '{}' - Resource: '0x{:X}' using {} - UTC: {current_time}", 
                                        authority_name, resource_id, 
                                        std::any::type_name_of_val(&client).split("::").last().unwrap_or("Unknown"));

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

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    }
}
