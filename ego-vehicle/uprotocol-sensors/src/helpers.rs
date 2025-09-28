use crate::sensors::{Listen, SensorComms};
use carla::client::{ActorBase, Sensor, World};
use log;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use up_rust::{UMessageBuilder, UPayloadFormat, UTransport, UUri};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Polls until an actor with `role_name` appears, or `running` flips false.
pub async fn wait_for_actor_id_by_role(
    carla_world: &World,
    running: &AtomicBool,
    role_name: &str,
    polling_ms: u64,
) -> Result<u32> {
    let poll = Duration::from_millis(polling_ms);
    let mut found: Option<u32> = None;

    while running.load(Ordering::SeqCst) && found.is_none() {
        log::info!("Waiting for actor with role_name='{role_name}'...");
        let _ = carla_world.wait_for_tick();

        for actor in carla_world.actors().iter() {
            for attr in actor.attributes().iter() {
                if attr.id() == "role_name" && attr.value_string() == role_name {
                    log::info!("Found actor id={} with role_name='{role_name}'", actor.id());
                    found = Some(actor.id());
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }

        if found.is_none() {
            sleep(poll).await; // async, non-blocking
        }
    }

    found.ok_or_else(|| "Stopped before target actor appeared".into())
}

/// A factory that can build a typed view borrowing the `Sensor`.
pub trait ViewFactory {
    type View<'a>: Listen + Send + Sync + 'a
    where
        Self: 'a;
    fn make<'a>(&self, sensor: &'a Sensor) -> Self::View<'a>;
}

/// Generic setup: builds the typed view with a factory, encodes events, builds a UMessage,
/// and sends via the provided `Arc<dyn UTransport>`.
///
/// Returns `(comms, actor_id, sensor_keepalive)`.
pub async fn setup_sensor_with_transport<F, Encode>(
    carla_world: &World,
    running: &AtomicBool,
    role_name: &str,
    comms_name: &str,
    polling_ms: u64,
    factory: F,
    uuri: UUri,
    encode: Encode,
    payload_format: UPayloadFormat,
    transport: Arc<dyn UTransport>,
) -> Result<(SensorComms, u32, Sensor)>
where
    F: ViewFactory,
    Encode: for<'a> Fn(<F::View<'a> as Listen>::Data) -> Result<Vec<u8>> + Send + Sync + 'static,
{
    // 1) Wait for the actor
    let actor_id = wait_for_actor_id_by_role(carla_world, running, role_name, polling_ms).await?;

    // 2) Fetch & convert to Sensor
    let sensor_actor = carla_world
        .actor(actor_id)
        .ok_or_else(|| format!("Unable to locate sensor actor id: {actor_id}"))?;
    let sensor = sensor_actor
        .into_kinds()
        .try_into_sensor()
        .map_err(|_| "Unable to turn actor into a sensor")?;

    // 3) Create comms (keep alive in caller)
    let comms = SensorComms::new(comms_name);

    // 4) Capture stack for async handler
    let uuri_shared = uuri.clone();
    let encode = Arc::new(encode);
    let transport = Arc::clone(&transport);
    let role_name: String = role_name.to_owned();

    // 5) Build typed view borrowing `sensor` (no clone!)
    {
        let view = factory.make(&sensor);

        // 6) Attach: encode -> UMessageBuilder -> transport.send
        comms.listen_on_async(&view, move |evt| {
            let uuri = uuri_shared.clone();
            let encode = Arc::clone(&encode);
            let transport = Arc::clone(&transport);
            let role_name = role_name.clone();

            async move {
                let payload = match encode(evt) {
                    Ok(b) => b,
                    Err(e) => {
                        log::error!("Event encoding failed for {role_name}: {e}");
                        return;
                    }
                };

                let umsg = match UMessageBuilder::publish(uuri)
                    .build_with_payload(payload, payload_format)
                {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("UMessage build failed for {role_name}: {e}");
                        return;
                    }
                };

                if let Err(err) = transport.send(umsg).await {
                    log::error!("Transport send failed for {role_name}: {:?}", err);
                } else {
                    log::info!("Transport send succeeded for {role_name}.");
                }
            }
        });
        // `view` (and its borrow of `sensor`) ends here
    }

    // 7) Return handles to keep things alive
    Ok((comms, actor_id, sensor))
}
