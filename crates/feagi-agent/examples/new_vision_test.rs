//! Executable example: register a frame-based agent and stream frames to FEAGI.
//!
//! Usage:
//! 1) Ensure FEAGI is running with a loaded genome.
//! 2) Put PNG/JPG/BMP/TIFF frames in a directory.
//! 3) Run this example with the required environment variables.
//!
//! Example:
//! FEAGI_TEST_FRAME_DIR="/path/to/frames" \
//! FEAGI_TEST_FRAME_LOOPS=3 \
//! FEAGI_TEST_GAZE_X=0.5 \
//! FEAGI_TEST_GAZE_Y=0.5 \
//! FEAGI_TEST_GAZE_MODULATION=0.5 \
//! cargo run --example system_frame_agent
//!
//! Required environment variables:
//! - FEAGI_TEST_FRAME_DIR (path to frame directory)
//!
//! Optional environment variables (defaults shown):
//! - FEAGI_TEST_FRAME_LOOPS (default: 3)
//! - FEAGI_TEST_GAZE_X (default: 0.5, range: 0.0-1.0)
//! - FEAGI_TEST_GAZE_Y (default: 0.5, range: 0.0-1.0)
//! - FEAGI_TEST_GAZE_MODULATION (default: 0.5, range: 0.0-1.0)

use anyhow::{Context, Result};
use feagi_agent::sdk::registration::{AgentRegistrar, FeagiApiConfig};
use feagi_agent::sdk::types::{
    ColorSpace, CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, FrameChangeHandling,
    GazeProperties, ImageFrame, SegmentedImageFrameProperties,
};
use feagi_agent::sdk::{AgentDescriptor, ConnectorAgent};
use feagi_agent::{AgentClient, AgentConfig, AgentType};
use feagi_config::{load_config, FeagiConfig};
use feagi_io::SensoryUnit;
use feagi_sensorimotor::data_types::descriptors::{ColorSpace, SegmentedXYImageResolutions};
use feagi_sensorimotor::data_types::{ImageFrame, Percentage, Percentage2D};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Example settings sourced from environment variables.
struct ExampleSettings {
    agent_id: String,
    cortical_unit_id: u8,
    color_space: ColorSpace,
    frame_dir: PathBuf,
    frame_loops: usize,
    gaze_x: f32,
    gaze_y: f32,
    gaze_modulation: f32,
}

fn main() -> Result<()> {
    run_example()
}

/// Load the FEAGI configuration using the standard loader.
fn load_feagi_config() -> Result<FeagiConfig> {
    load_config(None, None).context("Failed to load FEAGI configuration")
}

/// Require an environment variable and return its raw string value.
fn require_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be set"))
}

/// Parse an environment variable or return a provided default.
fn parse_env_or_default<T>(name: &str, default_value: T) -> Result<T>
where
    T: FromStr + Copy,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(raw) => raw
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("{name} must be a valid value; got '{raw}'; error: {e}")),
        Err(_) => Ok(default_value),
    }
}

/// Load example settings from environment variables.
fn load_example_settings() -> Result<ExampleSettings> {
    let descriptor = AgentDescriptor::new(1, "feagi", "system-frame-test", 1)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Failed to build AgentDescriptor")?;
    Ok(ExampleSettings {
        agent_id: descriptor.to_base64(),
        cortical_unit_id: 0,
        color_space: ColorSpace::Gamma,
        frame_dir: PathBuf::from(require_env("FEAGI_TEST_FRAME_DIR")?),
        frame_loops: parse_env_or_default("FEAGI_TEST_FRAME_LOOPS", 3)?,
        gaze_x: parse_env_or_default("FEAGI_TEST_GAZE_X", 0.5)?,
        gaze_y: parse_env_or_default("FEAGI_TEST_GAZE_Y", 0.5)?,
        gaze_modulation: parse_env_or_default("FEAGI_TEST_GAZE_MODULATION", 0.5)?,
    })
}

/// Determine if a file extension is supported by the image loader.
fn is_supported_extension(extension: &str) -> bool {
    matches!(extension, "png" | "jpg" | "jpeg" | "bmp" | "tiff")
}

/// Load image frame paths from a directory, sorted by file name.
fn load_frame_paths(frame_dir: &Path) -> Result<Vec<PathBuf>> {
    if !frame_dir.is_dir() {
        return Err(anyhow::anyhow!(
            "FEAGI_TEST_FRAME_DIR must be a directory: {}",
            frame_dir.display()
        ));
    }

    let mut paths: Vec<PathBuf> = fs::read_dir(frame_dir)
        .with_context(|| format!("Failed to read frame directory: {}", frame_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| is_supported_extension(&ext.to_ascii_lowercase()))
                .unwrap_or(false)
        })
        .collect();

    paths.sort();

    if paths.is_empty() {
        return Err(anyhow::anyhow!(
            "No supported frame files found in {}",
            frame_dir.display()
        ));
    }

    Ok(paths)
}

/// Load a single image frame from disk using the requested color space.
fn load_image_frame(path: &Path, color_space: &ColorSpace) -> Result<ImageFrame> {
    let bytes =
        fs::read(path).with_context(|| format!("Failed to read frame: {}", path.display()))?;
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .ok_or_else(|| anyhow::anyhow!("Frame missing extension: {}", path.display()))?;

    let frame_result = match extension.as_str() {
        "png" => ImageFrame::new_from_png_bytes(&bytes, color_space),
        "jpg" | "jpeg" => ImageFrame::new_from_jpeg_bytes(&bytes, color_space),
        "bmp" => ImageFrame::new_from_bmp_bytes(&bytes, color_space),
        "tiff" => ImageFrame::new_from_tiff_bytes(&bytes, color_space),
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported frame extension '{}': {}",
                extension,
                path.display()
            ))
        }
    };

    let frame = frame_result.map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(frame)
}

/// Load frames and validate consistent dimensions.
fn load_frame_sequence(
    frame_paths: &[PathBuf],
    color_space: &ColorSpace,
) -> Result<Vec<ImageFrame>> {
    if frame_paths.is_empty() {
        return Err(anyhow::anyhow!("No frame paths provided"));
    }

    let mut frames = Vec::with_capacity(frame_paths.len());
    let mut expected_resolution: Option<feagi_agent::sdk::types::ImageXYResolution> = None;
    let mut expected_layout = None;

    for path in frame_paths {
        let frame = load_image_frame(path, color_space)?;
        let resolution = frame.get_xy_resolution();
        let layout = *frame.get_channel_layout();

        if let Some(expected) = expected_resolution {
            if resolution.width != expected.width || resolution.height != expected.height {
                return Err(anyhow::anyhow!(
                    "Frame resolution mismatch in {} (expected {}x{}, got {}x{})",
                    path.display(),
                    expected.width,
                    expected.height,
                    resolution.width,
                    resolution.height
                ));
            }
        } else {
            expected_resolution = Some(resolution);
        }

        if let Some(expected) = expected_layout {
            if layout != expected {
                return Err(anyhow::anyhow!(
                    "Frame channel layout mismatch in {} (expected {:?}, got {:?})",
                    path.display(),
                    expected,
                    layout
                ));
            }
        } else {
            expected_layout = Some(layout);
        }

        frames.push(frame);
    }

    Ok(frames)
}

fn format_tcp_endpoint(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("tcp://[{host}]:{port}")
    } else {
        format!("tcp://{host}:{port}")
    }
}

/// Build the agent configuration from FEAGI config and settings.
fn build_agent_config(
    config: &FeagiConfig,
    settings: &ExampleSettings,
    resolution: (u32, u32),
) -> Result<AgentConfig> {
    let (width, height) = resolution;
    let timestep = config.neural.burst_engine_timestep;
    if timestep <= 0.0 {
        return Err(anyhow::anyhow!(
            "config.neural.burst_engine_timestep must be > 0"
        ));
    }

    let service_startup_ms = config.timeouts.service_startup * 1000.0;
    let connect_timeout_ms = config.zmq.socket_connect_timeout;
    if connect_timeout_ms == 0 {
        return Err(anyhow::anyhow!(
            "config.zmq.socket_connect_timeout must be > 0"
        ));
    }
    let registration_retries = (service_startup_ms / connect_timeout_ms as f64).ceil() as u32;
    if registration_retries == 0 {
        return Err(anyhow::anyhow!(
            "Derived feagi_registration_retries must be > 0"
        ));
    }

    let sensory_hwm = i32::try_from(config.zmq.streams.sensory.receive_high_water_mark)
        .context("sensory receive_high_water_mark must fit into i32")?;
    let sensory_linger = i32::try_from(config.zmq.streams.sensory.linger_ms)
        .context("sensory linger_ms must fit into i32")?;

    let registration_endpoint =
        format_tcp_endpoint(&config.agent.host, config.agent.registration_port);
    let sensory_endpoint = format_tcp_endpoint(&config.zmq.host, config.ports.zmq_sensory_port);
    let motor_endpoint = format_tcp_endpoint(&config.zmq.host, config.ports.zmq_motor_port);
    let viz_endpoint = format_tcp_endpoint(&config.zmq.host, config.ports.zmq_visualization_port);
    let control_endpoint = format_tcp_endpoint(&config.zmq.host, config.ports.zmq_rest_port);

    Ok(
        AgentConfig::new(settings.agent_id.clone(), AgentType::Sensory)
            .with_vision_unit(
                "segmented-vision",
                (width as usize, height as usize),
                3,
                SensoryUnit::SegmentedVision,
                settings.cortical_unit_id,
            )
            .with_registration_endpoint(registration_endpoint)
            .with_sensory_endpoint(sensory_endpoint)
            .with_motor_endpoint(motor_endpoint)
            .with_visualization_endpoint(viz_endpoint)
            .with_control_endpoint(control_endpoint)
            .with_sensory_socket_config(
                sensory_hwm,
                sensory_linger,
                config.zmq.streams.sensory.immediate,
            )
            .with_heartbeat_interval(config.zmq.client_heartbeat_timeout as f64 / 1000.0)
            .with_connection_timeout_ms(connect_timeout_ms)
            .with_registration_retries(registration_retries),
    )
}

/// Register the vision device for this example using the SDK connector cache.
fn register_vision_device(
    connector: &mut ConnectorAgent,
    settings: &ExampleSettings,
    frame: &ImageFrame,
) -> Result<()> {
    let mut sensor_cache = connector.get_sensor_cache();
    let unit_index = CorticalUnitIndex::from(settings.cortical_unit_id);
    let channel_count = CorticalChannelCount::new(1).context("CorticalChannelCount must be > 0")?;
    let frame_change_handling = FrameChangeHandling::Absolute;
    let image_props = frame.get_image_frame_properties();
    let input_resolution = image_props.get_image_resolution();
    let segmented_resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
        input_resolution,
        input_resolution,
    );
    let segmented_props = SegmentedImageFrameProperties::new(
        segmented_resolutions,
        image_props.get_color_channel_layout(),
        image_props.get_color_channel_layout(),
        image_props.get_color_space(),
    );
    let gaze_x = Percentage::new_from_0_1(settings.gaze_x)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Invalid FEAGI_TEST_GAZE_X (expected 0.0-1.0)")?;
    let gaze_y = Percentage::new_from_0_1(settings.gaze_y)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Invalid FEAGI_TEST_GAZE_Y (expected 0.0-1.0)")?;
    let gaze_modulation = Percentage::new_from_0_1(settings.gaze_modulation)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Invalid FEAGI_TEST_GAZE_MODULATION (expected 0.0-1.0)")?;
    let initial_gaze = GazeProperties::new(Percentage2D::new(gaze_x, gaze_y), gaze_modulation);

    sensor_cache
        .segmented_vision_register(
            unit_index,
            channel_count,
            frame_change_handling,
            image_props,
            segmented_props,
            initial_gaze,
        )
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Failed to register segmented vision device")?;

    Ok(())
}

/// Build an AgentRegistrar for the FEAGI HTTP API.
fn build_registrar(config: &FeagiConfig) -> Result<AgentRegistrar> {
    let timeout = Duration::from_secs_f64(config.timeouts.service_startup);
    AgentRegistrar::new(FeagiApiConfig::new(
        config.api.host.clone(),
        config.api.port,
        timeout,
    ))
    .map_err(|e| anyhow::anyhow!("{e}"))
    .context("Failed to create AgentRegistrar")
}

fn run_example_next() {
    let config = load_feagi_config().expect("Failed to load FEAGI configuration");
    let _registration_endpoint =
        format_tcp_endpoint(&config.agent.host, config.agent.registration_port);
}

fn run_example() -> Result<()> {
    let config = load_feagi_config()?;
    let settings = load_example_settings()?;

    let frame_paths = load_frame_paths(&settings.frame_dir)?;
    let frames = load_frame_sequence(&frame_paths, &settings.color_space)?;
    let first_frame = frames
        .first()
        .context("Loaded frames list is empty after validation")?;

    let resolution = first_frame.get_xy_resolution();
    let agent_config =
        build_agent_config(&config, &settings, (resolution.width, resolution.height))?;

    let agent_descriptor = AgentDescriptor::try_from_base64(&settings.agent_id)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Invalid FEAGI_TEST_AGENT_ID_BASE64")?;
    let mut connector = ConnectorAgent::new_empty(agent_descriptor);
    register_vision_device(&mut connector, &settings, first_frame)?;

    let mut client = AgentClient::new(agent_config).context("Failed to create AgentClient")?;
    client.connect().context("Failed to connect to FEAGI")?;

    let registrar = build_registrar(&config)?;
    let device_registrations = connector
        .get_device_registration_json()
        .context("Failed to export device registrations")?;

    let runtime = Runtime::new().context("Failed to create Tokio runtime")?;
    runtime
        .block_on(registrar.sync_device_registrations(device_registrations, &settings.agent_id))
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Failed to sync device registrations to FEAGI")?;

    let frame_interval = Duration::from_secs_f64(config.neural.burst_engine_timestep);
    let reconnect_delay = Duration::from_millis(config.zmq.socket_connect_timeout);
    let retry_window_ms = config.timeouts.service_startup * 1000.0;
    let mut registration_retries =
        (retry_window_ms / config.zmq.socket_connect_timeout as f64).ceil() as u32;
    if registration_retries == 0 {
        registration_retries = 1;
    }
    let unit_index = CorticalUnitIndex::from(settings.cortical_unit_id);
    let channel_index = CorticalChannelIndex::from(0u32);
    for _ in 0..settings.frame_loops {
        for frame in frames.iter() {
            let encoded = {
                let mut sensor_cache = connector.get_sensor_cache();
                sensor_cache
                    .segmented_vision_write(unit_index, channel_index, frame.clone().into())
                    .map_err(|e| anyhow::anyhow!("{e}"))
                    .context("Failed to write segmented vision frame")?;
                sensor_cache
                    .encode_all_sensors_to_neurons(Instant::now())
                    .map_err(|e| anyhow::anyhow!("{e}"))
                    .context("Failed to encode sensors to neurons")?;
                sensor_cache
                    .encode_neurons_to_bytes()
                    .map_err(|e| anyhow::anyhow!("{e}"))
                    .context("Failed to encode neurons to bytes")?;
                sensor_cache
                    .get_feagi_byte_container()
                    .get_byte_ref()
                    .to_vec()
            };
            let mut sent = client
                .try_send_sensory_bytes(&encoded)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to send sensory bytes")?;
            if !sent {
                for _ in 0..registration_retries {
                    std::thread::sleep(reconnect_delay);
                    sent = client
                        .try_send_sensory_bytes(&encoded)
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .context("Failed to send sensory bytes")?;
                    if sent {
                        break;
                    }
                }
            }
            if !sent {
                return Err(anyhow::anyhow!(
                    "Sensory stream not connected after retries; frame dropped"
                ));
            }
            std::thread::sleep(frame_interval);
        }
    }

    Ok(())
}
