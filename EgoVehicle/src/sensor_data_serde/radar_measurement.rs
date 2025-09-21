use carla::sensor::data::{
    RadarDetection as CarlaRadarDetection, RadarMeasurement as RadarMeasurementEvent,
};
use serde::{Deserialize, Serialize};

/// Remote schema for the foreign element type
#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::sensor::data::RadarDetection")]
pub struct RadarDetectionRemote {
    pub velocity: f32,
    pub azimuth: f32,
    pub altitude: f32,
    pub depth: f32,
}

// -------------------- &[RadarDetection] (serialize-only) --------------------
mod slice_radar_detection_remote {
    use super::*;
    use serde::ser::{SerializeSeq, Serializer};

    struct AsRemote<'a>(&'a CarlaRadarDetection);
    impl<'a> Serialize for AsRemote<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            super::RadarDetectionRemote::serialize(self.0, s)
        }
    }

    pub fn serialize<S: Serializer>(
        slice: &[CarlaRadarDetection],
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
pub struct RadarMeasurementSerBorrowed<'a> {
    pub detection_amount: usize,
    #[serde(with = "self::slice_radar_detection_remote")]
    pub detections: &'a [CarlaRadarDetection],
    pub len: usize,
    pub is_empty: bool,
}

impl<'a> From<&'a RadarMeasurementEvent> for RadarMeasurementSerBorrowed<'a> {
    fn from(m: &'a RadarMeasurementEvent) -> Self {
        Self {
            detection_amount: m.detection_amount(),
            detections: m.as_slice(),
            len: m.len(),
            is_empty: m.is_empty(),
        }
    }
}

// -------------------- Vec<RadarDetection> (round-trip) --------------------
mod vec_radar_detection_remote {
    use super::*;
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    struct AsRemote<'a>(&'a CarlaRadarDetection);
    impl<'a> Serialize for AsRemote<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            super::RadarDetectionRemote::serialize(self.0, s)
        }
    }

    struct FromRemote(CarlaRadarDetection);
    impl<'de> Deserialize<'de> for FromRemote {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            super::RadarDetectionRemote::deserialize(d).map(FromRemote)
        }
    }

    pub fn serialize<S: Serializer>(v: &Vec<CarlaRadarDetection>, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for d in v {
            seq.serialize_element(&AsRemote(d))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<CarlaRadarDetection>, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<CarlaRadarDetection>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<RadarDetection>")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(FromRemote(x)) = seq.next_element::<FromRemote>()? {
                    out.push(x); // <-- x is CarlaRadarDetection; no `.0`
                }
                Ok(out)
            }
        }
        d.deserialize_seq(V)
    }
}

#[derive(Serialize, Deserialize)]
pub struct RadarMeasurementSerDe {
    pub detection_amount: usize,
    #[serde(with = "self::vec_radar_detection_remote")]
    pub detections: Vec<CarlaRadarDetection>,
    pub len: usize,
    pub is_empty: bool,
}
