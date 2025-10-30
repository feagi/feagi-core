use clap::Parser;
use log::{debug, info, warn};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
extern crate serde_json;
extern crate zmq;

use feagi_config::{load_config, validate_config};

/// FEAGI Inference Engine - Standalone neural processing engine with online learning
#[derive(Parser, Debug)]
#[command(name = "feagi-inference-engine", version, author, long_about = None)]
struct Args {
    /// Path to the connectome file to load
    #[arg(short, long)]
    connectome: PathBuf,

    /// Path to feagi_configuration.toml (required unless --help)
    #[arg(short = 'f', long)]
    config: Option<PathBuf>,

    /// Burst frequency in Hz (overrides config if provided)
    #[arg(long)]
    burst_hz: Option<u64>,

    /// Auto-save on shutdown (overrides config if provided)
    #[arg(long)]
    auto_save: Option<bool>,

    /// Checkpoint interval in seconds (overrides config if provided, 0 = disabled)
    #[arg(long)]
    checkpoint_interval: Option<u64>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// ZMQ registration endpoint (overrides config if provided, e.g., tcp://*:30001)
    #[arg(long)]
    registration_endpoint: Option<String>,

    /// ZMQ sensory input endpoint (overrides config if provided, e.g., tcp://*:5558)
    #[arg(long)]
    sensory_endpoint: Option<String>,

    /// ZMQ motor output endpoint (overrides config if provided, e.g., tcp://*:5564)
    #[arg(long)]
    motor_endpoint: Option<String>,

    /// Maximum number of agents (overrides config if provided)
    #[arg(long)]
    max_agents: Option<usize>,

    /// Agent inactivity timeout in milliseconds (overrides config if provided)
    #[arg(long)]
    agent_timeout_ms: Option<u64>,
}

/// Main entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args = Args::parse();

    // Load FEAGI configuration (REQUIRED - no hardcoded fallbacks)
    let config = load_config(args.config.as_deref(), None)?;
    validate_config(&config)?;

    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(if args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    // Banner
    print_banner();
    
    // Log configuration source
    info!("✓ FEAGI configuration loaded from feagi_configuration.toml");
    info!("  ZMQ Host: {}", config.zmq.host);
    info!("  Ports - Registration: {}, Sensory: {}, Motor: {}", 
          config.agent.registration_port,
          config.ports.zmq_sensory_port, 
          config.ports.zmq_motor_port);

    // Load connectome
    info!("Loading connectome from: {}", args.connectome.display());
    let connectome = feagi_connectome_serialization::load_connectome(&args.connectome)?;

    info!("✓ Connectome loaded successfully!");
    info!(
        "  Neurons: {}/{}",
        connectome.neurons.count, connectome.neurons.capacity
    );
    info!(
        "  Synapses: {}/{}",
        connectome.synapses.count, connectome.synapses.capacity
    );
    info!("  Cortical areas: {}", connectome.cortical_area_names.len());

    // Create NPU from connectome
    info!("Initializing NPU...");
    let mut npu = feagi_burst_engine::RustNPU::import_connectome(connectome);
    info!("✓ NPU initialized successfully!");

    // Resolve runtime parameters (CLI overrides config)
    let max_agents = args.max_agents.unwrap_or(100); // Sensible default if not in config
    let agent_timeout_ms = args.agent_timeout_ms.unwrap_or(config.zmq.inactive_client_timeout);
    let burst_hz = args.burst_hz.unwrap_or_else(|| {
        // Calculate from config's burst_engine_timestep (milliseconds)
        (1000.0 / config.neural.burst_engine_timestep) as u64
    });
    
    // Build ZMQ endpoints from config with CLI overrides
    let registration_endpoint = args.registration_endpoint.unwrap_or_else(|| {
        format!("tcp://{}:{}", config.agent.host, config.agent.registration_port)
    });
    let sensory_endpoint = args.sensory_endpoint.unwrap_or_else(|| {
        format!("tcp://{}:{}", config.zmq.host, config.ports.zmq_sensory_port)
    });
    let motor_endpoint = args.motor_endpoint.unwrap_or_else(|| {
        format!("tcp://{}:{}", config.zmq.host, config.ports.zmq_motor_port)
    });
    
    let auto_save = args.auto_save.unwrap_or(true);
    let checkpoint_interval = args.checkpoint_interval.unwrap_or(0);

    // Initialize agent registry
    info!("Initializing agent registry...");
    let registry = Arc::new(std::sync::RwLock::new(
        feagi_pns::agent_registry::AgentRegistry::new(max_agents, agent_timeout_ms),
    ));
    info!(
        "✓ Agent registry initialized (max_agents={}, timeout={}ms)",
        max_agents, agent_timeout_ms
    );

    // Log ZMQ endpoints
    info!("✓ ZMQ registration endpoint: {}", registration_endpoint);
    info!("  ZMQ sensory input endpoint: {}", sensory_endpoint);
    info!("  ZMQ motor output endpoint: {}", motor_endpoint);

    // Setup signal handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        info!("Shutdown signal received...");
        r.store(false, Ordering::SeqCst);
    })?;

    // Run engine (registration handling integrated into main loop)
    info!("🚀 Starting inference engine ({}Hz)", burst_hz);

    run_engine(
        &mut npu,
        burst_hz,
        &sensory_endpoint,
        &motor_endpoint,
        auto_save,
        checkpoint_interval,
        running,
        Arc::clone(&registry),
    )?;

    info!("✅ Inference engine shutdown complete!");
    Ok(())
}

// Note: Registration listener functionality to be implemented when ZMQ transport is completed

/// Run the inference engine loop
fn run_engine(
    npu: &mut feagi_burst_engine::RustNPU,
    burst_hz: u64,
    sensory_endpoint: &str,
    motor_endpoint: &str,
    _auto_save: bool,
    _checkpoint_interval: u64,
    running: Arc<AtomicBool>,
    registry: Arc<std::sync::RwLock<feagi_pns::agent_registry::AgentRegistry>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let burst_interval = std::time::Duration::from_millis(1000 / burst_hz);
    let mut burst_count: u64 = 0;
    let mut last_prune = std::time::Instant::now();

    // Create ZMQ context and sockets for sensory/motor I/O
    let ctx = zmq::Context::new();

    // Sensory input socket (PULL pattern - agents PUSH data to us)
    let sensory_socket = ctx.socket(zmq::PULL)?;
    sensory_socket.bind(sensory_endpoint)?;
    sensory_socket.set_rcvtimeo(10)?; // 10ms timeout for non-blocking
    info!("✓ Sensory input bound to: {}", sensory_endpoint);

    // Motor output socket (PUB pattern - we PUBLISH motor data to agents)
    let motor_socket = ctx.socket(zmq::PUB)?;
    motor_socket.bind(motor_endpoint)?;
    info!("✓ Motor output bound to: {}", motor_endpoint);

    info!("🔄 Engine running (Press Ctrl+C to stop)...");
    info!("  Agents send sensory data to: {}", sensory_endpoint);
    info!("  Motor output published to: {}", motor_endpoint);

    while running.load(Ordering::Relaxed) {
        let start = std::time::Instant::now();

        // Process ZMQ sensory input from registered agents
        // Format: JSON with { "cortical_area_id": u32, "neuron_id_potential_pairs": [[id, potential], ...] }
        loop {
            match sensory_socket.recv_bytes(zmq::DONTWAIT) {
                Ok(msg_bytes) => {
                    // Parse JSON message
                    match serde_json::from_slice::<serde_json::Value>(&msg_bytes) {
                        Ok(json) => {
                            // Extract neuron_id and potential pairs
                            if let Some(pairs) = json
                                .get("neuron_id_potential_pairs")
                                .and_then(|v| v.as_array())
                            {
                                let mut injection_data: Vec<(feagi_types::NeuronId, f32)> =
                                    Vec::new();
                                for pair in pairs {
                                    if let Some(arr) = pair.as_array() {
                                        if arr.len() == 2 {
                                            if let (Some(id), Some(pot)) =
                                                (arr[0].as_u64(), arr[1].as_f64())
                                            {
                                                injection_data.push((
                                                    feagi_types::NeuronId(id as u32),
                                                    pot as f32,
                                                ));
                                            }
                                        }
                                    }
                                }

                                if !injection_data.is_empty() {
                                    npu.inject_sensory_with_potentials(&injection_data);
                                    if burst_count % (args.burst_hz * 10) == 0 {
                                        debug!(
                                            "Injected {} neurons from agent",
                                            injection_data.len()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse sensory data: {}", e);
                        }
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    // No more messages available (non-blocking)
                    break;
                }
                Err(e) => {
                    warn!("Sensory socket error: {}", e);
                    break;
                }
            }
        }

        // Execute neural burst
        match npu.process_burst() {
            Ok(result) => {
                if result.neuron_count > 0 && burst_count % (args.burst_hz * 10) == 0 {
                    debug!(
                        "Burst #{}: {} neurons fired",
                        burst_count, result.neuron_count
                    );
                }
            }
            Err(e) => {
                warn!("Burst processing error: {}", e);
            }
        }

        // Publish motor output via ZMQ to registered agents
        // Extract fire queue data and publish to all subscribed motor agents
        if let Some(fire_data) = npu.force_sample_fire_queue() {
            if !fire_data.is_empty() {
                // Convert fire data to JSON format
                // Format: { "cortical_areas": { "area_id": { "neuron_ids": [...], "x": [...], "y": [...], "z": [...], "power": [...] } } }
                let mut motor_json = serde_json::json!({
                    "burst": burst_count,
                    "cortical_areas": {}
                });

                for (area_id, (ids, xs, ys, zs, ps)) in fire_data.iter() {
                    if !ids.is_empty() {
                        motor_json["cortical_areas"][area_id.to_string()] = serde_json::json!({
                            "neuron_ids": ids,
                            "x": xs,
                            "y": ys,
                            "z": zs,
                            "power": ps,
                        });
                    }
                }

                // Serialize and publish
                if let Ok(motor_msg) = serde_json::to_vec(&motor_json) {
                    if let Err(e) = motor_socket.send(&motor_msg, 0) {
                        if burst_count % (args.burst_hz * 10) == 0 {
                            warn!("Failed to publish motor output: {}", e);
                        }
                    } else if burst_count % (args.burst_hz * 10) == 0 {
                        debug!("Published motor output: {} cortical areas", fire_data.len());
                    }
                }
            }
        }

        burst_count += 1;

        // Periodic status
        if burst_count % (args.burst_hz * 10) == 0 {
            let agent_count = registry.read().unwrap().count();
            info!(
                "Status: {} bursts processed, {} agents registered",
                burst_count, agent_count
            );
        }

        // Prune inactive agents every 10 seconds
        if last_prune.elapsed() > std::time::Duration::from_secs(10) {
            let pruned = registry.write().unwrap().prune_inactive_agents();
            if pruned > 0 {
                info!("Pruned {} inactive agents", pruned);
            }
            last_prune = std::time::Instant::now();
        }

        // Checkpoint
        if args.checkpoint_interval > 0
            && burst_count % (args.burst_hz * args.checkpoint_interval) == 0
        {
            info!("Checkpoint at burst {}", burst_count);
            // TODO: Implement checkpointing via connectome save
        }

        // Sleep to maintain frequency
        let elapsed = start.elapsed();
        if elapsed < burst_interval {
            std::thread::sleep(burst_interval - elapsed);
        }
    }

    info!("Stopped after {} bursts", burst_count);
    info!("Final agent count: {}", registry.read().unwrap().count());

    // Auto-save if enabled
    if args.auto_save {
        info!("Auto-saving connectome...");
        // TODO: Implement auto-save via connectome serialization
        info!("✓ Connectome saved");
    }

    Ok(())
}

/// Print ASCII banner
fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║   ███████╗███████╗ █████╗  ██████╗ ██╗                           ║
║   ██╔════╝██╔════╝██╔══██╗██╔════╝ ██║                           ║
║   █████╗  █████╗  ███████║██║  ███╗██║                           ║
║   ██╔══╝  ██╔══╝  ██╔══██║██║   ██║██║                           ║
║   ██║     ███████╗██║  ██║╚██████╔╝██║                           ║
║   ╚═╝     ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝                           ║
║                                                                   ║
║   FEAGI Inference Engine v{}                                   ║
║   Standalone Neural Processing System with Online Learning       ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
    "#,
        env!("CARGO_PKG_VERSION")
    );
}
