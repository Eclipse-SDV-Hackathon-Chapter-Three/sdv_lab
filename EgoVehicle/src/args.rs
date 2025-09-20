use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,
    #[clap(long, default_value_t = 2000)]
    pub port: u16,
    #[clap(long, default_value = "ego_vehicle")]
    pub ego_vehicle_role: String,
    #[clap(long, default_value = "ego_vehicle_sensor_lane_invasion")]
    pub ego_vehicle_sensor_lane_invasion_role: Option<String>,
    #[clap(long, default_value = "ego_vehicle_sensor_collision")]
    pub ego_vehicle_sensor_collision_role: Option<String>,
    #[clap(long, default_value_t = 0.100)]
    pub delta: f64,
    #[clap(long, default_value = None)]
    pub router: Option<String>,
}
