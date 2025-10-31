# Industrial-Grade Debugging Infrastructure for FEAGI-Core

**Status**: Proposal  
**Date**: 2025-10-31  
**Author**: FEAGI Architecture Team

## Executive Summary

This document provides a comprehensive proposal for implementing industrial-grade debuggability in FEAGI-Core. The current state shows fragmented logging (`log`, `tracing`, `env_logger`), inconsistent error handling, and minimal diagnostic capabilities. This proposal outlines a unified debugging infrastructure that enables rapid problem diagnosis, performance analysis, and production monitoring.

---

## Current State Analysis

### Logging Infrastructure (Inconsistent)

**Current State:**
- **feagi-api**: Uses `tracing` + `tracing-subscriber` ✅
- **feagi-services**: Uses `log` crate ❌
- **feagi-bdu**: Uses `log` crate ❌
- **feagi-burst-engine**: Minimal logging ❌
- **feagi-inference-engine**: Uses `env_logger` + `log` ❌

**Issues:**
1. **No unified logging standard** - Mix of `log` and `tracing` makes correlation difficult
2. **No structured logging** - Plain text logs without context fields
3. **No log aggregation** - No centralized collection for distributed deployments
4. **Inconsistent log levels** - Different crates use different verbosity strategies
5. **No correlation IDs** - Cannot trace requests across service boundaries
6. **No performance metrics** - No latency or throughput logging

### Error Handling (Partially Structured)

**Current State:**
- ✅ `thiserror` used consistently for error types
- ✅ Transport-agnostic error types in `feagi-services`
- ❌ **No error context propagation** - Errors lose stack traces
- ❌ **No error reporting** - No structured error reporting to external systems
- ❌ **No error aggregation** - Cannot identify error patterns

### Observability (Minimal)

**Current State:**
- ❌ **No metrics** - No Prometheus/Grafana integration
- ❌ **No distributed tracing** - No OpenTelemetry/Jaeger
- ❌ **No health checks** - Basic health endpoint exists but no detailed diagnostics
- ❌ **No performance profiling** - No built-in profiling tools
- ❌ **No runtime introspection** - Cannot inspect state at runtime

### Testing Infrastructure (Basic)

**Current State:**
- ✅ Unit tests exist
- ✅ Integration tests exist
- ❌ **No debugging test utilities** - Hard to reproduce production issues
- ❌ **No chaos testing** - No fault injection capabilities
- ❌ **No performance regression tests** - No automated performance benchmarks

---

## Recommended Architecture

### 1. Unified Logging with `tracing`

**Decision**: Standardize on `tracing` across all crates.

**Benefits:**
- Structured logging with key-value fields
- Automatic span propagation (correlation IDs)
- Zero-cost in release builds (compile-time filtering)
- Integration with OpenTelemetry
- Excellent async support

**Implementation:**

```rust
// All crates should use tracing, not log
use tracing::{info, warn, error, debug, trace, instrument};

#[instrument(skip(state), fields(cortical_area_id = %area_id))]
pub async fn get_cortical_area(
    State(state): State<ApiState>,
    Path(area_id): Path<String>,
) -> ApiResult<Json<CorticalArea>> {
    // Automatic span creation with context
    info!("Fetching cortical area");
    // ...
}
```

**Migration Path:**
1. Add `tracing` to all `Cargo.toml` files
2. Replace all `log::*!` macros with `tracing::*!`
3. Add `#[instrument]` attributes to key functions
4. Remove `log` and `env_logger` dependencies

---

### 2. Structured Error Context with `anyhow` + `thiserror`

**Decision**: Use `anyhow` for application errors, `thiserror` for API boundaries.

**Benefits:**
- Rich error context chains
- Automatic backtraces (with `RUST_BACKTRACE=1`)
- Error cause chains preserved
- Compatible with existing `thiserror` types

**Implementation:**

```rust
use anyhow::{Context, Result, bail};
use thiserror::Error;

// Service layer errors (API boundaries)
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Not found: {resource} with id '{id}'")]
    NotFound { resource: String, id: String },
    // ...
}

// Internal errors (with context)
pub async fn load_genome(path: &Path) -> Result<Genome> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read genome file: {}", path.display()))?;
    
    let genome = parse_genome(&content)
        .with_context(|| format!("Failed to parse genome from: {}", path.display()))?;
    
    Ok(genome)
}
```

**Error Reporting:**

```rust
// Structured error reporting
#[derive(Serialize)]
struct ErrorReport {
    error_type: String,
    message: String,
    context: HashMap<String, String>,
    backtrace: Option<String>,
    timestamp: DateTime<Utc>,
}

pub fn report_error(err: &anyhow::Error) -> ErrorReport {
    ErrorReport {
        error_type: err.to_string(),
        message: err.root_cause().to_string(),
        context: extract_context(err),
        backtrace: std::env::var("RUST_BACKTRACE").ok()
            .and_then(|_| Some(format!("{:?}", err))),
        timestamp: Utc::now(),
    }
}
```

---

### 3. Distributed Tracing with OpenTelemetry

**Decision**: Integrate OpenTelemetry for distributed tracing.

**Benefits:**
- Trace requests across service boundaries
- Identify performance bottlenecks
- Visualize request flows
- Integration with Jaeger, Zipkin, etc.

**Implementation:**

```rust
// Add to Cargo.toml
[dependencies]
opentelemetry = "0.21"
opentelemetry_sdk = "0.21"
tracing-opentelemetry = "0.21"
opentelemetry-otlp = "0.14"

// Initialize in main.rs
use opentelemetry::global;
use opentelemetry_sdk::{trace::TracerProvider, Resource};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

fn init_tracing() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://jaeger:4317")
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config().with_resource(
                Resource::new(vec![
                    KeyValue::new("service.name", "feagi-core"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])
            )
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to create OTLP tracer");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}
```

---

### 4. Metrics with Prometheus

**Decision**: Add Prometheus metrics for observability.

**Metrics to Track:**
- **Burst Engine**: Burst count, burst duration, neurons fired, synapses activated
- **API**: Request count, request latency, error rate (by endpoint)
- **Genome Loading**: Load time, genome size, cortical areas created
- **BDU**: Synaptogenesis time, neurons created, synapses created
- **System**: Memory usage, CPU usage, thread count

**Implementation:**

```rust
// Add to Cargo.toml
[dependencies]
prometheus = "0.13"

// Metrics definitions
use prometheus::{Counter, Histogram, Gauge, Registry};

lazy_static! {
    pub static ref BURST_COUNT: Counter = Counter::new(
        "feagi_burst_total",
        "Total number of bursts executed"
    ).unwrap();
    
    pub static ref BURST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("feagi_burst_duration_seconds", "Burst execution time")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
    ).unwrap();
    
    pub static ref API_REQUESTS: Counter = Counter::new(
        "feagi_api_requests_total",
        "Total API requests"
    ).with_label_values(&["endpoint", "method", "status"])
    .unwrap();
    
    pub static ref API_LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new("feagi_api_request_duration_seconds", "API request latency")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
    ).unwrap();
    
    pub static ref CONNECTOME_SIZE: Gauge = Gauge::new(
        "feagi_connectome_neurons",
        "Number of neurons in connectome"
    ).unwrap();
    
    pub static ref CONNECTOME_SYNAPSES: Gauge = Gauge::new(
        "feagi_connectome_synapses",
        "Number of synapses in connectome"
    ).unwrap();
}

// Instrumentation
#[instrument]
pub async fn execute_burst() {
    let _timer = BURST_DURATION.start_timer();
    BURST_COUNT.inc();
    // ... burst logic
}

// Expose metrics endpoint
pub fn create_metrics_router() -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
}

async fn metrics_handler() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

---

### 5. Runtime Diagnostics API

**Decision**: Add `/v1/debug/*` endpoints for runtime introspection.

**Endpoints:**
- `GET /v1/debug/state` - Current system state (burst count, neurons, synapses)
- `GET /v1/debug/health` - Detailed health check (memory, threads, locks)
- `GET /v1/debug/trace/{trace_id}` - Retrieve trace by ID
- `GET /v1/debug/metrics` - Prometheus metrics
- `GET /v1/debug/logs?level=error&limit=100` - Recent logs (filtered)
- `POST /v1/debug/panic-test` - Trigger panic for testing (dev only)

**Implementation:**

```rust
// Add to feagi-api/src/endpoints/debug.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct DebugState {
    pub burst_count: u64,
    pub neurons: usize,
    pub synapses: usize,
    pub cortical_areas: usize,
    pub memory_usage_mb: f64,
    pub thread_count: usize,
    pub active_agents: usize,
}

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub checks: Vec<HealthCheck>,
}

#[derive(Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: String,
}

/// GET /v1/debug/state
#[utoipa::path(get, path = "/v1/debug/state", tag = "debug")]
pub async fn get_debug_state(
    State(state): State<ApiState>,
) -> ApiResult<Json<DebugState>> {
    let connectome = state.connectome_service.as_ref();
    let runtime = state.runtime_service.as_ref();
    
    let stats = connectome.get_statistics().await?;
    let runtime_status = runtime.get_status().await?;
    
    Ok(Json(DebugState {
        burst_count: runtime_status.burst_count,
        neurons: stats.neuron_count,
        synapses: stats.synapse_count,
        cortical_areas: stats.cortical_area_count,
        memory_usage_mb: get_memory_usage(),
        thread_count: get_thread_count(),
        active_agents: state.agent_service.get_active_count().await?,
    }))
}

/// GET /v1/debug/health
#[utoipa::path(get, path = "/v1/debug/health", tag = "debug")]
pub async fn get_debug_health(
    State(state): State<ApiState>,
) -> ApiResult<Json<HealthStatus>> {
    let mut checks = Vec::new();
    
    // Check connectome
    match state.connectome_service.get_statistics().await {
        Ok(_) => checks.push(HealthCheck {
            name: "connectome".to_string(),
            status: "healthy".to_string(),
            message: "Connectome accessible".to_string(),
        }),
        Err(e) => checks.push(HealthCheck {
            name: "connectome".to_string(),
            status: "unhealthy".to_string(),
            message: e.to_string(),
        }),
    }
    
    // Check runtime
    match state.runtime_service.get_status().await {
        Ok(_) => checks.push(HealthCheck {
            name: "runtime".to_string(),
            status: "healthy".to_string(),
            message: "Runtime operational".to_string(),
        }),
        Err(e) => checks.push(HealthCheck {
            name: "runtime".to_string(),
            status: "unhealthy".to_string(),
            message: e.to_string(),
        }),
    }
    
    let overall_status = if checks.iter().all(|c| c.status == "healthy") {
        "healthy"
    } else {
        "degraded"
    };
    
    Ok(Json(HealthStatus {
        status: overall_status.to_string(),
        checks,
    }))
}
```

---

### 6. Performance Profiling Integration

**Decision**: Integrate `tracing` with `tracing-chrome` for Chrome DevTools profiling.

**Benefits:**
- Visualize function execution timelines
- Identify hot paths
- Memory allocation tracking
- CPU flame graphs

**Implementation:**

```rust
// Add to Cargo.toml
[dependencies]
tracing-chrome = "0.6"

// Initialize in main.rs
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::prelude::*;

fn init_profiling() {
    let (chrome_layer, guard) = ChromeLayerBuilder::new()
        .file("trace.json")
        .build();
    
    tracing_subscriber::registry()
        .with(chrome_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Keep guard alive for entire program lifetime
    std::mem::forget(guard);
}

// Usage: Open trace.json in Chrome DevTools (chrome://tracing)
```

**Alternative: `perf` on Linux**

```bash
# Build with debug symbols
cargo build --release

# Profile
perf record --call-graph dwarf ./target/release/feagi
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

---

### 7. Debugging Test Utilities

**Decision**: Create debugging utilities for tests.

**Implementation:**

```rust
// Add to feagi-core/crates/feagi-test-utils/src/lib.rs
pub mod debug;

pub mod debug {
    use std::sync::Once;
    use tracing_subscriber::{EnvFilter, fmt};
    
    static INIT: Once = Once::new();
    
    /// Initialize tracing for tests
    pub fn init_test_logging() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .with_test_writer()
                .init();
        });
    }
    
    /// Capture logs during test execution
    pub struct LogCapture {
        // Implementation to capture logs
    }
    
    /// Assert error message contains substring
    pub fn assert_error_contains(err: &anyhow::Error, substring: &str) {
        assert!(
            err.to_string().contains(substring),
            "Error message '{}' does not contain '{}'",
            err,
            substring
        );
    }
    
    /// Wait for condition with timeout
    pub async fn wait_for<F>(mut condition: F, timeout: Duration) -> Result<()>
    where
        F: FnMut() -> bool,
    {
        let start = Instant::now();
        while !condition() {
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for condition"));
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
```

---

### 8. Log Aggregation Configuration

**Decision**: Provide structured logging configuration for production.

**Output Formats:**
- **Development**: Human-readable (`tracing_subscriber::fmt`)
- **Production**: JSON (`tracing_subscriber::fmt::json`)
- **Container**: Docker-friendly (single-line JSON)

**Implementation:**

```rust
// Add to feagi-core/crates/feagi-config/src/logging.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String, // "trace", "debug", "info", "warn", "error"
    pub format: LogFormat, // "text" or "json"
    pub output: LogOutput, // "stdout", "file", "syslog"
    pub file_path: Option<String>,
    pub enable_tracing: bool, // Enable OpenTelemetry
    pub tracing_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File(String),
    Syslog,
}

pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    let fmt_layer = match config.format {
        LogFormat::Text => tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_line_number(true)
            .boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_line_number(true)
            .boxed(),
    };
    
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);
    
    if config.enable_tracing {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(config.tracing_endpoint.as_ref().unwrap())
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        
        registry.with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
    } else {
        registry.init();
    }
    
    Ok(())
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
1. ✅ **Standardize on `tracing`**
   - Replace all `log` usage with `tracing`
   - Add `#[instrument]` to key functions
   - Configure `tracing-subscriber` consistently

2. ✅ **Error Context**
   - Migrate to `anyhow` for application errors
   - Keep `thiserror` for API boundaries
   - Add error context chains

3. ✅ **Structured Logging**
   - Configure JSON logging for production
   - Add correlation IDs to spans
   - Implement log level filtering

### Phase 2: Observability (Week 3-4)
4. ✅ **Metrics**
   - Add Prometheus metrics
   - Instrument burst engine, API, BDU
   - Expose `/metrics` endpoint

5. ✅ **Distributed Tracing**
   - Integrate OpenTelemetry
   - Configure Jaeger exporter
   - Add trace IDs to logs

6. ✅ **Health Checks**
   - Implement `/v1/debug/health`
   - Add component health checks
   - Expose system metrics

### Phase 3: Advanced Debugging (Week 5-6)
7. ✅ **Runtime Diagnostics**
   - Implement `/v1/debug/*` endpoints
   - Add state introspection
   - Add log retrieval API

8. ✅ **Performance Profiling**
   - Integrate `tracing-chrome`
   - Add CPU profiling support
   - Document profiling workflow

9. ✅ **Testing Utilities**
   - Create `feagi-test-utils` crate
   - Add debugging test helpers
   - Document test debugging patterns

### Phase 4: Production Hardening (Week 7-8)
10. ✅ **Log Aggregation**
    - Configure JSON logging
    - Add Docker-friendly logging
    - Document log shipping (ELK, Loki)

11. ✅ **Error Reporting**
    - Add structured error reporting
    - Integrate with Sentry (optional)
    - Add error aggregation

12. ✅ **Documentation**
    - Write debugging guide
    - Document metrics and traces
    - Create troubleshooting runbook

---

## Configuration Example

**`feagi_configuration.toml`:**

```toml
[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # text or json
output = "stdout"  # stdout, file, syslog
file_path = "/var/log/feagi/feagi.log"  # if output = "file"

[tracing]
enabled = true
endpoint = "http://jaeger:4317"
service_name = "feagi-core"

[metrics]
enabled = true
endpoint = "/metrics"
port = 9090  # Separate metrics port (optional)

[debug]
enabled = true  # Enable /v1/debug/* endpoints
log_retention_hours = 24
```

---

## Usage Examples

### 1. Structured Logging

```rust
use tracing::{info, instrument, Span};

#[instrument(skip(self), fields(cortical_area_id = %area_id))]
pub async fn get_cortical_area(&self, area_id: String) -> Result<CorticalArea> {
    info!("Fetching cortical area");
    // Automatic span with cortical_area_id field
    // Logs will include: [cortical_area_id="v1"]
}
```

### 2. Error Context

```rust
use anyhow::{Context, Result};

pub async fn load_genome(path: &Path) -> Result<Genome> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Reading genome: {}", path.display()))?;
    
    parse_genome(&content)
        .with_context(|| format!("Parsing genome: {}", path.display()))?;
}
```

### 3. Metrics

```rust
use crate::metrics::*;

pub async fn execute_burst() {
    let _timer = BURST_DURATION.start_timer();
    BURST_COUNT.inc();
    // ... burst logic
}
```

### 4. Debugging in Tests

```rust
use feagi_test_utils::debug::*;

#[tokio::test]
async fn test_genome_loading() {
    init_test_logging();
    
    let result = load_genome(Path::new("test.genome"))
        .context("Failed to load test genome");
    
    assert_error_contains(&result.unwrap_err(), "Invalid genome");
}
```

---

## Dependencies Summary

### New Dependencies (to add)

```toml
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.21"

# Errors
anyhow = "1.0"
thiserror = "1.0"  # Already present

# Metrics
prometheus = "0.13"

# Tracing
opentelemetry = "0.21"
opentelemetry-sdk = "0.21"
opentelemetry-otlp = "0.14"

# Profiling
tracing-chrome = "0.6"  # Optional, dev only
```

### Dependencies to Remove

```toml
# Remove these (replace with tracing)
log = "0.4"  # Remove
env_logger = "0.11"  # Remove
```

---

## Success Metrics

**Debugging Speed:**
- **Before**: 2-4 hours to diagnose production issues
- **After**: 15-30 minutes with structured logs and traces

**Error Detection:**
- **Before**: Errors discovered by users
- **After**: Errors detected via metrics/alerts within minutes

**Performance Analysis:**
- **Before**: Manual profiling, unclear bottlenecks
- **After**: Automated metrics, visual traces, CPU profiling

**Test Debugging:**
- **Before**: Print debugging, unclear failure causes
- **After**: Structured test logs, error context chains

---

## Next Steps

1. **Review and approve** this proposal
2. **Prioritize phases** based on current pain points
3. **Assign implementation** to team members
4. **Create tracking issues** for each phase
5. **Begin Phase 1** implementation

---

## References

- [tracing documentation](https://docs.rs/tracing/)
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
- [Prometheus Rust Client](https://docs.rs/prometheus/)
- [anyhow documentation](https://docs.rs/anyhow/)
- [Rust Performance Book - Profiling](https://nnethercote.github.io/perf-book/profiling.html)

