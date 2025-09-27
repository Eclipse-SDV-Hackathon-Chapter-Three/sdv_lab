# Zenoh Pub/Sub
import time

# Logging
import datetime

# Show Results
import numpy as np
import matplotlib.pyplot as plt


class ZenohHandler:
    """
    Handles Zenoh Pub/Sub communication for the PID controller.
    Subscribes to velocity topics and publishes acceleration.
    """
    def __init__(self, controller, session, sub_current, sub_desired, pub_acc, sub_stamp=None, sub_enable="control/pid/enable"):
        """
        Initialize the Zenoh handler.

        Args:
            controller (PIDController): PID controller instance
            session (zenoh.Session): Zenoh session object
            sub_current (str): Topic for current velocity
            sub_desired (str): Topic for desired velocity
            pub_acc (str): Topic to publish acceleration values to
            sub_stamp (str, optional): Topic to subscribe for clock/timestamp status
            sub_enable (str): Topic to subscribe for enable/disable commands
            pub_status (str): Topic to publish PID status
        """
        self.controller = controller
        self.session = session
        self.sub_current = sub_current
        self.sub_desired = sub_desired
        self.sub_stamp = sub_stamp
        self.sub_enable = sub_enable

        # --------------------------------------------------------------
        # Runtime state flag – starts *active* by default.
        # --------------------------------------------------------------
        self.pid_active = False

        # Publisher for broadcasting current PID output
        self.pub_acc = self.session.declare_publisher(pub_acc)

        self.current_velocity = 0.0
        self.desired_velocity = 0.0

        self.current_time = 0.0
        self.previous_time = 0.0

        self.results = {
            'desired_velocity': [],
            'current_velocity': [],
            'current_time': [],
            'acceleration': []
        }

    def stamp_listener(self, sample):
        try:
            value = float(sample.payload.to_string())
            print(f"Received current clock '{value}'")
            self.current_time = value
        except Exception as e:
            print(f"[ERROR] Timestamp processing failed: {e}")

    def current_listener(self, sample):
        """
        Listener for current velocity topics.

        Args:
            sample: Zenoh sample object
        """
        try:
            value = float(sample.payload.to_string())
            print(f"Received current velocity '{value}'")
            self.current_velocity = value
            self.publish_acc()
        except Exception as e:
            print(f"[ERROR] Current velocity processing failed: {e}")


    def desired_listener(self, sample):
        """
        Listener for desired velocity topics.

        Args:
            sample: Zenoh sample object
        """
        try:
            value = float(sample.payload.to_string())
            print(f"Received desired velocity '{value}'")
            self.desired_velocity = value
        except Exception as e:
            print(f"[ERROR] Desired velocity processing failed: {e}")


    def publish_acc(self):
        """
        Compute and publish acceleration based on the current and desired velocities.
        """
        current_time = time.perf_counter()
        # Skip the control law entirely if the PID is disabled ----------------
        if not self.pid_active:
            return

        acceleration = self.controller.compute(
            self.desired_velocity,
            self.current_velocity,
            self.current_time
        )

        # Publish computed acceleration to the designated topic
        print(f"Publishing Acceleration: {str(acceleration)}")
        self.pub_acc.put(str(acceleration))

        # Store results for later analysis
        self.results['desired_velocity'].append(self.desired_velocity)
        self.results['current_velocity'].append(self.current_velocity)
        self.results['current_time'].append(self.current_time)
        self.results['acceleration'].append(acceleration)

        print(f"Delta time: {self.current_time - self.previous_time} seconds")
        self.previous_time = self.current_time

    # ------------------------------------------------------------------
    # Enable / disable handling
    # ------------------------------------------------------------------
    def enable_listener(self, sample):
        """Parse the boolean enable flag and switch PID state accordingly."""
        try:
            payload = sample.payload.to_string().strip().lower()
            if payload in ("true", "1", "on"):
                enable = True
            elif payload in ("false", "0", "off"):
                enable = False
            else:
                raise ValueError("Malformed enable payload")
        except Exception as e:
            print(f"[ERROR] Enable/disable processing failed: {e}")
            return

        # Transition only when state changes --------------------------------
        if enable and not self.pid_active:
            self._activate_pid()
        elif not enable and self.pid_active:
            self._deactivate_pid()

    def _activate_pid(self):
        self.pid_active = True
        self.controller.reset()
        timestamp = datetime.datetime.now().isoformat()
        print(f"[INFO] PID controller ACTIVATED at {timestamp}")

    def _deactivate_pid(self):
        self.pid_active = False
        self.controller.reset()
        timestamp = datetime.datetime.now().isoformat()
        print(f"[INFO] PID controller DEACTIVATED at {timestamp}")


    def start(self):
        """
        Start subscribing to Zenoh topics for current and desired velocity.
        """
        self.session.declare_subscriber(
            self.sub_stamp,
            self.stamp_listener
        )
        print("Current timestamp subscriber started, waiting for data...")

        self.session.declare_subscriber(
            self.sub_current,
            self.current_listener
        )
        print("Current velocity subscriber started, waiting for data...")

        if self.sub_enable:
            self.session.declare_subscriber(
                self.sub_enable,
                self.enable_listener,
            )
            print("Enable/Disable subscriber started, waiting for data...")

        self.session.declare_subscriber(
            self.sub_desired,
            self.desired_listener
        )
        print("Desired velocity subscriber started, waiting for data...")


    def store_results(self):
        with open("desired_velocity.log", 'w') as desired_velocity:
            for dv in self.results['desired_velocity']:
                desired_velocity.write(str(dv) + '\n')

        with open("current_velocity.log", 'w') as current_velocity:
            for cv in self.results['current_velocity']:
                current_velocity.write(str(cv) + '\n')

        with open("current_time.log", 'w') as current_time:
            for ct in self.results['current_time']:
                current_time.write(str(ct) + '\n')

        with open("acceleration.log", 'w') as acceleration:
            for acc in self.results['acceleration']:
                acceleration.write(str(acc) + '\n')

    def show_results(self):
        """
        Plot the results of the PID controller computations.
        """
        desired_velocity = np.array(self.results['desired_velocity'])
        current_velocity = np.array(self.results['current_velocity'])
        current_time = np.array(self.results['current_time'])
        acceleration = np.array(self.results['acceleration'])

        # Align samples
        samples = np.amin(
            np.array([
                desired_velocity.shape[0],
                current_velocity.shape[0],
                current_time.shape[0],
                acceleration.shape[0]
            ])
        )

        desired_velocity = np.reshape(desired_velocity, samples)
        current_velocity = np.reshape(current_velocity, samples)
        current_time = np.reshape(current_time, samples)
        acceleration = np.reshape(acceleration, samples)

        # Creating Plots
        fig, vel = plt.subplots()

        vel.set_title("Target (g) vs Velocity (b) + Acceleration (o)")
        vel.set_xlabel("Time")
        vel.set_ylabel("Kph")

        vel.set_xlim((np.amin(current_time) - 2.5), np.amax(current_time) + 2.5)
        vel.set_ylim((np.amin(current_velocity) - 2.5), np.amax(current_velocity) + 2.5)

        vel.plot(current_time, desired_velocity, 'tab:green')
        vel.plot(current_time, current_velocity, 'tab:blue')
        vel.plot(current_time, acceleration, 'tab:orange')

        vel.grid()

        plt.savefig('results.png')
        plt.close()
