// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! FEAGI HTTP API Server Example
//!
//! This example demonstrates how to start a fully functional FEAGI HTTP API server
//! with all services wired together.
//!
//! Run with: cargo run --example http_api_server --package feagi-api

use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::{BurstLoopRunner, RustNPU};
use feagi_observability::{init_logging, parse_debug_flags};
use feagi_services::SystemServiceImpl;
use feagi_services::*;
use parking_lot::{Mutex as ParkingLotMutex, RwLock};
use std::sync::{Arc, Mutex as StdMutex};

#[derive(Debug, Default)]
struct NpuTraceArgs {
    enabled: bool,
    synapse: bool,
    dynamics: bool,
    src: Option<String>,
    dst: Option<String>,
    neuron: Option<String>,
}

fn parse_npu_trace_args() -> NpuTraceArgs {
    let mut out = NpuTraceArgs::default();
    for arg in std::env::args() {
        if arg == "--npu-trace" {
            out.enabled = true;
            out.synapse = true;
            out.dynamics = true;
            continue;
        }
        if arg == "--npu-trace-synapse" {
            out.enabled = true;
            out.synapse = true;
            continue;
        }
        if arg == "--npu-trace-dynamics" {
            out.enabled = true;
            out.dynamics = true;
            continue;
        }
        if let Some(v) = arg.strip_prefix("--npu-trace-src=") {
            out.enabled = true;
            out.src = Some(v.to_string());
            continue;
        }
        if let Some(v) = arg.strip_prefix("--npu-trace-dst=") {
            out.enabled = true;
            out.dst = Some(v.to_string());
            continue;
        }
        if let Some(v) = arg.strip_prefix("--npu-trace-neuron=") {
            out.enabled = true;
            out.neuron = Some(v.to_string());
            continue;
        }
    }
    out
}

fn apply_npu_trace_env(args: &NpuTraceArgs) {
    if !args.enabled {
        return;
    }

    // Gate the trace emitters (read once at runtime via OnceLock in burst-engine).
    if args.synapse {
        std::env::set_var("FEAGI_NPU_TRACE_SYNAPSE", "1");
    }
    if args.dynamics {
        std::env::set_var("FEAGI_NPU_TRACE_DYNAMICS", "1");
    }
    if let Some(v) = &args.src {
        std::env::set_var("FEAGI_NPU_TRACE_SRC", v);
    }
    if let Some(v) = &args.dst {
        std::env::set_var("FEAGI_NPU_TRACE_DST", v);
    }
    if let Some(v) = &args.neuron {
        std::env::set_var("FEAGI_NPU_TRACE_NEURON", v);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional NPU trace gating + filters (single-switch debugging)
    let npu_trace = parse_npu_trace_args();
    apply_npu_trace_env(&npu_trace);

    // Initialize logging via FEAGI observability (supports --debug-* and FEAGI_DEBUG)
    let mut debug_flags = parse_debug_flags();

    // If NPU trace was requested, ensure the feagi-npu-trace target is enabled at debug level.
    // This works with the existing debug flag system even though it’s a tracing target, not a crate.
    if npu_trace.enabled {
        debug_flags
            .enabled_crates
            .insert("feagi-npu-trace".to_string(), true);
    }

    // Keep guard alive for duration of process.
    let _logging_guard = init_logging(&debug_flags, None, None, None)?;

    println!("🚀 FEAGI HTTP API Server - Starting...\n");

    // ========================================================================
    // STEP 1: Initialize Core Components
    // ========================================================================

    println!("📦 Initializing core components...");

    // Get ConnectomeManager singleton
    let connectome = ConnectomeManager::instance();

    // Note: In a real deployment, you would:
    // 1. Create and attach an NPU to the ConnectomeManager
    // 2. Create a BurstLoopRunner for runtime control
    // 3. Load a genome to populate the connectome
    //
    // For this demo, we're showing the API structure is working

    println!("✅ Core components initialized\n");

    // ========================================================================
    // STEP 2: Create Service Layer
    // ========================================================================

    println!("🔧 Creating service layer...");

    let genome_service = Arc::new(GenomeServiceImpl::new(connectome.clone()))
        as Arc<dyn GenomeService + Send + Sync>;

    // Cast to GenomeServiceImpl to access get_current_genome_arc()
    let genome_service_impl = Arc::new(GenomeServiceImpl::new(connectome.clone()));
    let current_genome = genome_service_impl.get_current_genome_arc();
    let genome_service = genome_service_impl as Arc<dyn GenomeService + Send + Sync>;

    let connectome_service = Arc::new(ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone()))
        as Arc<dyn ConnectomeService + Send + Sync>;

    let neuron_service = Arc::new(NeuronServiceImpl::new(connectome.clone()))
        as Arc<dyn NeuronService + Send + Sync>;

    let analytics_service = Arc::new(AnalyticsServiceImpl::new(
        connectome.clone(),
        None, // No burst runner for this demo
    )) as Arc<dyn AnalyticsService + Send + Sync>;

    // Create RuntimeService with a simple BurstLoopRunner
    // (For full functionality, configure with actual NPU and visualization publisher)

    // Dummy publishers for testing
    struct DummyViz;
    impl feagi_npu_burst_engine::VisualizationPublisher for DummyViz {
        fn publish_visualization(&self, _data: &[u8]) -> Result<(), String> {
            Ok(())
        }
    }
    struct DummyMotor;
    impl feagi_npu_burst_engine::MotorPublisher for DummyMotor {
        fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
            Ok(())
        }
    }

    use feagi_npu_burst_engine::backend::CPUBackend;
    use feagi_npu_burst_engine::DynamicNPU;
    use feagi_npu_runtime::StdRuntime;

    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu_result = RustNPU::new(runtime, backend, 10, 10, 10).expect("Failed to create NPU");
    let npu_for_runtime = Arc::new(StdMutex::new(DynamicNPU::F32(npu_result))); // Minimal NPU
    let burst_loop =
        BurstLoopRunner::new::<DummyViz, DummyMotor>(npu_for_runtime, None, None, 30.0); // No publishers
    let burst_runner_for_runtime = Arc::new(ParkingLotMutex::new(burst_loop));

    let runtime_service = Arc::new(RuntimeServiceImpl::new(burst_runner_for_runtime))
        as Arc<dyn RuntimeService + Send + Sync>;

    // For examples, create basic version info
    let mut version_info = feagi_services::types::VersionInfo::default();
    version_info
        .crates
        .insert("example".to_string(), "1.0.0".to_string());
    version_info.rust_version = "1.75".to_string();
    version_info.build_timestamp = "example build".to_string();

    let system_service = Arc::new(SystemServiceImpl::new(
        connectome.clone(),
        Some(burst_runner_for_runtime.clone()),
        version_info,
    )) as Arc<dyn SystemService + Send + Sync>;

    println!("✅ Service layer created:");
    println!("   - GenomeService");
    println!("   - ConnectomeService");
    println!("   - NeuronService");
    println!("   - AnalyticsService");
    println!("   - RuntimeService");
    println!("   - SystemService\n");

    // ========================================================================
    // STEP 3: Create API State
    // ========================================================================

    println!("🌐 Creating API state...");

    // Get FEAGI session timestamp (when this instance started)
    let feagi_session_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let api_state = ApiState {
        agent_service: None,
        analytics_service,
        connectome_service,
        genome_service,
        neuron_service,
        runtime_service,
        system_service,
        snapshot_service: None,
        feagi_session_timestamp,
    };

    println!(
        "✅ API state created (FEAGI session: {})\n",
        feagi_session_timestamp
    );

    // ========================================================================
    // STEP 4: Create and Start HTTP Server
    // ========================================================================

    let bind_address = "127.0.0.1:8000";
    println!("🌐 Starting HTTP server on {}...", bind_address);

    let app = create_http_server(api_state);

    println!("✅ HTTP server configured\n");
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                   FEAGI API SERVER READY                  ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║                                                           ║");
    println!(
        "║  HTTP API:       http://{}                 ║",
        bind_address
    );
    println!(
        "║  Swagger UI:     http://{}/swagger-ui/      ║",
        bind_address
    );
    println!(
        "║  OpenAPI Spec:   http://{}/openapi.json    ║",
        bind_address
    );
    println!("║                                                           ║");
    println!("║  Available Endpoints:                                     ║");
    println!("║    - GET  /health                                         ║");
    println!("║    - GET  /v1/genome/file_name                            ║");
    println!("║    - POST /v1/genome/upload/barebones                    ║");
    println!("║    - GET  /v1/cortical_area/ipu                           ║");
    println!("║    - GET  /v1/cortical_area/cortical_area_id_list         ║");
    println!("║    - GET  /v1/system/health_check                         ║");
    println!("║    - GET  /v1/system/readiness_check                      ║");
    println!("║    ...and 50+ more endpoints                              ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Start server
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
