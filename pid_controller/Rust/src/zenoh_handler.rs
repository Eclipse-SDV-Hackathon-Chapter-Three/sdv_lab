use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json;
use log::{info, error};
use zenoh::prelude::r#async::*;

use crate::pid_controller::PIDController;

#[derive(Debug, Serialize, Deserialize)]
struct VelocityStatus {
    velocity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClockStatus {
    time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TargetSpeed {
    speed: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EngageStatus {
    engaged: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActuationCommand {
    acceleration: f64,
    timestamp: u64,
}

pub struct ZenohHandler {
    controller: Arc<Mutex<PIDController>>,
    session: Arc<zenoh::Session>,
    sub_stamp: String,
    sub_current: String,
    sub_desired: String,
    sub_enable: String,
    pub_acc: String,
    
    // State variables
    current_velocity: Arc<Mutex<f64>>,
    desired_velocity: Arc<Mutex<f64>>,
    current_time: Arc<Mutex<u64>>,
    previous_time: Arc<Mutex<u64>>,
    is_engaged: Arc<Mutex<u8>>,
    pid_active: Arc<Mutex<bool>>,
    
    // Results storage
    results: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl ZenohHandler {
    pub fn new(
        controller: PIDController,
        session: zenoh::Session,
        sub_stamp: String,
        sub_current: String,
        sub_desired: String,
        sub_enable: String,
        pub_acc: String,
    ) -> Self {
        let mut results = HashMap::new();
        results.insert("desired_velocity".to_string(), Vec::new());
        results.insert("current_velocity".to_string(), Vec::new());
        results.insert("current_time".to_string(), Vec::new());
        results.insert("acceleration".to_string(), Vec::new());

        ZenohHandler {
            controller: Arc::new(Mutex::new(controller)),
            session: Arc::new(session),
            sub_stamp,
            sub_current,
            sub_desired,
            sub_enable,
            pub_acc,
            current_velocity: Arc::new(Mutex::new(0.0)),
            desired_velocity: Arc::new(Mutex::new(0.0)),
            current_time: Arc::new(Mutex::new(0)),
            previous_time: Arc::new(Mutex::new(0)),
            is_engaged: Arc::new(Mutex::new(0)),
            pid_active: Arc::new(Mutex::new(false)),
            results: Arc::new(Mutex::new(results)),
        }
    }

    pub async fn start(&self) {
        info!("Starting ZenohHandler subscribers...");

        // Spawn tasks for each subscriber to keep them alive
        self.setup_clock_subscriber().await;
        self.setup_velocity_subscriber().await;
        self.setup_target_subscriber().await;
        self.setup_engage_subscriber().await;
    }
    
    async fn setup_clock_subscriber(&self) {
        let current_time_arc = Arc::clone(&self.current_time);
        let sub_stamp = self.sub_stamp.clone();
        let session = Arc::clone(&self.session);
        
        tokio::spawn(async move {
            let subscriber = session.declare_subscriber(&sub_stamp)
                .res().await.unwrap();
            
            info!("Timestamp subscriber started, waiting for data...");
            
            while let Ok(sample) = subscriber.recv_async().await {
                let bytes = sample.payload.contiguous();
                if let Ok(clock_status) = serde_json::from_slice::<ClockStatus>(&bytes) {
                    let time = clock_status.time;
                    {
                        let mut current_time = current_time_arc.lock().unwrap();
                        *current_time = time;
                    }
                    info!("Received current clock '{}'", time);
                } else {
                    error!("[ERROR] Timestamp processing failed");
                }
            }
        });
    }
    
    async fn setup_velocity_subscriber(&self) {
        let current_velocity = Arc::clone(&self.current_velocity);
        let sub_current = self.sub_current.clone();
        let session = Arc::clone(&self.session);
        
        // Clone all necessary data for publish_acc
        let desired_velocity = Arc::clone(&self.desired_velocity);
        let current_time = Arc::clone(&self.current_time);
        let previous_time = Arc::clone(&self.previous_time);
        let pid_active = Arc::clone(&self.pid_active);
        let controller = Arc::clone(&self.controller);
        let results = Arc::clone(&self.results);
        let pub_acc = self.pub_acc.clone();
        
        tokio::spawn(async move {
            let subscriber = session.declare_subscriber(&sub_current)
                .res().await.unwrap();
            
            info!("Velocity subscriber started, waiting for data...");
            
            while let Ok(sample) = subscriber.recv_async().await {
                let bytes = sample.payload.contiguous();
                if let Ok(velocity_status) = serde_json::from_slice::<VelocityStatus>(&bytes) {
                    {
                        let mut vel = current_velocity.lock().unwrap();
                        *vel = velocity_status.velocity;
                    }
                    info!("Received current velocity '{}'", velocity_status.velocity);
                    
                    // Trigger PID computation (equivalent to Python's self.publish_acc())
                    Self::publish_acc(
                        &desired_velocity,
                        &current_velocity,
                        &current_time,
                        &previous_time,
                        &pid_active,
                        &controller,
                        &session,
                        pub_acc.clone(),
                        &results,
                    ).await;
                }
            }
        });
    }

    async fn setup_target_subscriber(&self) {
        let desired_velocity = Arc::clone(&self.desired_velocity);
        let sub_desired = self.sub_desired.clone();
        let session = Arc::clone(&self.session);
        
        tokio::spawn(async move {
            let subscriber = session.declare_subscriber(&sub_desired)
                .res().await.unwrap();
            
            info!("Target Speed subscriber started, waiting for data...");
            
            while let Ok(sample) = subscriber.recv_async().await {
                let bytes = sample.payload.contiguous();
                
                // Try JSON format first, then fall back to raw number
                let speed_value = if let Ok(target_speed) = serde_json::from_slice::<TargetSpeed>(&bytes) {
                    target_speed.speed
                } else if let Ok(payload_str) = std::str::from_utf8(&bytes) {
                    // Try to parse as raw number
                    match payload_str.trim().parse::<f64>() {
                        Ok(speed) => speed,
                        Err(_) => {
                            error!("Failed to parse target speed: {}", payload_str);
                            continue;
                        }
                    }
                } else {
                    error!("Failed to parse target speed payload");
                    continue;
                };
                
                {
                    let mut vel = desired_velocity.lock().unwrap();
                    *vel = speed_value;
                }
                info!("Received desired velocity '{}'", speed_value);
            }
        });
    }
    
    async fn setup_engage_subscriber(&self) {
        let is_engaged = Arc::clone(&self.is_engaged);
        let pid_active = Arc::clone(&self.pid_active);
        let controller = Arc::clone(&self.controller);
        let sub_enable = self.sub_enable.clone();
        let session = Arc::clone(&self.session);
        
        tokio::spawn(async move {
            let subscriber = session.declare_subscriber(&sub_enable)
                .res().await.unwrap();
            
            info!("Engage subscriber started, waiting for data...");
            
            while let Ok(sample) = subscriber.recv_async().await {
                let bytes = sample.payload.contiguous();
                if let Ok(engage_status) = serde_json::from_slice::<EngageStatus>(&bytes) {
                    let _was_engaged;
                    let engaged = engage_status.engaged;
                    {
                        let mut engaged_state = is_engaged.lock().unwrap();
                        _was_engaged = *engaged_state;
                        *engaged_state = engaged;
                    }
                    
                    info!("Received engage status: {}", engaged);
                    
                    // Handle activation/deactivation
                    let enable = engaged != 0;
                    let was_active = {
                        let active = pid_active.lock().unwrap();
                        *active
                    };
                    
                    if enable && !was_active {
                        Self::activate_pid(&pid_active, &controller);
                    } else if !enable && was_active {
                        Self::deactivate_pid(&pid_active, &controller);
                    }
                }
            }
        });
    }

    // Static method for PID computation and publishing (equivalent to Python's publish_acc)
    async fn publish_acc(
        desired_velocity: &Arc<Mutex<f64>>,
        current_velocity: &Arc<Mutex<f64>>,
        current_time: &Arc<Mutex<u64>>,
        previous_time: &Arc<Mutex<u64>>,
        pid_active: &Arc<Mutex<bool>>,
        controller: &Arc<Mutex<PIDController>>,
        session: &Arc<zenoh::Session>,
        pub_acc: String,  // Keep as String (owned)
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
            match pid.compute(desired_vel, current_vel, curr_time as f64) {
                Ok(acc) => acc,
                Err(e) => {
                    error!("PID computation failed: {}", e);
                    return; // Skip publishing if computation fails
                }
            }
        };

        // Create publisher and publish acceleration
        let publisher = session.declare_publisher(pub_acc).res().await.unwrap();
        let actuation_cmd = ActuationCommand {
            acceleration,
            timestamp: curr_time,
        };
        
        if let Ok(json_payload) = serde_json::to_string(&actuation_cmd) {
            if let Err(e) = publisher.put(json_payload).res().await {
                error!("Failed to publish acceleration: {}", e);
            } else {
                info!("Publishing Acceleration: {}", acceleration);
            }
        }

        // Store results for later analysis
        {
            let mut results_guard = results.lock().unwrap();
            results_guard.get_mut("desired_velocity").unwrap().push(desired_vel);
            results_guard.get_mut("current_velocity").unwrap().push(current_vel);
            results_guard.get_mut("current_time").unwrap().push(curr_time as f64);
            results_guard.get_mut("acceleration").unwrap().push(acceleration);
        }

        // Calculate and log delta time
        let (_prev_time, delta_time) = {
            let mut prev = previous_time.lock().unwrap();
            let delta = if *prev > 0 { curr_time - *prev } else { 0 };
            *prev = curr_time;
            (*prev, delta)
        };
        
        if delta_time > 0 {
            info!("Delta time: {} seconds", delta_time);
        }
    }

    // Activation method (equivalent to Python's _activate_pid)
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

    // Deactivation method (equivalent to Python's _deactivate_pid)
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
        
        // Store each result type in separate files (like Python version)
        for (key, values) in results.iter() {
            let filename = format!("{}.log", key);
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
            std::fs::write("pid_results.json", json).unwrap_or_else(|e| {
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
                
                // Additional statistics
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
    pub fn get_state(&self) -> (f64, f64, u64, bool) {
        let current_vel = *self.current_velocity.lock().unwrap();
        let desired_vel = *self.desired_velocity.lock().unwrap();
        let current_time = *self.current_time.lock().unwrap();
        let is_active = *self.pid_active.lock().unwrap();
        
        (current_vel, desired_vel, current_time, is_active)
    }
}