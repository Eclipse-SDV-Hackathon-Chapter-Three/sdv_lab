pub struct PIDController {
    kp: f64,
    ki: f64,
    kd: f64,
    velocity_error: f64,
    previous_error: f64,
    accumulated_error: f64,
    previous_time: f64,
}

impl PIDController {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        PIDController {
            kp,
            ki,
            kd,
            velocity_error: 0.0,
            previous_error: 0.0,
            accumulated_error: 0.0,
            previous_time: 0.0,
        }
    }

    pub fn compute(&mut self, desired_velocity: f64, current_velocity: f64, current_time: f64) -> Result<f64, String> {
        if self.previous_time == 0.0 {
            self.previous_time = current_time;
            return Ok(0.0);
        }

        let delta_time = current_time - self.previous_time;
        self.previous_time = current_time;

        if delta_time <= 0.0 {
            return Err("delta_time must be positive and higher than 0.".to_string());
        }

        self.previous_error = self.velocity_error;
        self.velocity_error = desired_velocity - current_velocity;
        self.accumulated_error += self.velocity_error * delta_time;
        let derivative_error = (self.velocity_error - self.previous_error) / delta_time;

        let acceleration = (self.kp * self.velocity_error)
            + (self.ki * self.accumulated_error)
            + (self.kd * derivative_error);

        Ok(acceleration)
    }

    pub fn reset(&mut self) {
        self.velocity_error = 0.0;
        self.previous_error = 0.0;
        self.accumulated_error = 0.0;
        self.previous_time = 0.0;
    }
}