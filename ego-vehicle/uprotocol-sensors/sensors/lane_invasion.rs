use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::LaneInvasionEvent;

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

pub struct LaneInvasionFactory;

impl ViewFactory for LaneInvasionFactory {
    type View<'a> = LaneInvasion<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        LaneInvasion(s)
    }
}
