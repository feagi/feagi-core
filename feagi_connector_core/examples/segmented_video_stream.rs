use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use image::{DynamicImage, ImageBuffer, Rgb};
use feagi_connector_core as feagi_connector;

// region Args
/// Protocol for communication with FEAGI
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Protocol {
    /// ZeroMQ protocol
    #[value(name = "zmq")]
    ZMQ,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::ZMQ
    }
}

/// Segmented video stream example for FEAGI connector
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Protocol to use for communication
    #[arg(long, default_value = "zmq")]
    protocol: Protocol,

    /// Path to video file (local or absolute)
    #[arg(long, default_value = "video.mp4")]
    video_path: PathBuf,

    /// Socket path or address for connection
    #[arg(long, default_value = "tcp://localhost:8081")]
    socket_path: String,

    /// Enable gaze reflex functionality
    #[arg(long, default_value = "false")]
    with_gaze_reflex: bool,

    /// Eccentricity X value (0.0 to 1.0)
    #[arg(long, default_value = "0.5")]
    eccentricity_x: f32,

    /// Eccentricity Y value (0.0 to 1.0)
    #[arg(long, default_value = "0.5")]
    eccentricity_y: f32,

    /// Modularity X value (0.0 to 1.0)
    #[arg(long, default_value = "0.5")]
    modularity_x: f32,

    /// Modularity Y value (0.0 to 1.0)
    #[arg(long, default_value = "0.5")]
    modularity_y: f32,
}

//endregion

fn main() {
    let args = Args::parse();

    println!("Protocol: {:?}", args.protocol);
    println!("Video path: {:?}", args.video_path);
    println!("Socket path: {}", args.socket_path);
    println!("With gaze reflex: {}", args.with_gaze_reflex);
    println!("Eccentricity: ({}, {})", args.eccentricity_x, args.eccentricity_y);
    println!("Modularity: ({}, {})", args.modularity_x, args.modularity_y);

    // Sanity Checks
    {
        match args.video_path.try_exists() {
            Err(e) => panic!("Unable to access filesystem! {}", e),
            Ok(false) => panic!("Unable to locate file at path {}", args.video_path.display()),
            Ok(true) => (),
        }
        if !(0.0..1.0f32).contains(&args.eccentricity_x) || !(0.0..1.0f32).contains(&args.eccentricity_y) || !(0.0..1.0f32).contains(&args.modularity_x) || !(0.0..1.0f32).contains(&args.modularity_y) {
            panic!("Modularity and Eccentricity values must be within 0 and 1!")
        }
    }




    // TODO: Implement video streaming logic
}

