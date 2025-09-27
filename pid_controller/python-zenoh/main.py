import zenoh, time
from controller import PIDController
from zenoh_handler import ZenohHandler

def main():
    """
    Main entry point for the PID control system.
    Sets up PID controller, Zenoh session and starts event loop.
    """
    # Fine-tuning
    Kp = 0.125
    Ki = Kp / 8
    Kd = Kp / 10
    print(f"PID => Kp={Kp}, Ki={Ki}, Kd={Kd}")

    # Create PID controller with tuning parameters
    pid = PIDController(kp=Kp, ki=Ki, kd=Kd)

    # Create Zenoh session
    config = zenoh.Config()
    session = zenoh.open(config)

    # Start Zenoh handler
    handler = ZenohHandler(
        controller=pid,
        session=session,
        sub_stamp='vehicle/status/clock_status',
        sub_current='vehicle/status/velocity_status',
        sub_desired='adas/cruise_control/target_speed',
		sub_enable='adas/cruise_control/engage',
        pub_acc='control/command/actuation_cmd'
    )

    handler.start()

    print("PID controller running (CTRL-C to terminate)...")

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("KeyboardInterrupt")
    finally:
        handler.store_results()
        handler.show_results()
        session.close()
        print("Done!")


if __name__ == "__main__":
    main()
