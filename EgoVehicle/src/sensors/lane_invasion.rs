use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData; // enum that Sensor::listen emits
use carla::sensor::data::LaneInvasionEvent;
use serde::{Serialize, Deserialize};

/// Typed view over a CARLA Sensor that emits `LaneInvasionEvent`.
pub struct LaneInvasion<'a>(pub &'a CarlaSensor);

impl<'a> Listen for LaneInvasion<'a> {
    type Data = LaneInvasionEvent;

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
                log::warn!("Received non LaneInvasionEvent");
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::road::element::LaneMarking_Type")]
pub enum LaneMarkingTypeSerDe {
    Other = 0,
    Broken = 1,
    Solid = 2,
    SolidSolid = 3,
    SolidBroken = 4,
    BrokenSolid = 5,
    BrokenBroken = 6,
    BottsDots = 7,
    Grass = 8,
    Curb = 9,
    None = 10,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::road::element::LaneMarking_Color")]
pub enum LaneMarkingColorSerDe {
    Standard = 0,
    Blue = 1,
    Green = 2,
    Red = 3,
    Yellow = 4,
    Other = 5,
}


#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::road::element::LaneMarking_LaneChange")]
pub enum LaneMarkingLaneChangeSerDe {
    None = 0,
    Right = 1,
    Left = 2,
    Both = 3,
}

#[derive(Serialize, Deserialize)]
pub struct LaneMarkingSerDe {
    #[serde(with = "LaneMarkingTypeSerDe")]
    pub marking_type: carla::road::element::LaneMarking_Type,

    #[serde(with = "LaneMarkingColorSerDe")]
    pub marking_color: carla::road::element::LaneMarking_Color,

    #[serde(with = "LaneMarkingLaneChangeSerDe")]
    pub lane_change: carla::road::element::LaneMarking_LaneChange,

    pub width: f64,
}

#[derive(Serialize, Deserialize)]
pub struct LaneInvasionEventSerDe {
    pub crossed_lane_markings: Vec<LaneMarkingSerDe>
}

impl From<LaneInvasionEvent> for LaneInvasionEventSerDe {
    fn from(value: LaneInvasionEvent) -> Self {
        let mut crossed_lane_markings: Vec<LaneMarkingSerDe> = Vec::new();
        for clm in value.crossed_lane_markings() {
            let lane_marking_serde = LaneMarkingSerDe {
                marking_type: clm.type_(),
                marking_color: clm.color(),
                lane_change: clm.lane_change(),
                width: clm.width()
            };
            crossed_lane_markings.push(lane_marking_serde);
        }

        LaneInvasionEventSerDe {
            crossed_lane_markings
        }
    }
}
