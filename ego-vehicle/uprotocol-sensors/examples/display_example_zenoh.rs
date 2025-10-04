// Example demonstrating image display functionality from Zenoh
// This example shows how to receive and display CARLA sensor images from uProtocol/Zenoh

use carla_data_serde::ImageEventSerDe;
use show_image::WindowOptions;
use std::sync::{Arc, OnceLock};
use up_rust::{UTransport, UUri};
use up_rust::{UListener, UMessage};
use up_transport_zenoh::UPTransportZenoh;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};


// // Rate limiting for image display
// static LAST_DISPLAY_TIME: OnceLock<std::sync::Mutex<std::time::Instant>> = OnceLock::new();
static IMAGE_WINDOW: OnceLock<std::sync::Mutex<Option<show_image::WindowProxy>>> = OnceLock::new();
// const DISPLAY_INTERVAL_MS: u64 = 50; // Display every 50ms (20 FPS)

const FPS_LOG_INTERVAL_SECONDS: u64 = 1; // Log FPS every 1 second

struct ImageListener {
    frame_count: AtomicU64,
    received_frame_count: AtomicU64,
    fps_start_time: std::sync::Mutex<std::time::Instant>,
    last_fps_log_time: std::sync::Mutex<std::time::Instant>,
}

impl ImageListener {
    fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            frame_count: AtomicU64::new(0),
            received_frame_count: AtomicU64::new(0),
            fps_start_time: std::sync::Mutex::new(now),
            last_fps_log_time: std::sync::Mutex::new(now),
        }
    }
}

#[async_trait::async_trait]
impl UListener for ImageListener {
    async fn on_receive(&self, msg: UMessage) {

        // if let Some(payload) = msg.payload {
        let Some(payload) = msg.payload.as_deref() else {
            return;
        };
        if payload.is_empty() {
            return;
        }

        // Track received frames
        let received_frame_number = self.received_frame_count.fetch_add(1, Ordering::Relaxed) + 1;
        println!("Received image data payload of {} bytes (Received Frame #{})", payload.len(), received_frame_number);

        // Use proper deserialization with ImageEventSerDe
        match serde_json::from_slice::<ImageEventSerDe>(&payload) {
            Ok(image_data) => {
                println!("Successfully deserialized ImageEventSerDe");
                println!("Image dimensions: {}x{}", image_data.width, image_data.height);
                println!("FOV angle: {}", image_data.fov_angle);
                println!("Array length: {}", image_data.array.len());
                
                let width = image_data.width as u32;
                let height = image_data.height as u32;
                
                // The array contains FfiColor structs in an ndarray format
                let pixel_array = &image_data.array;
                
                println!("Pixel array shape: {:?}", pixel_array.shape());
                println!("Array dimensions: height={}, width={}", pixel_array.nrows(), pixel_array.ncols());
                
                // Convert ndarray of FfiColor to RGB data
                let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
                let mut non_zero_count = 0;
                
                // Iterate through the 2D array
                for row in 0..pixel_array.nrows() {
                    for col in 0..pixel_array.ncols() {
                        let pixel = &pixel_array[[row, col]];
                        
                        // FfiColor has fields: b, g, r, a
                        let r = pixel.r;
                        let g = pixel.g;
                        let b = pixel.b;
                        
                        // Count non-zero pixels for debugging
                        if r != 0 || g != 0 || b != 0 {
                            non_zero_count += 1;
                        }
                        
                        // Add RGB values (convert BGRA to RGB)
                        rgb_data.push(r);
                        rgb_data.push(g);
                        rgb_data.push(b);
                    }
                }
                
                println!("Converted {} pixels to RGB data", pixel_array.len());
                println!("Non-zero pixels: {} out of {}", non_zero_count, pixel_array.len());
                
                // Check first few RGB pixels for debugging
                if rgb_data.len() >= 12 {
                    println!("First 4 pixels RGB values: {:?}", &rgb_data[0..12]);
                    
                    // Check if image is all black
                    let non_zero_rgb_count = rgb_data.iter().filter(|&&x| x != 0).count();
                    println!("Non-zero RGB pixels: {} out of {}", non_zero_rgb_count, rgb_data.len());
                }
                
                // Ensure we have the right amount of data
                let expected_size = (width * height * 3) as usize;
                if rgb_data.len() != expected_size {
                    eprintln!("Warning: RGB data size mismatch. Expected: {}, Got: {}", expected_size, rgb_data.len());
                    // Resize or pad data if necessary
                    rgb_data.resize(expected_size, 0);
                }
                
                if let Err(e) = display_carla_image_from_raw(
                    width, 
                    height, 
                    rgb_data, 
                    "CARLA Image from Zenoh",
                    self
                ).await {
                    eprintln!("Failed to display image: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize ImageEventSerDe: {}", e);
                // Print first 100 bytes of payload for debugging
                let preview = if payload.len() > 100 { &payload[..100] } else { &payload };
                eprintln!("Payload preview: {:?}", String::from_utf8_lossy(preview));
            }
        }
        // }
    }
}

// Function to display CARLA image data using show-image from raw data (rate limited)
async fn display_carla_image_from_raw(width: u32, height: u32, rgb_data: Vec<u8>, window_title: &str, image_listener: &ImageListener) -> Result<(), Box<dyn std::error::Error>> {
    // Rate limiting - only display every DISPLAY_INTERVAL_MS milliseconds
    // let last_time = LAST_DISPLAY_TIME.get_or_init(|| std::sync::Mutex::new(std::time::Instant::now()));
    // let mut last_time_guard = match last_time.lock() {
    //     Ok(guard) => guard,
    //     Err(poisoned) => {
    //         eprintln!("Warning: Last time mutex was poisoned, recovering...");
    //         poisoned.into_inner()
    //     }
    // };
    
    // let now = std::time::Instant::now();
    // if now.duration_since(*last_time_guard) < std::time::Duration::from_millis(DISPLAY_INTERVAL_MS) {
    //     // Skip this frame due to rate limiting
    //     return Ok(());
    // }
    // *last_time_guard = now;
    // drop(last_time_guard);
    
    // Create image view
    let image_view = show_image::ImageView::new(
        show_image::ImageInfo::rgb8(width, height),
        &rgb_data,
    );
    
    // Get or create window
    let window_storage = IMAGE_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut window_guard = match window_storage.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Warning: Window mutex was poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    if window_guard.is_none() {
        // Create new window
        let new_window = show_image::create_window(window_title, WindowOptions {
            size: Some([width, height]),
            ..WindowOptions::default()
        })?;
        window_guard.replace(new_window);
    }
    
    if let Some(window) = window_guard.as_ref() {
        window.set_image("carla_image", &image_view)?;
    }
    
    // Increment frame counter and calculate FPS
    let frame_number = image_listener.frame_count.fetch_add(1, Ordering::Relaxed) + 1;
    
    // Log FPS periodically
    let mut last_log_guard = match image_listener.last_fps_log_time.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Warning: FPS log time mutex was poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    let now = std::time::Instant::now();
    if now.duration_since(*last_log_guard) >= std::time::Duration::from_secs(FPS_LOG_INTERVAL_SECONDS) {
        let mut start_guard = match image_listener.fps_start_time.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Warning: FPS start time mutex was poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        
        let elapsed = now.duration_since(*start_guard);
        let display_fps = frame_number as f64 / elapsed.as_secs_f64();
        let received_frames = image_listener.received_frame_count.load(Ordering::Relaxed);
        let receive_fps = received_frames as f64 / elapsed.as_secs_f64();

        println!("FPS - Display: {:.2}, Receive: {:.2} (Display Frame #{}, Received Frame #{}, Elapsed: {:.2}s)", 
                 display_fps, receive_fps, frame_number, received_frames, elapsed.as_secs_f64());
        
        // Reset counters and start time for next interval
        image_listener.frame_count.store(0, Ordering::Relaxed);
        image_listener.received_frame_count.store(0, Ordering::Relaxed);
        *start_guard = now;

        *last_log_guard = now;
    }
    
    println!("Image displayed in window: {} (Frame #{})", window_title, frame_number);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize show-image context - this function doesn't return!
    show_image::run_context(|| {
        // Create a tokio runtime for our async operations
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(run_display_app()).unwrap();
    });
}

async fn run_display_app() -> Result<(), Box<dyn std::error::Error>> {
    println!("CARLA Image Display Example - Receiving from Zenoh/uProtocol");
    
    // Initialize logging
    pretty_env_logger::init();

    // Create the uProtocol transport using Zenoh as the underlying transport
    let transport = UPTransportZenoh::builder("hpc")
        .expect("authority not accepted!")
        .build(/* ... building for now without configuration ... */)
        .await
        .expect("unable to build UPTransportZenoh");

    // Subscribe to image sensor data from EGOVehicle
    let image_uri = UUri::from_str("//EGOVehicle/0/2/8013")?; // IMAGE_SENSOR resource
    
    println!("Subscribing to image data from: {}", image_uri.to_uri(false));
    
    // Register the image listener
    transport
        .register_listener(
            &image_uri,
            None,
            Arc::new(ImageListener::new()),
        )
        .await
        .expect("Failed to register image listener");
    
    println!("Listening for image data... Press Ctrl+C to exit");
    
    // Keep the program running using a proper async loop
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        println!("Cancelled by user. Bye!");
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("Exiting...");

    Ok(())
}
