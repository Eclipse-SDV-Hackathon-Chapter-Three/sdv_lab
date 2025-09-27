use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::CollisionEvent;

/// Typed view over a CARLA Sensor that emits `ColisionEvent`.
pub struct Collision<'a>(pub &'a CarlaSensor);

impl<'a> Listen for Collision<'a> {
    type Data = CollisionEvent;

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
                log::warn!("Received non CollisionEvent");
            }
        });
    }
}

pub struct CollisionFactory;

impl ViewFactory for CollisionFactory {
    type View<'a> = Collision<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        Collision(s)
    }
}
