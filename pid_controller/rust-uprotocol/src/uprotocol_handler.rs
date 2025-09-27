//
// Copyright (c) 2025 The X-Verse <https://github.com/The-Xverse>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json;
use log::{info, debug, error};
use up_rust::{UUri, UListener, UMessage, UMessageBuilder, UTransport, UPayloadFormat};
use up_transport_zenoh::UPTransportZenoh;

use crate::pid_controller::PIDController;

#[derive(Debug, Serialize, Deserialize)]
struct VelocityStatus {
    velocity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClockStatus {
    time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TargetSpeed {
    speed: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EngageStatus {
    engaged: u8,
}

pub struct UProtocolHandler {
    controller: Arc<Mutex<PIDController>>,
    transport: Arc<UPTransportZenoh>,
    
    // uProtocol URIs
    velocity_uri: UUri,
    clock_uri: UUri,
    engage_uri: UUri,
    target_speed_uri: UUri,
    actuation_uri: UUri,
    
    // State variables
    current_velocity: Arc<Mutex<f64>>,
    desired_velocity: Arc<Mutex<f64>>,
    current_time: Arc<Mutex<f64>>,
    previous_time: Arc<Mutex<f64>>,
    is_engaged: Arc<Mutex<u8>>,
    pid_active: Arc<Mutex<bool>>,
    
    // Results storage
    results: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl UProtocolHandler {
    pub fn new(
        controller: PIDController,
        transport: UPTransportZenoh,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut results = HashMap::new();
        results.insert("desired_velocity".to_string(), Vec::new());
        results.insert("current_velocity".to_string(), Vec::new());
        results.insert("current_time".to_string(), Vec::new());
        results.insert("acceleration".to_string(), Vec::new());

        // Create URIs for different services
        let velocity_uri = UUri::try_from_parts("EGOVehicle", 0, 2, 0x8001)?;
        let clock_uri = UUri::try_from_parts("EGOVehicle", 0, 2, 0x8002)?;
        let engage_uri = UUri::try_from_parts("AAOS", 0, 2, 0x8002)?;
        let target_speed_uri = UUri::try_from_parts("AAOS", 0, 2, 0x8001)?;
        let actuation_uri = UUri::try_from_parts("CruiseControl", 0, 2, 0x8001)?;

        Ok(UProtocolHandler {
            controller: Arc::new(Mutex::new(controller)),
            transport: Arc::new(transport),
            velocity_uri,
            clock_uri,
            engage_uri,
            target_speed_uri,
            actuation_uri,
            current_velocity: Arc::new(Mutex::new(0.0)),
            desired_velocity: Arc::new(Mutex::new(0.0)),
            current_time: Arc::new(Mutex::new(0.0)),
            previous_time: Arc::new(Mutex::new(0.0)),
            is_engaged: Arc::new(Mutex::new(0)),
            pid_active: Arc::new(Mutex::new(false)),
            results: Arc::new(Mutex::new(results)),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting UProtocolHandler subscribers...");

        // Register listeners for each subscription
        self.setup_clock_subscriber().await?;
        self.setup_velocity_subscriber().await?;
        self.setup_target_subscriber().await?;
        self.setup_engage_subscriber().await?;

        Ok(())
    }
    
    async fn setup_clock_subscriber(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_time_arc = Arc::clone(&self.current_time);
        let transport = Arc::clone(&self.transport);
        let clock_uri = self.clock_uri.clone();
        
        let listener = ClockListener::new(current_time_arc);
        transport.register_listener(&clock_uri, None, Arc::new(listener)).await?;
        
        info!("Timestamp subscriber registered");
        Ok(())
    }
    
    async fn setup_velocity_subscriber(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_velocity = Arc::clone(&self.current_velocity);
        let transport = Arc::clone(&self.transport);
        let velocity_uri = self.velocity_uri.clone();
        
        // Clone all necessary data for publish_acc
        let desired_velocity = Arc::clone(&self.desired_velocity);
        let current_time = Arc::clone(&self.current_time);
        let previous_time = Arc::clone(&self.previous_time);
        let pid_active = Arc::clone(&self.pid_active);
        let controller = Arc::clone(&self.controller);
        let results = Arc::clone(&self.results);
        let actuation_uri = self.actuation_uri.clone();
        let transport_for_publish = Arc::clone(&self.transport);
        
        let listener = VelocityListener::new(
            current_velocity,
            desired_velocity,
            current_time,
            previous_time,
            pid_active,
            controller,
            results,
            actuation_uri,
            transport_for_publish,
        );
        
        transport.register_listener(&velocity_uri, None, Arc::new(listener)).await?;
        
        info!("Velocity subscriber registered");
        Ok(())
    }

    async fn setup_target_subscriber(&self) -> Result<(), Box<dyn std::error::Error>> {
        let desired_velocity = Arc::clone(&self.desired_velocity);
        let transport = Arc::clone(&self.transport);
        let target_speed_uri = self.target_speed_uri.clone();
        
        let listener = TargetSpeedListener::new(desired_velocity);
        transport.register_listener(&target_speed_uri, None, Arc::new(listener)).await?;
        
        info!("Target Speed subscriber registered");
        Ok(())
    }
    
    async fn setup_engage_subscriber(&self) -> Result<(), Box<dyn std::error::Error>> {
        let is_engaged = Arc::clone(&self.is_engaged);
        let pid_active = Arc::clone(&self.pid_active);
        let controller = Arc::clone(&self.controller);
        let transport = Arc::clone(&self.transport);
        let engage_uri = self.engage_uri.clone();
        
        let listener = EngageListener::new(is_engaged, pid_active, controller);
        transport.register_listener(&engage_uri, None, Arc::new(listener)).await?;
        
        info!("Engage subscriber registered");
        Ok(())
    }

    // Static method for PID computation and publishing
    async fn publish_acc(
        desired_velocity: &Arc<Mutex<f64>>,
        current_velocity: &Arc<Mutex<f64>>,
        current_time: &Arc<Mutex<f64>>,
        previous_time: &Arc<Mutex<f64>>,
        pid_active: &Arc<Mutex<bool>>,
        controller: &Arc<Mutex<PIDController>>,
        transport: &Arc<UPTransportZenoh>,
        actuation_uri: UUri,
        results: &Arc<Mutex<HashMap<String, Vec<f64>>>>,
    ) {
        // Check if PID is active
        let is_active = {
            let active = pid_active.lock().unwrap();
            *active
        };
        
        if !is_active {
            return;
        }

        let (desired_vel, current_vel, curr_time) = {
            let desired = desired_velocity.lock().unwrap();
            let current = current_velocity.lock().unwrap();
            let time = current_time.lock().unwrap();
            (*desired, *current, *time)
        };

        // Compute acceleration using PID controller
        let acceleration = {
            let mut pid = controller.lock().unwrap();
            match pid.compute(desired_vel, current_vel, curr_time) {
                Ok(acc) => acc,
                Err(e) => {
                    error!("PID computation failed: {}", e);
                    return;
                }
            }
        };

        // Create and publish uProtocol message
        let actuation_cmd_payload = format!("{}", acceleration);
        let message = UMessageBuilder::publish(actuation_uri)
            .build_with_payload(actuation_cmd_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)
            .unwrap();
        
        if let Err(e) = transport.send(message).await {
            error!("Failed to publish acceleration: {}", e);
        } else {
            debug!("Publishing Acceleration: {}", actuation_cmd_payload);
        }

        // Store results for later analysis
        {
            let mut results_guard = results.lock().unwrap();
            results_guard.get_mut("desired_velocity").unwrap().push(desired_vel);
            results_guard.get_mut("current_velocity").unwrap().push(current_vel);
            results_guard.get_mut("current_time").unwrap().push(curr_time);
            results_guard.get_mut("acceleration").unwrap().push(acceleration);
        }

        // Calculate and log delta time
        let (_prev_time, delta_time) = {
            let mut prev = previous_time.lock().unwrap();
            let delta = if *prev > 0.0 { curr_time - *prev } else { 0.0 };
            *prev = curr_time;
            (*prev, delta)
        };
        
        if delta_time > 0.0 {
            debug!("Delta time: {} seconds", delta_time);
        }
    }

    // Activation method
    fn activate_pid(
        pid_active: &Arc<Mutex<bool>>,
        controller: &Arc<Mutex<PIDController>>,
    ) {
        {
            let mut active = pid_active.lock().unwrap();
            *active = true;
        }
        {
            let mut pid = controller.lock().unwrap();
            pid.reset();
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        info!("[INFO] PID controller ACTIVATED at {}", timestamp);
    }

    // Deactivation method
    fn deactivate_pid(
        pid_active: &Arc<Mutex<bool>>,
        controller: &Arc<Mutex<PIDController>>,
    ) {
        {
            let mut active = pid_active.lock().unwrap();
            *active = false;
        }
        {
            let mut pid = controller.lock().unwrap();
            pid.reset();
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        info!("[INFO] PID controller DEACTIVATED at {}", timestamp);
    }
    
    pub fn store_results(&self) {
        let results = self.results.lock().unwrap();
        
        // Create logs directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all("logs") {
            error!("Failed to create logs directory: {}", e);
            return;
        }
        
        // Store each result type in separate files
        for (key, values) in results.iter() {
            let filename = format!("logs/{}.log", key);
            let content = values.iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            
            if let Err(e) = std::fs::write(&filename, content) {
                error!("Failed to write {}: {}", filename, e);
            } else {
                info!("Results saved to {}", filename);
            }
        }

        // Also save as JSON for compatibility
        if let Ok(json) = serde_json::to_string(&*results) {
            std::fs::write("logs/pid_results.json", json).unwrap_or_else(|e| {
                error!("Failed to write JSON results: {}", e);
            });
        }
    }
    
    pub fn show_results(&self) {
        let results = self.results.lock().unwrap();
        
        info!("PID Controller Results Summary:");
        
        if let (Some(desired), Some(current), Some(acceleration)) = (
            results.get("desired_velocity"),
            results.get("current_velocity"), 
            results.get("acceleration")
        ) {
            let data_points = desired.len().min(current.len()).min(acceleration.len());
            info!("Total data points: {}", data_points);
            
            if data_points > 0 {
                let mut min_error = f64::MAX;
                let mut max_error = f64::MIN;
                let mut sum_error = 0.0;
                
                for i in 0..data_points {
                    let error = desired[i] - current[i];
                    min_error = min_error.min(error);
                    max_error = max_error.max(error);
                    sum_error += error;
                }
                
                let avg_error = sum_error / data_points as f64;
                
                info!("Min error: {:.4}", min_error);
                info!("Max error: {:.4}", max_error);
                info!("Avg error: {:.4}", avg_error);
                
                if let Some(acc_values) = results.get("acceleration") {
                    if !acc_values.is_empty() {
                        let min_acc = acc_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max_acc = acc_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                        let avg_acc = acc_values.iter().sum::<f64>() / acc_values.len() as f64;
                        
                        info!("Acceleration - Min: {:.4}, Max: {:.4}, Avg: {:.4}", min_acc, max_acc, avg_acc);
                    }
                }
            }
        } else {
            info!("No data points available");
        }
    }

    // Additional helper method to get current PID status
    #[allow(dead_code)]    
    pub fn is_active(&self) -> bool {
        let active = self.pid_active.lock().unwrap();
        *active
    }

    // Get current state for debugging
    #[allow(dead_code)]    
    pub fn get_state(&self) -> (f64, f64, f64, bool) {
        let current_vel = *self.current_velocity.lock().unwrap();
        let desired_vel = *self.desired_velocity.lock().unwrap();
        let current_time = *self.current_time.lock().unwrap();
        let is_active = *self.pid_active.lock().unwrap();
        
        (current_vel, desired_vel, current_time, is_active)
    }
}

// Listener implementations
struct ClockListener {
    current_time: Arc<Mutex<f64>>,
}

impl ClockListener {
    fn new(current_time: Arc<Mutex<f64>>) -> Self {
        Self { current_time }
    }
}

#[async_trait::async_trait]
impl UListener for ClockListener {
    async fn on_receive(&self, message: UMessage) {
        if let Some(payload) = message.payload {
            let bytes = &payload[..];
            
            // Try to parse as text first (new format)
            let time_value = if let Ok(payload_str) = std::str::from_utf8(&bytes) {
                match payload_str.trim().parse::<f64>() {
                    Ok(time) => time,
                    Err(_) => {
                        // Fall back to JSON format for backward compatibility
                        if let Ok(clock_status) = serde_json::from_slice::<ClockStatus>(&bytes) {
                            clock_status.time
                        } else {
                            error!("[ERROR] Timestamp processing failed as JSON");
                            return;
                        }
                    }
                }
            } else {
                error!("[ERROR] Timestamp processing failed as UTF-8");
                return;
            };
            
            {
                let mut clock = self.current_time.lock().unwrap();
                *clock = time_value;
            }
            debug!("Received current clock '{:.4}' seconds", time_value);
        }
    }
}

struct VelocityListener {
    current_velocity: Arc<Mutex<f64>>,
    desired_velocity: Arc<Mutex<f64>>,
    current_time: Arc<Mutex<f64>>,
    previous_time: Arc<Mutex<f64>>,
    pid_active: Arc<Mutex<bool>>,
    controller: Arc<Mutex<PIDController>>,
    results: Arc<Mutex<HashMap<String, Vec<f64>>>>,
    actuation_uri: UUri,
    transport: Arc<UPTransportZenoh>,
}

impl VelocityListener {
    fn new(
        current_velocity: Arc<Mutex<f64>>,
        desired_velocity: Arc<Mutex<f64>>,
        current_time: Arc<Mutex<f64>>,
        previous_time: Arc<Mutex<f64>>,
        pid_active: Arc<Mutex<bool>>,
        controller: Arc<Mutex<PIDController>>,
        results: Arc<Mutex<HashMap<String, Vec<f64>>>>,
        actuation_uri: UUri,
        transport: Arc<UPTransportZenoh>,
    ) -> Self {
        Self {
            current_velocity,
            desired_velocity,
            current_time,
            previous_time,
            pid_active,
            controller,
            results,
            actuation_uri,
            transport,
        }
    }
}

#[async_trait::async_trait]
impl UListener for VelocityListener {
    async fn on_receive(&self, message: UMessage) {
        if let Some(payload) = message.payload {
            let bytes = &payload[..];
            
            // Try to parse as text first (new format)
            let velocity_value = if let Ok(payload_str) = std::str::from_utf8(&bytes) {
                match payload_str.trim().parse::<f64>() {
                    Ok(velocity) => velocity,
                    Err(_) => {
                        // Fall back to JSON format for backward compatibility
                        if let Ok(velocity_status) = serde_json::from_slice::<VelocityStatus>(&bytes) {
                            velocity_status.velocity
                        } else {
                            error!("Failed to parse velocity payload");
                            return;
                        }
                    }
                }
            } else {
                error!("Failed to parse velocity payload as UTF-8");
                return;
            };
            
            {
                let mut vel = self.current_velocity.lock().unwrap();
                *vel = velocity_value;
            }
            debug!("Received current velocity '{:.2}'", velocity_value);
            
            // Trigger PID computation
            UProtocolHandler::publish_acc(
                &self.desired_velocity,
                &self.current_velocity,
                &self.current_time,
                &self.previous_time,
                &self.pid_active,
                &self.controller,
                &self.transport,
                self.actuation_uri.clone(),
                &self.results,
            ).await;
        }
    }
}

struct TargetSpeedListener {
    desired_velocity: Arc<Mutex<f64>>,
}

impl TargetSpeedListener {
    fn new(desired_velocity: Arc<Mutex<f64>>) -> Self {
        Self { desired_velocity }
    }
}

#[async_trait::async_trait]
impl UListener for TargetSpeedListener {
    async fn on_receive(&self, message: UMessage) {
        if let Some(payload) = message.payload {
            let bytes = &payload[..];
            
            let speed_value = if let Ok(target_speed) = serde_json::from_slice::<TargetSpeed>(&bytes) {
                target_speed.speed
            } else if let Ok(payload_str) = std::str::from_utf8(&bytes) {
                match payload_str.trim().parse::<f64>() {
                    Ok(speed) => speed,
                    Err(_) => {
                        error!("Failed to parse target speed: {}", payload_str);
                        return;
                    }
                }
            } else {
                error!("Failed to parse target speed payload");
                return;
            };
            
            {
                let mut vel = self.desired_velocity.lock().unwrap();
                *vel = speed_value;
            }
            info!("Received desired velocity '{:.2}'", speed_value);
        }
    }
}

struct EngageListener {
    is_engaged: Arc<Mutex<u8>>,
    pid_active: Arc<Mutex<bool>>,
    controller: Arc<Mutex<PIDController>>,
}

impl EngageListener {
    fn new(
        is_engaged: Arc<Mutex<u8>>,
        pid_active: Arc<Mutex<bool>>,
        controller: Arc<Mutex<PIDController>>,
    ) -> Self {
        Self {
            is_engaged,
            pid_active,
            controller,
        }
    }
}

#[async_trait::async_trait]
impl UListener for EngageListener {
    async fn on_receive(&self, message: UMessage) {
        if let Some(payload) = message.payload {
            let bytes = &payload[..];
            
            // Try to parse as text first (new format)
            let engaged_value = if let Ok(payload_str) = std::str::from_utf8(&bytes) {
                match payload_str.trim().parse::<u8>() {
                    Ok(engaged) => engaged,
                    Err(_) => {
                        // Fall back to JSON format for backward compatibility
                        if let Ok(engage_status) = serde_json::from_slice::<EngageStatus>(&bytes) {
                            engage_status.engaged
                        } else {
                            error!("Failed to parse engage status payload");
                            return;
                        }
                    }
                }
            } else {
                error!("Failed to parse engage status payload as UTF-8");
                return;
            };
            
            let _was_engaged;
            {
                let mut engaged_state = self.is_engaged.lock().unwrap();
                _was_engaged = *engaged_state;
                *engaged_state = engaged_value;
            }
            
            info!("Received engage status: {}", engaged_value);
            
            // Handle activation/deactivation
            let enable = engaged_value != 0;
            let was_active = {
                let active = self.pid_active.lock().unwrap();
                *active
            };
            
            if enable && !was_active {
                UProtocolHandler::activate_pid(&self.pid_active, &self.controller);
            } else if !enable && was_active {
                UProtocolHandler::deactivate_pid(&self.pid_active, &self.controller);
            }
        }
    }
}
