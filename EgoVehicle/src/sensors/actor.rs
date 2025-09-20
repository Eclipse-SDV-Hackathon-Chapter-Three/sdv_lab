use carla::client::ActorBase;
use nalgebra::{Isometry3, Translation3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vector3DSerDe {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vector3<f32>> for Vector3DSerDe {
    fn from(v: Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<&Vector3<f32>> for Vector3DSerDe {
    fn from(v: &Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vector3DSerDe> for Vector3<f32> {
    fn from(v: Vector3DSerDe) -> Self {
        Vector3::new(v.x, v.y, v.z)
    }
}

impl From<&carla::geom::Vector3D> for Vector3DSerDe {
    fn from(v: &carla::geom::Vector3D) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vector3DSerDe> for carla::geom::Vector3D {
    fn from(v: Vector3DSerDe) -> Self {
        carla::geom::Vector3D {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

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
