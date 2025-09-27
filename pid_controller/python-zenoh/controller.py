# PID Logic
class PIDController:
    """
    PIDController implements a Proportional-Integral-Derivative controller to compute
    the control output (acceleration) based on error terms.
    """

    def __init__(self, kp, ki, kd):
        """
        Initialize the PID controller with specified gain values.

        Args:
            kp (float): Proportional gain
            ki (float): Integral gain
            kd (float): Derivative gain
        """
        self.kp = kp
        self.ki = ki
        self.kd = kd

        self.velocity_error = 0.0
        self.previous_error = 0.0
        self.accumulated_error = 0.0
        self.previous_time = 0.0


    def compute(self, desired_velocity, current_velocity, current_time):
        """
        Compute the acceleration using PID logic.

        Args:
            desired_velocity (float): Target velocity
            current_velocity (float): Current measured velocity
            current_time (float): Current time in seconds

        Returns:
            acceleration (float): Computed acceleration value

        Raises:
            ValueError: If delta_time is zero or negative
        """
        if self.previous_time == 0.0:
            self.previous_time = current_time
            return 0.0

        delta_time = current_time - self.previous_time
        self.previous_time = current_time

        if delta_time <= 0.0:
            raise ValueError("delta_time must be positive and higher than 0.")

        # PID Calculations
        self.previous_error = self.velocity_error
        self.velocity_error = desired_velocity - current_velocity
        self.accumulated_error = self.accumulated_error + (self.velocity_error * delta_time)
        self.derivative_error = (self.velocity_error - self.previous_error) / delta_time
        acceleration = (
            (self.kp * self.velocity_error) +
            (self.ki * self.accumulated_error) +
            (self.kd * self.derivative_error)
        )

        return acceleration

    # ------------------------------------------------------------------
    # Reset internal state so the controller restarts after 
    # enable/disable transition.  Call this from ZenohHandler when an
    # "deactivate" command is received.
    # ------------------------------------------------------------------
    def reset(self):
        """Clear internal PID error accumulators and timers."""
        self.velocity_error    = 0.0
        self.previous_error    = 0.0
        self.accumulated_error = 0.0
        self.previous_time     = 0.0
