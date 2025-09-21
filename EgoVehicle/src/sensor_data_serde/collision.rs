use crate::sensor_data_serde::{ActorSerDe, Vector3DSerDe};
use carla::sensor::data::CollisionEvent;
use serde::{Deserialize, Serialize};

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
