//! FEAGI HTTP API Server Example
//!
//! This example demonstrates how to start a fully functional FEAGI HTTP API server
//! with all services wired together.
//!
//! Run with: cargo run --example http_api_server --package feagi-api

use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_bdu::ConnectomeManager;
use feagi_burst_engine::{BurstLoopRunner, RustNPU};
use feagi_services::*;
use parking_lot::{Mutex as ParkingLotMutex, RwLock};
use std::sync::{Arc, Mutex as StdMutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

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

    let connectome_service = Arc::new(ConnectomeServiceImpl::new(connectome.clone()))
        as Arc<dyn ConnectomeService + Send + Sync>;

    let neuron_service = Arc::new(NeuronServiceImpl::new(connectome.clone()))
        as Arc<dyn NeuronService + Send + Sync>;

    let analytics_service = Arc::new(AnalyticsServiceImpl::new(
        connectome.clone(),
        None, // No burst runner for this demo
    )) as Arc<dyn AnalyticsService + Send + Sync>;

    // Create RuntimeService with a simple BurstLoopRunner
    // (For full functionality, configure with actual NPU and visualization publisher)
    let npu_for_runtime = Arc::new(StdMutex::new(RustNPU::new(10, 10, 10))); // Minimal NPU
    let burst_loop = BurstLoopRunner::new::<()>(npu_for_runtime, None, 30.0); // No viz publisher
    let burst_runner_for_runtime = Arc::new(ParkingLotMutex::new(burst_loop));

    let runtime_service = Arc::new(RuntimeServiceImpl::new(burst_runner_for_runtime))
        as Arc<dyn RuntimeService + Send + Sync>;

    println!("✅ Service layer created:");
    println!("   - GenomeService");
    println!("   - ConnectomeService");
    println!("   - NeuronService");
    println!("   - AnalyticsService");
    println!("   - RuntimeService\n");

    // ========================================================================
    // STEP 3: Create API State
    // ========================================================================

    println!("🌐 Creating API state...");

    let api_state = ApiState {
        analytics_service,
        connectome_service,
        genome_service,
        neuron_service,
        runtime_service,
    };

    println!("✅ API state created\n");

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
    println!("║  HTTP API:       http://{}                 ║", bind_address);
    println!("║  Swagger UI:     http://{}/swagger-ui/      ║", bind_address);
    println!("║  OpenAPI Spec:   http://{}/openapi.json    ║", bind_address);
    println!("║                                                           ║");
    println!("║  Available Endpoints:                                     ║");
    println!("║    - GET  /health                                         ║");
    println!("║    - GET  /api/v1/genome                                  ║");
    println!("║    - POST /api/v1/genome/load                             ║");
    println!("║    - GET  /api/v1/cortical-areas                          ║");
    println!("║    - GET  /api/v1/neurons                                 ║");
    println!("║    - GET  /api/v1/runtime/status                          ║");
    println!("║    - GET  /api/v1/analytics/health                        ║");
    println!("║    ...and 50+ more endpoints                              ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Start server
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

