use crate::helpers::ViewFactory;
use crate::sensors::Listen;
use carla::client::Sensor as CarlaSensor;
use carla::sensor::SensorData;
use carla::sensor::data::Image as ImageEvent;

/// Typed view over a CARLA Sensor that emits `ImageEvent`.
pub struct Image<'a>(pub &'a CarlaSensor);

impl<'a> Listen for Image<'a> {
    type Data = ImageEvent;

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
                log::warn!("Received non ImageEvent");
            }
        });
    }
}

pub struct ImageFactory;

impl ViewFactory for ImageFactory {
    type View<'a> = Image<'a>;
    fn make<'a>(&self, s: &'a CarlaSensor) -> Self::View<'a> {
        Image(s)
    }
}
