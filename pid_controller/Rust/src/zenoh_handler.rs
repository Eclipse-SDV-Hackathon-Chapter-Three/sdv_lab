use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use log::{info, error};
use zenoh::prelude::r#async::*;

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
    engaged: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActuationCommand {
    acceleration: f64,
    timestamp: f64,
}

pub struct ZenohHandler {
    controller: Arc<Mutex<PIDController>>,
    session: zenoh::Session,
    sub_stamp: String,
    sub_current: String,
    sub_desired: String,
    sub_enable: String,
    pub_acc: String,
    
    // State variables
    current_velocity: Arc<Mutex<f64>>,
    desired_velocity: Arc<Mutex<f64>>,
    current_time: Arc<Mutex<f64>>,
    is_engaged: Arc<Mutex<bool>>,
    
    // Results storage
    results: Arc<Mutex<HashMap<OrderedFloat<f64>, (f64, f64, f64)>>>, // time -> (current_vel, desired_vel, acceleration)
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
        ZenohHandler {
            controller: Arc::new(Mutex::new(controller)),
            session,
            sub_stamp,
            sub_current,
            sub_desired,
            sub_enable,
            pub_acc,
            current_velocity: Arc::new(Mutex::new(0.0)),
            desired_velocity: Arc::new(Mutex::new(0.0)),
            current_time: Arc::new(Mutex::new(0.0)),
            is_engaged: Arc::new(Mutex::new(false)),
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) {
        // Set up subscribers
        self.setup_clock_subscriber().await;
        self.setup_velocity_subscriber().await;
        self.setup_target_subscriber().await;
        self.setup_engage_subscriber().await;
    }
    
    async fn setup_clock_subscriber(&self) {
        
        let current_time_arc = Arc::clone(&self.current_time);
        let _sub = self.session.declare_subscriber(&self.sub_stamp)
            .callback(move |sample| {
                
                println!("Received sample on clock_status topic");
                let bytes = sample.payload.contiguous();
                if let Ok(clock_status) = serde_json::from_slice::<ClockStatus>(&bytes) {
                    let time = clock_status.time;
                    {
                        let mut current_time = current_time_arc.lock().unwrap();
                        *current_time = time;
                    }
                    info!("Received timestamp: {}", time);
                } else {
                    error!("Failed to deserialize timestamp payload");
                }
            })
            .res().await.unwrap();
        info!("Timestamp subscriber started, waiting for data...");
    }
    
    async fn setup_velocity_subscriber(&self) {
        let current_velocity = Arc::clone(&self.current_velocity);
        
        let _sub = self.session.declare_subscriber(&self.sub_current)
            .callback(move |sample| {
                let bytes = sample.payload.contiguous();
                if let Ok(velocity_status) = serde_json::from_slice::<VelocityStatus>(&bytes) {
                    let mut vel = current_velocity.lock().unwrap();
                    *vel = velocity_status.velocity;
                }
            })
            .res().await.unwrap();
            ;
    }
    
    async fn setup_target_subscriber(&self) {
        let desired_velocity = Arc::clone(&self.desired_velocity);
        
        let _sub = self.session.declare_subscriber(&self.sub_desired)
            .callback(move |sample| {
                let bytes = sample.payload.contiguous();
                if let Ok(target_speed) = serde_json::from_slice::<TargetSpeed>(&bytes) {
                    let mut vel = desired_velocity.lock().unwrap();
                    *vel = target_speed.speed;
                }
            })
            .res().await.unwrap();
            ;
    }
    
    async fn setup_engage_subscriber(&self) {
        let is_engaged = Arc::clone(&self.is_engaged);
        let controller = Arc::clone(&self.controller);
        
        let _sub = self.session.declare_subscriber(&self.sub_enable)
            .callback(move |sample| {
                let bytes = sample.payload.contiguous();
                if let Ok(engage_status) = serde_json::from_slice::<EngageStatus>(&bytes) {
                    let was_engaged;
                    {
                        let mut engaged = is_engaged.lock().unwrap();
                        was_engaged = *engaged;
                        *engaged = engage_status.engaged;
                    }
                    
                    // Reset controller when disengaged
                    if was_engaged && !engage_status.engaged {
                        let mut pid = controller.lock().unwrap();
                        pid.reset();
                    }
                }
            })
            .res().await.unwrap();
    }
    
    pub fn store_results(&self) {
        // Save results to file
        let results = self.results.lock().unwrap();
        let mut serializable_results = Vec::new();
        
        for (key, value) in results.iter() {
            serializable_results.push((key.into_inner(), *value));
        }
        
        if let Ok(json) = serde_json::to_string(&serializable_results) {
            std::fs::write("pid_results.json", json).unwrap_or_else(|e| {
                error!("Failed to write results: {}", e);
            });
        }
    }
    
    pub fn show_results(&self) {
        let results = self.results.lock().unwrap();
        
        info!("PID Controller Results Summary:");
        info!("Total data points: {}", results.len());
        
        if !results.is_empty() {
            let mut min_error = f64::MAX;
            let mut max_error = f64::MIN;
            let mut sum_error = 0.0;
            
            for (_, (current, desired, _)) in results.iter() {
                let error = desired - current;
                min_error = min_error.min(error);
                max_error = max_error.max(error);
                sum_error += error;
            }
            
            let avg_error = sum_error / results.len() as f64;
            
            info!("Min error: {:.4}", min_error);
            info!("Max error: {:.4}", max_error);
            info!("Avg error: {:.4}", avg_error);
        }
    }
}