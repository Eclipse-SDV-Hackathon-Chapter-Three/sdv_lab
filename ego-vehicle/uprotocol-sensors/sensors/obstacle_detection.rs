use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::ObstacleDetectionEvent;

/// Typed view over a CARLA Sensor that emits `ColisionEvent`.
pub struct ObstacleDetection<'a>(pub &'a CarlaSensor);

impl<'a> Listen for ObstacleDetection<'a> {
    type Data = ObstacleDetectionEvent;

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
                log::warn!("Received non ObstacleDetectionEvent");
            }
        });
    }
}

pub struct ObstacleDetectionFactory;

impl ViewFactory for ObstacleDetectionFactory {
    type View<'a> = ObstacleDetection<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        ObstacleDetection(s)
    }
}
