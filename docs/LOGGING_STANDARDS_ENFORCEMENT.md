# Logging and Debugging Standards Enforcement

**Status**: Implementation Guide  
**Date**: 2025-10-31  
**Author**: FEAGI Architecture Team

## Problem Statement

With 15+ crates in the feagi-core workspace, ensuring consistent logging, error handling, and debugging patterns across all crates is critical. Without enforcement mechanisms, crates will drift toward inconsistent patterns, making debugging difficult and observability incomplete.

---

## Solution: Unified Observability Infrastructure

### Architecture: Profiling, Logging, and Telemetry Together

```
┌─────────────────────────────────────────────────────────┐
│  feagi-observability (unified observability crate)      │
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  LOGGING                                          │  │
│  │  - Structured logging macros                     │  │
│  │  - Span propagation (correlation IDs)            │  │
│  │  - Context fields                                 │  │
│  └──────────────────────────────────────────────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  TELEMETRY                                        │  │
│  │  - Metrics (Prometheus)                          │  │
│  │  - Traces (OpenTelemetry)                        │  │
│  │  - Health checks                                 │  │
│  │  - System metrics                                │  │
│  └──────────────────────────────────────────────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  PROFILING                                       │  │
│  │  - CPU profiling (tracing-chrome, perf)        │  │
│  │  - Memory profiling                             │  │
│  │  - Flamegraph generation                        │  │
│  │  - Performance counters                         │  │
│  └──────────────────────────────────────────────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  SHARED INFRASTRUCTURE                          │  │
│  │  - Unified initialization                        │  │
│  │  - Correlation IDs                              │  │
│  │  - Context propagation                          │  │
│  │  - Error reporting                              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                         ↓
        ┌────────────────┴────────────────┐
        ↓                                  ↓
┌─────────────────┐              ┌─────────────────┐
│  feagi-api      │              │  feagi-brain-development      │
│  feagi-services │              │  feagi-burst-...│
│  ...            │              │  ...            │
└─────────────────┘              └─────────────────┘
```

**Why Unified?**
- ✅ **Shared Context**: Logs, traces, metrics, and profiles all use same correlation IDs
- ✅ **Unified Initialization**: One call to initialize all observability
- ✅ **Consistent Patterns**: Same APIs for all observability needs
- ✅ **Better Correlation**: Can correlate logs with traces with profiles
- ✅ **Simplified Maintenance**: Single crate to update
- ✅ **Performance**: Shared infrastructure reduces overhead

---

## Implementation Plan

### Step 1: Create `feagi-observability` Crate

**Location**: `feagi-core/crates/feagi-observability/`

**Purpose**: Unified observability infrastructure for logging, telemetry (metrics/traces), and profiling.

```rust
// feagi-observability/src/lib.rs

// Core observability modules
pub mod logging;      // Structured logging with spans
pub mod errors;       // Error handling and reporting
pub mod metrics;      // Prometheus metrics
pub mod tracing;      // Distributed tracing (OpenTelemetry)
pub mod profiling;   // CPU/Memory profiling
pub mod telemetry;   // Unified telemetry collection
pub mod context;     // Correlation IDs and context propagation
pub mod debug;        // Debugging utilities

// Unified initialization
pub mod init;

// Re-export commonly used items
pub use logging::*;
pub use errors::*;
pub use metrics::*;
pub use tracing::*;
pub use profiling::*;
pub use telemetry::*;
pub use context::*;
pub use init::*;
```

---

### Step 2: Standardized Logging Macros

**Location**: `feagi-observability/src/logging.rs`

```rust
//! Standardized logging macros for FEAGI
//!
//! All crates MUST use these macros instead of direct `tracing::*!` calls.
//! This ensures consistent field names, log levels, and structured logging.

use tracing::{info, warn, error, debug, trace};

/// Log a burst execution event
///
/// # Example
/// ```rust
/// feagi_log::burst_info!(
///     burst_id = 42,
///     neurons_fired = 1000,
///     synapses_activated = 5000,
///     duration_ms = 15.2
/// );
/// ```
#[macro_export]
macro_rules! burst_info {
    ($($field:ident = $value:expr),* $(,)?) => {
        tracing::info!(
            target: "feagi.burst",
            $($field = $value),*,
            "Burst executed"
        );
    };
}

/// Log a genome loading event
#[macro_export]
macro_rules! genome_info {
    ($($field:ident = $value:expr),* $(,)?) => {
        tracing::info!(
            target: "feagi.genome",
            $($field = $value),*,
            "Genome operation"
        );
    };
}

/// Log an API request
#[macro_export]
macro_rules! api_info {
    ($method:expr, $path:expr, $($field:ident = $value:expr),* $(,)?) => {
        tracing::info!(
            target: "feagi.api",
            method = $method,
            path = $path,
            $($field = $value),*,
            "API request"
        );
    };
}

/// Log an error with context
#[macro_export]
macro_rules! feagi_error {
    ($($field:ident = $value:expr),*; $msg:expr) => {
        tracing::error!(
            target: "feagi.error",
            $($field = $value),*,
            $msg
        );
    };
}

/// Log a warning with context
#[macro_export]
macro_rules! feagi_warn {
    ($($field:ident = $value:expr),*; $msg:expr) => {
        tracing::warn!(
            target: "feagi.warn",
            $($field = $value),*,
            $msg
        );
    };
}

/// Log debug information (only in debug builds)
#[macro_export]
macro_rules! feagi_debug {
    ($($field:ident = $value:expr),*; $msg:expr) => {
        #[cfg(debug_assertions)]
        tracing::debug!(
            target: "feagi.debug",
            $($field = $value),*,
            $msg
        );
    };
}

// Re-export standard tracing macros for convenience
pub use tracing::{info, warn, error, debug, trace, instrument, Instrument};
```

---

### Step 3: Standardized Error Types

**Location**: `feagi-observability/src/errors.rs`

```rust
//! Standardized error types for FEAGI
//!
//! All crates should use these error types or convert to them.
//! This ensures consistent error handling and reporting.

use anyhow::{Context, Result as AnyhowResult};
use thiserror::Error;

/// Base error type for FEAGI operations
#[derive(Error, Debug)]
pub enum FeagiObservabilityError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Helper trait for adding context to errors
pub trait FeagiErrorContext {
    fn with_feagi_context(self, context: &str) -> anyhow::Error;
}

impl<T> FeagiErrorContext for T
where
    T: Into<anyhow::Error>,
{
    fn with_feagi_context(self, context: &str) -> anyhow::Error {
        self.into().context(context.to_string())
    }
}

/// Standard result type for FEAGI operations
pub type FeagiResult<T> = AnyhowResult<T>;

/// Convert any error to a structured error report
pub fn error_report(err: &anyhow::Error) -> ErrorReport {
    ErrorReport {
        error_type: err.to_string(),
        message: err.root_cause().to_string(),
        context: extract_error_context(err),
        backtrace: std::env::var("RUST_BACKTRACE").ok()
            .and_then(|_| Some(format!("{:?}", err))),
        timestamp: chrono::Utc::now(),
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorReport {
    pub error_type: String,
    pub message: String,
    pub context: Vec<String>,
    pub backtrace: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

fn extract_error_context(err: &anyhow::Error) -> Vec<String> {
    let mut contexts = Vec::new();
    let mut source = Some(err);
    
    while let Some(e) = source {
        contexts.push(e.to_string());
        source = e.source();
    }
    
    contexts
}
```

---

### Step 4: Standardized Metrics

**Location**: `feagi-observability/src/metrics.rs`

```rust
//! Standardized Prometheus metrics for FEAGI
//!
//! All crates should use these metric definitions to ensure
//! consistent metric names and labels across the system.

use prometheus::{Counter, Histogram, Gauge, Registry, Encoder, TextEncoder};
use std::sync::OnceLock;

/// Global metrics registry
static METRICS_REGISTRY: OnceLock<Registry> = OnceLock::new();

/// Initialize metrics registry (call once at startup)
pub fn init_metrics() -> Registry {
    METRICS_REGISTRY.get_or_init(|| Registry::new()).clone()
}

/// Get metrics registry
pub fn registry() -> &'static Registry {
    METRICS_REGISTRY.get().expect("Metrics not initialized")
}

// ============================================================================
// BURST ENGINE METRICS
// ============================================================================

lazy_static::lazy_static! {
    /// Total number of bursts executed
    pub static ref BURST_COUNT: Counter = Counter::new(
        "feagi_burst_total",
        "Total number of bursts executed"
    ).unwrap();
    
    /// Burst execution duration in seconds
    pub static ref BURST_DURATION: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "feagi_burst_duration_seconds",
            "Burst execution time"
        )
        .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
    ).unwrap();
    
    /// Number of neurons fired in last burst
    pub static ref NEURONS_FIRED: Gauge = Gauge::new(
        "feagi_neurons_fired",
        "Number of neurons fired in last burst"
    ).unwrap();
    
    /// Number of synapses activated in last burst
    pub static ref SYNAPSES_ACTIVATED: Gauge = Gauge::new(
        "feagi_synapses_activated",
        "Number of synapses activated in last burst"
    ).unwrap();
}

// ============================================================================
// API METRICS
// ============================================================================

lazy_static::lazy_static! {
    /// Total API requests by endpoint, method, and status
    pub static ref API_REQUESTS: Counter = Counter::with_opts(
        prometheus::Opts::new(
            "feagi_api_requests_total",
            "Total API requests"
        )
    ).unwrap();
    
    /// API request latency in seconds
    pub static ref API_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "feagi_api_request_duration_seconds",
            "API request latency"
        )
        .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
    ).unwrap();
}

// ============================================================================
// GENOME METRICS
// ============================================================================

lazy_static::lazy_static! {
    /// Genome loading duration in seconds
    pub static ref GENOME_LOAD_DURATION: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "feagi_genome_load_duration_seconds",
            "Genome loading time"
        )
        .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0])
    ).unwrap();
    
    /// Number of cortical areas in current genome
    pub static ref CORTICAL_AREAS: Gauge = Gauge::new(
        "feagi_cortical_areas",
        "Number of cortical areas"
    ).unwrap();
}

// ============================================================================
// CONNECTOME METRICS
// ============================================================================

lazy_static::lazy_static! {
    /// Total neurons in connectome
    pub static ref CONNECTOME_NEURONS: Gauge = Gauge::new(
        "feagi_connectome_neurons",
        "Number of neurons in connectome"
    ).unwrap();
    
    /// Total synapses in connectome
    pub static ref CONNECTOME_SYNAPSES: Gauge = Gauge::new(
        "feagi_connectome_synapses",
        "Number of synapses in connectome"
    ).unwrap();
}

// ============================================================================
// SYSTEM METRICS
// ============================================================================

lazy_static::lazy_static! {
    /// Memory usage in bytes
    pub static ref MEMORY_USAGE: Gauge = Gauge::new(
        "feagi_memory_usage_bytes",
        "Memory usage in bytes"
    ).unwrap();
    
    /// Number of active threads
    pub static ref THREAD_COUNT: Gauge = Gauge::new(
        "feagi_thread_count",
        "Number of active threads"
    ).unwrap();
}

/// Register all metrics with the registry
pub fn register_all_metrics(registry: &Registry) {
    registry.register(Box::new(BURST_COUNT.clone())).unwrap();
    registry.register(Box::new(BURST_DURATION.clone())).unwrap();
    registry.register(Box::new(NEURONS_FIRED.clone())).unwrap();
    registry.register(Box::new(SYNAPSES_ACTIVATED.clone())).unwrap();
    
    registry.register(Box::new(API_REQUESTS.clone())).unwrap();
    registry.register(Box::new(API_LATENCY.clone())).unwrap();
    
    registry.register(Box::new(GENOME_LOAD_DURATION.clone())).unwrap();
    registry.register(Box::new(CORTICAL_AREAS.clone())).unwrap();
    
    registry.register(Box::new(CONNECTOME_NEURONS.clone())).unwrap();
    registry.register(Box::new(CONNECTOME_SYNAPSES.clone())).unwrap();
    
    registry.register(Box::new(MEMORY_USAGE.clone())).unwrap();
    registry.register(Box::new(THREAD_COUNT.clone())).unwrap();
}

/// Export metrics as Prometheus text format
pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&registry().gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

---

### Step 5: Tracing Initialization

**Location**: `feagi-observability/src/tracing.rs`

```rust
//! Standardized tracing initialization for FEAGI
//!
//! All binaries should use this initialization function
//! to ensure consistent tracing setup.

use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use tracing_subscriber::fmt::{self, Layer};

/// Initialize tracing with standardized configuration
pub fn init_tracing(config: &TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    let fmt_layer = match config.format {
        LogFormat::Text => {
            Layer::default()
                .with_target(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .boxed()
        }
        LogFormat::Json => {
            Layer::default()
                .json()
                .with_target(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .boxed()
        }
    };
    
    let registry = Registry::default()
        .with(filter)
        .with(fmt_layer);
    
    #[cfg(feature = "opentelemetry")]
    if let Some(endpoint) = &config.tracing_endpoint {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
            )
            .with_trace_config(
                opentelemetry_sdk::trace::config().with_resource(
                    opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "feagi-core"),
                        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])
                )
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        
        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();
    } else {
        registry.init();
    }
    
    #[cfg(not(feature = "opentelemetry"))]
    registry.init();
    
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub level: String,
    pub format: LogFormat,
    pub tracing_endpoint: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Text,
    Json,
}

impl Default for TracingConfig {
    fn default() -> Self {
        TracingConfig {
            level: "info".to_string(),
            format: LogFormat::Text,
            tracing_endpoint: None,
        }
    }
}
```

---

### Step 6: Workspace-Level Dependency

**Update**: `feagi-core/Cargo.toml`

```toml
[workspace.dependencies]
# ... existing dependencies ...

# Observability (shared across all crates)
feagi-observability = { path = "crates/feagi-observability" }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
anyhow = "1.0"
thiserror = "1.0.50"
prometheus = "0.13"
```

**Update all crate `Cargo.toml` files:**

```toml
[dependencies]
# Use workspace dependency for consistency
feagi-observability = { workspace = true }
# Remove direct tracing/anyhow/thiserror dependencies
```

---

### Step 7: Build-Time Enforcement

**Location**: `feagi-core/.cargo/config.toml`

```toml
[build]
# Deny warnings in CI
rustflags = ["-D", "warnings"]

[lints.rust]
# Deny direct tracing usage (must use feagi-observability)
tracing-direct = { level = "deny", path = "tools/lints/tracing_direct.rs" }
# Deny direct log usage (must use tracing)
log-crate = { level = "deny", path = "tools/lints/log_crate.rs" }
```

**Location**: `feagi-core/tools/lints/tracing_direct.rs`

```rust
//! Clippy lint: Deny direct tracing::*! macro usage
//! 
//! All crates must use feagi-observability macros instead.

use rustc_middle::ty::TyCtxt;
use rustc_hir::*;
use rustc_lint::{LateContext, LateLintPass, Lint, LintPass};
use rustc_session::declare_lint_pass;

declare_lint! {
    TRACING_DIRECT,
    Deny,
    "deny direct tracing::*! macro usage - use feagi-observability macros instead"
}

declare_lint_pass!(TracingDirect => [TRACING_DIRECT]);

impl<'tcx> LateLintPass<'tcx> for TracingDirect {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // Check for tracing::info!, tracing::error!, etc.
        // Implementation details...
    }
}
```

**Alternative: Simpler approach with `deny.toml`**

**Location**: `feagi-core/deny.toml`

```toml
[bans]
# Deny direct log crate usage
multiple-versions = "deny"
wildcards = "deny"

[[bans.deny]]
name = "log"
reason = "All crates must use feagi-observability logging macros"

[[bans.deny]]
name = "env_logger"
reason = "Use feagi-observability tracing initialization instead"
```

---

### Step 8: CI Enforcement

**Location**: `.github/workflows/check-logging.yml` (or similar)

```yaml
name: Check Logging Standards

on: [push, pull_request]

jobs:
  check-logging:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Check for direct log crate usage
        run: |
          if grep -r "use log::" crates/; then
            echo "❌ Found direct 'log' crate usage. Use feagi-observability instead."
            exit 1
          fi
      
      - name: Check for direct tracing::*! macros
        run: |
          if grep -r "tracing::info!" crates/ --include="*.rs" | grep -v "feagi-observability"; then
            echo "❌ Found direct tracing::*! macro usage. Use feagi-observability macros instead."
            exit 1
          fi
      
      - name: Verify feagi-observability dependency
        run: |
          for crate in crates/*/Cargo.toml; do
            if ! grep -q "feagi-observability" "$crate"; then
              echo "❌ Crate $crate missing feagi-observability dependency"
              exit 1
            fi
          done
      
      - name: Check logging macro usage
        run: |
          # Verify all crates use feagi-observability macros
          cargo build --workspace 2>&1 | grep -i "error" || true
```

---

### Step 9: Documentation and Examples

**Location**: `feagi-core/crates/feagi-observability/README.md`

```markdown
# feagi-observability

Standardized logging, error handling, metrics, and tracing for FEAGI.

## Usage

### Logging

```rust
use feagi_observability::{burst_info, api_info, feagi_error};

// Burst event
burst_info!(
    burst_id = 42,
    neurons_fired = 1000,
    synapses_activated = 5000,
    duration_ms = 15.2
);

// API event
api_info!("GET", "/v1/cortical_area/ipu", status = 200, latency_ms = 5.0);

// Error
feagi_error!(cortical_area_id = "v1"; "Failed to load cortical area");
```

### Metrics

```rust
use feagi_observability::metrics::*;

BURST_COUNT.inc();
let _timer = BURST_DURATION.start_timer();
// ... burst logic
```

### Tracing

```rust
use feagi_observability::{init_tracing, TracingConfig, LogFormat};

let config = TracingConfig {
    level: "info".to_string(),
    format: LogFormat::Json,
    tracing_endpoint: Some("http://jaeger:4317".to_string()),
};
init_tracing(&config)?;
```

### Errors

```rust
use feagi_observability::{FeagiResult, FeagiErrorContext};

fn load_genome(path: &Path) -> FeagiResult<Genome> {
    std::fs::read_to_string(path)
        .with_feagi_context("Reading genome file")
        .context("Failed to load genome")?;
    // ...
}
```

## Migration Guide

1. **Replace `log` with `feagi-observability` macros**
2. **Replace `tracing::*!` with `feagi-observability` macros**
3. **Use `FeagiResult` instead of custom result types**
4. **Initialize tracing with `init_tracing()`**
5. **Use standardized metrics from `metrics` module**
```

---

### Step 10: Code Review Checklist

**Location**: `feagi-core/.github/PULL_REQUEST_TEMPLATE.md`

```markdown
## Logging and Debugging Checklist

- [ ] All logging uses `feagi-observability` macros (not direct `tracing::*!`)
- [ ] All errors use `FeagiResult` or convert to it
- [ ] Metrics are registered and used from `feagi-observability`
- [ ] Tracing is initialized with `init_tracing()` (for binaries)
- [ ] No direct `log` crate usage
- [ ] No direct `env_logger` usage
- [ ] Error context is added with `with_feagi_context()`
- [ ] Log messages include relevant context fields
```

---

## Migration Strategy

### Phase 1: Create `feagi-observability` (Week 1)
1. Create crate structure
2. Implement logging macros
3. Implement error types
4. Implement metrics
5. Implement tracing initialization

### Phase 2: Migrate Core Crates (Week 2)
1. Update `feagi-api` to use `feagi-observability`
2. Update `feagi-services` to use `feagi-observability`
3. Update `feagi-burst-engine` to use `feagi-observability`
4. Update `feagi-brain-development` to use `feagi-observability`

### Phase 3: Migrate Remaining Crates (Week 3)
1. Update all other crates
2. Remove `log` and `env_logger` dependencies
3. Update documentation

### Phase 4: Enforcement (Week 4)
1. Add CI checks
2. Add clippy lints (optional)
3. Update PR template
4. Document standards

---

## Benefits

### Consistency
- ✅ All crates use same logging macros
- ✅ Standardized error types
- ✅ Consistent metric names
- ✅ Unified tracing setup

### Maintainability
- ✅ Single place to update logging behavior
- ✅ Easier to add new metrics
- ✅ Centralized error handling
- ✅ Consistent debugging experience

### Observability
- ✅ Structured logs with consistent fields
- ✅ Standardized metrics
- ✅ Distributed tracing support
- ✅ Error reporting consistency

### Developer Experience
- ✅ Clear examples and documentation
- ✅ Build-time checks prevent mistakes
- ✅ CI enforcement catches violations
- ✅ Easy to add new logging/errors

---

## Enforcement Mechanisms Summary

1. **Shared Crate**: `feagi-observability` ensures all crates use same APIs
2. **Workspace Dependencies**: Centralized dependency versions
3. **Build-Time Checks**: Clippy lints (optional) or `deny.toml`
4. **CI Checks**: Automated verification of standards
5. **Code Review**: PR checklist ensures compliance
6. **Documentation**: Clear examples and migration guide

---

## Next Steps

1. **Create `feagi-observability` crate** structure
2. **Implement core modules** (logging, errors, metrics, tracing)
3. **Migrate one crate** (`feagi-api`) as proof of concept
4. **Add CI checks** to prevent regressions
5. **Document standards** and migration guide
6. **Migrate remaining crates** systematically

---

## References

- [tracing documentation](https://docs.rs/tracing/)
- [anyhow documentation](https://docs.rs/anyhow/)
- [Prometheus Rust client](https://docs.rs/prometheus/)
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)

