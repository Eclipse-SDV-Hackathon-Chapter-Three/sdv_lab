use carla::geom::Location as CarlaLocation;
use carla::sensor::data::{
    LidarDetection as CarlaLidarDetection, LidarMeasurement as LidarMeasurementEvent,
};
use serde::{Deserialize, Serialize};

/// Remote schema for nested foreign type `Location` (x, y, z)
#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::geom::Location")]
pub struct LocationRemote {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// `with`-module for (de)serializing `CarlaLocation` by delegating to `LocationRemote`.
mod location_with {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(loc: &CarlaLocation, s: S) -> Result<S::Ok, S::Error> {
        super::LocationRemote::serialize(loc, s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<CarlaLocation, D::Error> {
        super::LocationRemote::deserialize(d)
    }
}

/// Remote schema for the foreign element type `LidarDetection`
/// (nested `point` uses the `location_with` module)
#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::sensor::data::LidarDetection")]
pub struct LidarDetectionRemote {
    #[serde(with = "self::location_with")]
    pub point: CarlaLocation,
    pub intensity: f32,
}

// -------------------- &[LidarDetection] (serialize-only) --------------------
mod slice_lidar_detection_remote {
    use super::*;
    use serde::Serialize;
    use serde::ser::{SerializeSeq, Serializer};

    struct AsRemote<'a>(&'a CarlaLidarDetection);
    impl<'a> Serialize for AsRemote<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            super::LidarDetectionRemote::serialize(self.0, s)
        }
    }

    pub fn serialize<S: Serializer>(
        slice: &[CarlaLidarDetection],
        s: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(slice.len()))?;
        for d in slice {
            seq.serialize_element(&AsRemote(d))?;
        }
        seq.end()
    }
}

/// Borrowed, zero-copy serializer
#[derive(Serialize)]
pub struct LidarMeasurementSerBorrowed<'a> {
    pub horizontal_angle: f32,
    pub channel_count: usize,
    pub len: usize,
    pub is_empty: bool,
    #[serde(with = "self::slice_lidar_detection_remote")]
    pub detections: &'a [CarlaLidarDetection],
}

impl<'a> From<&'a LidarMeasurementEvent> for LidarMeasurementSerBorrowed<'a> {
    fn from(m: &'a LidarMeasurementEvent) -> Self {
        Self {
            horizontal_angle: m.horizontal_angle(),
            channel_count: m.channel_count(),
            len: m.len(),
            is_empty: m.is_empty(),
            detections: m.as_slice(), // borrow – zero alloc/copy
        }
    }
}

// -------------------- Vec<LidarDetection> (round-trip) --------------------
mod vec_lidar_detection_remote {
    use super::*;
    use serde::Serialize;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    struct AsRemote<'a>(&'a CarlaLidarDetection);
    impl<'a> Serialize for AsRemote<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            super::LidarDetectionRemote::serialize(self.0, s)
        }
    }

    struct FromRemote(CarlaLidarDetection);
    impl<'de> Deserialize<'de> for FromRemote {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            super::LidarDetectionRemote::deserialize(d).map(FromRemote)
        }
    }

    pub fn serialize<S: Serializer>(v: &Vec<CarlaLidarDetection>, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for d in v {
            seq.serialize_element(&AsRemote(d))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<CarlaLidarDetection>, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<CarlaLidarDetection>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<LidarDetection>")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(FromRemote(x)) = seq.next_element::<FromRemote>()? {
                    out.push(x); // already CarlaLidarDetection
                }
                Ok(out)
            }
        }
        d.deserialize_seq(V)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LidarMeasurementSerDe {
    pub horizontal_angle: f32,
    pub channel_count: usize,
    pub len: usize,
    pub is_empty: bool,
    #[serde(with = "self::vec_lidar_detection_remote")]
    pub detections: Vec<CarlaLidarDetection>,
}

impl From<LidarMeasurementEvent> for LidarMeasurementSerDe {
    fn from(m: LidarMeasurementEvent) -> Self {
        // If CarlaLidarDetection isn't Clone, rebuild from public fields (no FFI trait bounds):
        let detections: Vec<CarlaLidarDetection> = m
            .as_slice()
            .iter()
            .map(|d| CarlaLidarDetection {
                point: CarlaLocation {
                    x: d.point.x,
                    y: d.point.y,
                    z: d.point.z,
                },
                intensity: d.intensity,
            })
            .collect();

        Self {
            horizontal_angle: m.horizontal_angle(),
            channel_count: m.channel_count(),
            len: m.len(),
            is_empty: m.is_empty(),
            detections,
        }
    }
}
