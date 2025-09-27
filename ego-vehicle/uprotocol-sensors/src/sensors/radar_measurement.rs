use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::RadarMeasurement as RadarMeasurementEvent;

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
