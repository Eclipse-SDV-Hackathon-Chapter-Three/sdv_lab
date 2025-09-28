use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::ImuMeasurement as ImuMeasurementEvent;

/// Typed view over a CARLA Sensor that emits `ImuMeasurement`.
pub struct ImuMeasurement<'a>(pub &'a CarlaSensor);

impl<'a> Listen for ImuMeasurement<'a> {
    type Data = ImuMeasurementEvent;

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
                log::warn!("Received non ImuMeasurement");
            }
        });
    }
}

pub struct ImuMeasurementFactory;

impl ViewFactory for ImuMeasurementFactory {
    type View<'a> = ImuMeasurement<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        ImuMeasurement(s)
    }
}
