use crate::sensor_data_serde::Vector3DSerDe;
use carla::client::ActorBase;
use serde::{Deserialize, Serialize};
use nalgebra::{Isometry3, Translation3};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorSerDe {
    pub id: carla::rpc::ActorId,
    pub type_id: String,
    pub display_id: String,
    pub location: Translation3<f32>,
    pub transform: Isometry3<f32>,
    pub velocity: Vector3DSerDe,
    pub acceleration: Vector3DSerDe,
}

impl From<carla::client::Actor> for ActorSerDe {
    fn from(v: carla::client::Actor) -> Self {
        ActorSerDe {
            id: v.id(),
            type_id: v.type_id(),
            display_id: v.display_id(),
            location: v.location(),
            transform: v.transform(),
            velocity: v.velocity().into(),
            acceleration: v.acceleration().into(),
        }
    }
}
