use crate::sensors::Listen;
use crate::sensors::actor::{ActorSerDe, Vector3DSerDe};
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData; // enum that Sensor::listen emits
use carla::sensor::data::CollisionEvent;
use serde::{Deserialize, Serialize};

/// Typed view over a CARLA Sensor that emits `ColisionEvent`.
pub struct Collision<'a>(pub &'a CarlaSensor);

impl<'a> Listen for Collision<'a> {
    type Data = CollisionEvent;

    fn listen<F>(&self, f: F)
    where
        F: FnMut(Self::Data) + Send + 'static,
    {
        // CARLA expects FnMut(SensorData), so adapt here:
        let mut f = f;
        self.0.listen(move |data: SensorData| {
            if let Ok(evt) = data.try_into() {
                f(evt);
            } else {
                log::warn!("Received non CollisionEvent");
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct CollisionEventSerDe {
    pub actor: ActorSerDe,
    pub other_actor: Option<ActorSerDe>,
    pub normal_impulse: Vector3DSerDe,
}

impl From<CollisionEvent> for CollisionEventSerDe {
    fn from(value: CollisionEvent) -> Self {
        CollisionEventSerDe {
            actor: value.actor().into(),
            other_actor: value.other_actor().map(Into::into),
            normal_impulse: value.normal_impulse().into(),
        }
    }
}
