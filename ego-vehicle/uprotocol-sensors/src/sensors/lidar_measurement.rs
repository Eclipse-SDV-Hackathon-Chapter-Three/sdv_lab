use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::LidarMeasurement as LidarMeasurementEvent;

/// Typed view over a CARLA Sensor that emits `LidarMeasurementEvent`.
pub struct LidarMeasurement<'a>(pub &'a CarlaSensor);

impl<'a> Listen for LidarMeasurement<'a> {
    type Data = LidarMeasurementEvent;

    fn listen<F>(&self, f: F)
    where
        F: FnMut(Self::Data) + Send + 'static,
    {
        let mut f = f;
        self.0.listen(move |data: SensorData| {
            if let Ok(evt) = data.try_into() {
                f(evt);
            } else {
                log::warn!("Received non LidarMeasurementEvent");
            }
        });
    }
}

pub struct LidarMeasurementFactory;

impl ViewFactory for LidarMeasurementFactory {
    type View<'a> = LidarMeasurement<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        LidarMeasurement(s)
    }
}
