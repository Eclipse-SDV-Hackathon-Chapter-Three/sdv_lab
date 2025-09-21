use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::{
    RadarDetection as CarlaRadarDetection,
    RadarMeasurement as RadarMeasurementEvent,
};
use serde::{Deserialize, Serialize};

/// Typed view over a CARLA Sensor that emits `RadarMeasurementEvent`.
pub struct RadarMeasurement<'a>(pub &'a CarlaSensor);

impl<'a> Listen for RadarMeasurement<'a> {
    type Data = RadarMeasurementEvent;

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
                log::warn!("Received non RadarMeasurementEvent");
            }
        });
    }
}

pub struct RadarMeasurementFactory;

impl ViewFactory for RadarMeasurementFactory {
    type View<'a> = RadarMeasurement<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        RadarMeasurement(s)
    }
}


/// Remote schema for the foreign element type
#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::sensor::data::RadarDetection")]
pub struct RadarDetectionRemote {
    pub velocity: f32,
    pub azimuth:  f32,
    pub altitude: f32,
    pub depth:    f32,
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

    pub fn serialize<S: Serializer>(slice: &[CarlaRadarDetection], s: S) -> Result<S::Ok, S::Error> {
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
    use serde::{Serializer, Deserializer};
    use serde::ser::SerializeSeq;
    use serde::de::{SeqAccess, Visitor};
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

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<CarlaRadarDetection>, D::Error> {
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
