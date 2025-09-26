/*!
This example illustrates how uProtocol's _Communication Level API_ can be used to implement
a simple cruise control component which periodically reports its status and supports setting
the target speed.

The example implements a simple in-memory state and supports the following operations:
- Set target speed to a given value

The operations are exposed as uProtocol service endpoints using an in-memory RPC server.

Additionally, the example periodically publishes status messages containing current operational
information, including the current speed. The status messages are published to a configurable
uProtocol topic.

The example supports two different transports: Zenoh and MQTT 5. The transport can be
selected via command line arguments.
 */

use std::{
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use backon::{BackoffBuilder, ExponentialBuilder, Retryable};
use bytes::Buf;
use clap::{Parser, command};
use log::{debug, error, info};
use serde_json::{Value, json};
use up_rust::{
    LocalUriProvider, StaticUriProvider, UAttributes, UCode, UPayloadFormat, UPriority, UUri,
    communication::{
        CallOptions, InMemoryRpcServer, Publisher, RequestHandler, RpcServer,
        ServiceInvocationError, SimplePublisher, UPayload,
    },
};
use up_transport_mqtt5::{Mqtt5TransportOptions, MqttClientOptions};
use up_transport_zenoh::UPTransportZenoh;

const MAX_TARGET_SPEED_KMH: f32 = 180.0;
const RESOURCE_ID_SET_TARGET_SPEED: u16 = 0x0001;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// The uProtocol topic to publish status messages to.
    /// The topic must be a valid uProtocol URI with a resource ID in the [0x8000, 0xFFFE] range.
    /// The topic's authority is used as the local uEntity authority.
    #[arg(value_name = "URI", env = "TOPIC", default_value = "up://cruise-control.app/C110/1/8000", value_parser = UUri::from_str)]
    topic: UUri,

    #[command(subcommand)]
    transport: Transports,
}

#[derive(clap::Subcommand)]
enum Transports {
    /// Use MQTT 5 as transport
    Mqtt5 {
        #[command(flatten)]
        options: MqttClientOptions,
    },
    /// Use Zenoh as transport
    Zenoh,
}

async fn get_transport(
    cli: Cli,
) -> Result<Arc<dyn up_rust::UTransport>, Box<dyn std::error::Error>> {
    let authority = cli.topic.authority_name();
    match cli.transport {
        Transports::Zenoh => {
            info!("Using Zenoh transport with default configuration");
            let transport = UPTransportZenoh::builder(authority)?
                .build()
                .await
                .map(Arc::new)?;
            Ok(transport)
        }
        Transports::Mqtt5 { options } => {
            info!(
                "Using MQTT 5 transport [broker URI: {}]",
                options.broker_uri
            );
            let transport_options = Mqtt5TransportOptions {
                mqtt_client_options: options,
                mode: up_transport_mqtt5::TransportMode::InVehicle,
                ..Default::default()
            };
            let transport = up_transport_mqtt5::Mqtt5Transport::new(transport_options, authority)
                .await
                .map(Arc::new)?;
            (|| transport.connect())
                .retry(
                    ExponentialBuilder::default()
                        .with_max_delay(Duration::from_secs(8))
                        .build(),
                )
                .notify(|error, sleep_duration| {
                    error!("{}, retrying in {sleep_duration:?}", error.get_message());
                })
                .when(|err| {
                    // no need to keep retrying if authentication or permission is denied
                    err.get_code() != UCode::UNAUTHENTICATED
                        && err.get_code() != UCode::PERMISSION_DENIED
                })
                .await?;
            info!("Connected to MQTT5 broker");
            Ok(transport)
        }
    }
}

struct OperationalState {
    rng: fastrand::Rng,
    target_speed: u8,
    current_speed: u8,
    engine_temp: f32,
}

impl Default for OperationalState {
    fn default() -> Self {
        let rng = fastrand::Rng::new();
        Self {
            rng,
            target_speed: 100u8,
            current_speed: 90,
            engine_temp: 70.0,
        }
    }
}
impl OperationalState {
    fn update_state(&mut self) {
        // simulate some random fluctuations in values
        // even though they would not change that fast in a real vehicle
        // but this makes the example more interesting
        let lower_speed_bound = self.current_speed.saturating_sub(5).min(self.target_speed);
        let upper_speed_bound = self.current_speed.saturating_add(5).min(self.target_speed);
        self.current_speed = self.rng.u8(lower_speed_bound..=upper_speed_bound);
        let engine_temp_variation: f32 = self.rng.f32() * 4.0 - 2.0; // +/- 2 degrees
        self.engine_temp = (self.engine_temp + engine_temp_variation).clamp(-20.0, 150.0);
    }

    fn get_status(&self) -> Value {
        json!({
            "AmbientTemperature": 22,
            "Battery": 80,
            "CruiseControl": false,
            "Economy": "Normal",
            "Engine Temperature": self.engine_temp,
            "Gear": "P",
            "RPM": 0.0,
            "Range": 320,
            "ShareLocation": false,
            "Speed": self.current_speed,
            "SpeedUnit": "km/h",
            "TemperatureUnit": 0,
            "TypeOfVehicle": 0
        })
    }
}

// The handler for incoming requests to set the target speed.
struct SetTargetSpeed(Arc<RwLock<OperationalState>>);

#[async_trait::async_trait]
impl RequestHandler for SetTargetSpeed {
    async fn handle_request(
        &self,
        _resource_id: u16,
        message_attributes: &UAttributes,
        request_payload: Option<UPayload>,
    ) -> Result<Option<UPayload>, ServiceInvocationError> {
        info!(
            "Handling SetTargetSpeed request [source: {}]",
            message_attributes.source.as_ref().unwrap()
        );
        let Some(payload) = request_payload else {
            error!("Received empty payload for SetTargetSpeed request");
            return Err(ServiceInvocationError::InvalidArgument(
                "Payload cannot be empty".to_string(),
            ));
        };

        let target_speed = payload.payload().try_get_f32().map_err(|e| {
            error!("Failed to parse payload: {}", e);
            ServiceInvocationError::InvalidArgument("Invalid payload format".to_string())
        })?;
        if target_speed < 0.0 || target_speed > MAX_TARGET_SPEED_KMH {
            error!("Received out-of-range speed: {}", target_speed);
            return Err(ServiceInvocationError::InvalidArgument(format!(
                "Speed must be between 0 and {}",
                MAX_TARGET_SPEED_KMH
            )));
        }
        let mut operational_state = self.0.write().unwrap();
        // we have already checked the range above,
        // so casting to u8 is safe here
        operational_state.target_speed = target_speed as u8;
        info!(
            "Set target speed to {} km/h",
            operational_state.target_speed
        );
        // no response payload needed
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    env_logger::init();
    let command = Cli::parse();
    let status_topic_resource_id = command.topic.resource_id();
    let status_topic = command.topic.to_uri(true);
    let uri_provider = Arc::new(StaticUriProvider::try_from(&command.topic)?);

    let transport = get_transport(command).await?;
    let publisher = SimplePublisher::new(transport.clone(), uri_provider.clone());
    let operational_state = Arc::new(RwLock::new(OperationalState::default()));
    let request_handler = Arc::new(SetTargetSpeed(operational_state.clone()));
    let rpc_server = InMemoryRpcServer::new(transport.clone(), uri_provider.clone());
    rpc_server
        .register_endpoint(
            None,
            RESOURCE_ID_SET_TARGET_SPEED,
            request_handler,
        )
        .await?;

    info!(
        "cruise control service is running [setTargetSpeed endpoint: {}, status topic: {status_topic}]",
        uri_provider.get_resource_uri(RESOURCE_ID_SET_TARGET_SPEED).to_uri(true),
    );

    loop {
        // once in a while, update the current status
        let current_status = if fastrand::bool() {
            let mut state = operational_state.write().unwrap();
            state.update_state();
            state.get_status()
        } else {
            debug!("Skipping state update this cycle");
            operational_state.read().unwrap().get_status()
        };
        let current_status_str = serde_json::to_string_pretty(&current_status).unwrap();
        let payload = UPayload::new(
            serde_json::to_vec(&current_status).unwrap(),
            UPayloadFormat::UPAYLOAD_FORMAT_JSON,
        );

        if let Err(e) = publisher
            .publish(
                status_topic_resource_id,
                CallOptions::for_publish(Some(1000), None, Some(UPriority::UPRIORITY_CS1)),
                Some(payload),
            )
            .await
        {
            error!(
                "Failed to publish status message [topic: {}]: {}",
                status_topic, e
            );
        } else {
            debug!(
                "Successfully published status message [topic: {}]: {}",
                status_topic, current_status_str
            );
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_operational_state_update() {
        let mut state = OperationalState::default();
        state.target_speed = 120;
        state.current_speed = 100;
        state.engine_temp = 75.0;

        state.update_state();

        // After update, current speed should be within 5 km/h of previous speed and not exceed target speed
        assert!(state.current_speed >= 95 && state.current_speed <= 105 && state.current_speed <= state.target_speed);
        // Engine temperature should be within -20 to 150 degrees Celsius
        assert!(state.engine_temp >= -20.0 && state.engine_temp <= 150.0);

        // when setting target speed lower than current speed
        state.target_speed = 80;
        // and adjusting values a few times
        for _ in 0..10 {
            state.update_state();
            // then the adjusted speed must not exceed the new target speed
            assert!(
                state.current_speed <= 80,
                "Current speed {} exceeds target speed {}", state.current_speed, state.target_speed);
        }
    }
}
