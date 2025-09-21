use crate::sensor_data_serde::actor::ActorSerDe;
use carla::sensor::data::ObstacleDetectionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ObstacleDetectionEventSerDe {
    pub actor: ActorSerDe,
    pub other_actor: ActorSerDe,
    pub distance: f32,
}

impl From<ObstacleDetectionEvent> for ObstacleDetectionEventSerDe {
    fn from(value: ObstacleDetectionEvent) -> Self {
        ObstacleDetectionEventSerDe {
            actor: value.actor().into(),
            other_actor: value.other_actor().into(),
            distance: value.distance().into(),
        }
    }
}
