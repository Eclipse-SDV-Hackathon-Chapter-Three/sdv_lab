#!/usr/bin/env python

#
#  Copyright (c) 2025 The X-Verse <https://github.com/The-Xverse>
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
#

# ==============================================================================
# -- Imports -------------------------------------------------------------------
# ==============================================================================

import zenoh
import numpy as np


# ==============================================================================
# -- CarlaUtils ----------------------------------------------------------------
# ==============================================================================

class CarlaUtils(object):
    # Constants
    MIN_THROTTLE    = 0.0
    MIN_STEER       =-1.0
    MIN_BRAKE       = 0.0
    MID_STEER       = 0.0
    MAX_THROTTLE    = 1.0
    MAX_STEER       = 1.0
    MAX_BRAKE       = 1.0

    def rad_to_steer(steer: float) -> float:
        return steer * (180.0 / 70.0 / np.pi)

    def clamp_throttle(throttle: float) -> float:
        return np.fmax(np.fmin(throttle, CarlaUtils.MAX_THROTTLE), CarlaUtils.MIN_THROTTLE)

    def clamp_steer(steer: float) -> float:
        return np.fmax(np.fmin(CarlaUtils.rad_to_steer(steer), CarlaUtils.MAX_STEER), CarlaUtils.MIN_STEER)

    def clamp_brake(brake: float) -> float:
        return np.fmax(np.fmin(brake, CarlaUtils.MAX_BRAKE), CarlaUtils.MIN_BRAKE)


# ==============================================================================
# -- ZenohVehicle --------------------------------------------------------------
# ==============================================================================

class ZenohVehicle(object):
    # Constants
    MID_ACTUATION = 0.0

    def __init__(self):
        # Session
        self._session = zenoh.open(zenoh.Config())

        # Subscribers
        self._actuation_subscriber = self._session.declare_subscriber('ego_vehicle/input/actuation', self._actuation_callback)

        # Publishers
        self._brake_publisher = self._session.declare_publisher('ego_vehicle/output/brake')
        self._speed_publisher = self._session.declare_publisher('ego_vehicle/output/speed')

        # Vehicle Control - PID
        self._actuation = self.MID_ACTUATION        # A scalar value to control the vehicle brake + throttle [-1.0, 1.0]. Default is 0.0.

        # Vehicle Control - CARLA
        self._throttle = CarlaUtils.MIN_THROTTLE    # A scalar value to control the vehicle throttle [0.0, 1.0]. Default is 0.0.
        self._steer = CarlaUtils.MID_STEER          # A scalar value to control the vehicle steering [-1.0, 1.0]. Default is 0.0.
        self._brake = CarlaUtils.MIN_BRAKE          # A scalar value to control the vehicle brake [0.0, 1.0]. Default is 0.0.
        self._hand_brake = False                    # Determines whether hand brake will be used. Default is False.
        self._reverse = False                       # Determines whether the vehicle will move backwards. Default is False.
        self._manual_gear_shift = False             # Determines whether the vehicle will be controlled by changing gears manually. Default is False.
        self._gear = 0                              # States which gear is the vehicle running on. Default is 0.

        # Measurements
        self._speed = 0.0

    def _actuation_callback(self, sample):
        if sample.kind == zenoh.SampleKind.PUT and sample.key_expr == 'ego_vehicle/input/actuation':
            self._actuation = float(sample.payload.to_string())

    def get_actuation(self) -> tuple:
        self._throttle = CarlaUtils.MIN_THROTTLE
        self._brake = CarlaUtils.MIN_BRAKE

        if self._actuation >= 0.0:
            self._throttle = CarlaUtils.clamp_throttle(self._actuation)
        else:
            self._brake = CarlaUtils.clamp_brake(abs(self._actuation))

        return (self._throttle, self._brake)

    def publish_brake(self, brake: float):
        self._brake = brake
        self._brake_publisher.put(f"{self._brake}")

    def publish_speed(self, speed: float):
        self._speed = speed
        self._speed_publisher.put(f"{self._speed}")
