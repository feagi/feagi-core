# Unified Observability Architecture: Profiling, Logging, and Telemetry

**Status**: Architecture Extension  
**Date**: 2025-10-31  
**Author**: FEAGI Architecture Team

## Executive Summary

This document extends the `feagi-observability` crate proposal to include **profiling** and **telemetry** alongside logging, creating a unified observability infrastructure. Profiling, logging, and telemetry share common context, correlation IDs, and initialization patterns, making them natural partners in a single crate.

---

## Why Unified Infrastructure?

### 1. Shared Context and Correlation

**Problem**: Logs, traces, metrics, and profiles are often disconnected, making it hard to correlate.

**Solution**: Unified correlation IDs propagate across all observability systems:

```rust
// Same correlation ID used for:
// - Logs: "request_id=abc123"
// - Traces: Span with trace_id="abc123"
// - Metrics: Label request_id="abc123"
// - Profiles: Profile metadata includes request_id="abc123"
```

**Benefit**: Can trace a request from log → trace → metric → profile seamlessly.

### 2. Unified Initialization

**Problem**: Multiple initialization calls scattered across codebase.

**Solution**: Single initialization function:

```rust
use feagi_observability::init_observability;

init_observability(&ObservabilityConfig {
    logging: LoggingConfig { level: "info", format: LogFormat::Json },
    telemetry: TelemetryConfig { 
        metrics_enabled: true,
        tracing_enabled: true,
        tracing_endpoint: Some("http://jaeger:4317".to_string()),
    },
    profiling: ProfilingConfig {
        cpu_profiling: true,
        memory_profiling: false,
        output_dir: "./profiles".into(),
    },
})?;
```

**Benefit**: One place to configure all observability, ensures consistency.

### 3. Shared Data Collection

**Problem**: Each system (logging, metrics, tracing) collects similar data separately.

**Solution**: Unified collection layer:

```rust
// Single instrumentation point collects:
// - Logs (structured)
// - Metrics (counters/histograms)
// - Traces (spans)
// - Profiling samples (if enabled)
#[instrument]
pub async fn execute_burst() {
    // Automatically creates:
    // - Log entry
    // - Trace span
    // - Metrics counter
    // - Profiling sample (if enabled)
}
```

**Benefit**: Less overhead, consistent data collection.

### 4. Consistent Patterns

**Problem**: Different APIs for logging vs metrics vs tracing.

**Solution**: Unified macros:

```rust
// Same pattern for all observability
feagi_observability::burst_info!(
    burst_id = 42,
    neurons_fired = 1000,
    // Automatically creates:
    // - Log entry
    // - Metric update
    // - Trace span
    // - Profiling sample (if enabled)
);
```

**Benefit**: Developers learn one API, not three.

---

## Architecture: Unified Observability

```
┌─────────────────────────────────────────────────────────────┐
│                  feagi-observability                        │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Context Layer (Correlation IDs, Request Context)    │  │
│  │  - Trace ID propagation                               │  │
│  │  - Span context                                       │  │
│  │  - Request correlation                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                           ↓                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Instrumentation Layer (Unified Collection)          │  │
│  │  - #[instrument] macro                               │  │
│  │  - Structured logging macros                         │  │
│  │  - Metrics macros                                    │  │
│  │  - Profiling hooks                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                           ↓                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   LOGGING    │  │  TELEMETRY   │  │  PROFILING   │    │
│  │              │  │              │  │              │    │
│  │ - Structured │  │ - Metrics    │  │ - CPU        │    │
│  │ - Spans      │  │ - Traces     │  │ - Memory     │    │
│  │ - Context    │  │ - Health     │  │ - Flamegraph │    │
│  │              │  │              │  │              │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                           ↓                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Export Layer (Backends)                             │  │
│  │  - Logs → stdout, file, syslog, ELK                 │  │
│  │  - Metrics → Prometheus                              │  │
│  │  - Traces → Jaeger, Zipkin                          │  │
│  │  - Profiles → Chrome DevTools, perf, flamegraph      │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation: Profiling Module

**Location**: `feagi-observability/src/profiling.rs`

```rust
//! CPU and memory profiling for FEAGI
//!
//! Integrates with tracing for zero-overhead profiling when disabled.

use std::path::PathBuf;
use tracing::Instrument;

/// Profiling configuration
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Enable CPU profiling
    pub cpu_profiling: bool,
    
    /// Enable memory profiling
    pub memory_profiling: bool,
    
    /// Output directory for profiles
    pub output_dir: PathBuf,
    
    /// Profiling sample rate (0.0-1.0)
    pub sample_rate: f64,
    
    /// Enable Chrome DevTools tracing
    pub chrome_tracing: bool,
    
    /// Enable perf profiling (Linux only)
    pub perf_profiling: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        ProfilingConfig {
            cpu_profiling: false,
            memory_profiling: false,
            output_dir: PathBuf::from("./profiles"),
            sample_rate: 1.0,
            chrome_tracing: false,
            perf_profiling: false,
        }
    }
}

/// CPU profiler using tracing-chrome
pub struct CpuProfiler {
    #[cfg(feature = "chrome-tracing")]
    chrome_layer: Option<tracing_chrome::ChromeLayerBuilder>,
    output_path: PathBuf,
}

impl CpuProfiler {
    /// Create a new CPU profiler
    pub fn new(config: &ProfilingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let output_path = config.output_dir.join("trace.json");
        
        #[cfg(feature = "chrome-tracing")]
        let chrome_layer = if config.chrome_tracing {
            Some(tracing_chrome::ChromeLayerBuilder::new()
                .file(&output_path)
                .build())
        } else {
            None
        };
        
        #[cfg(not(feature = "chrome-tracing"))]
        let chrome_layer = None;
        
        Ok(CpuProfiler {
            chrome_layer,
            output_path,
        })
    }
    
    /// Start profiling
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation
        Ok(())
    }
    
    /// Stop profiling and save
    pub fn stop(&mut self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Implementation
        Ok(self.output_path.clone())
    }
}

/// Memory profiler
pub struct MemoryProfiler {
    enabled: bool,
    samples: Vec<MemorySample>,
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub heap_size: usize,
    pub allocations: usize,
    pub deallocations: usize,
}

impl MemoryProfiler {
    pub fn new(enabled: bool) -> Self {
        MemoryProfiler {
            enabled,
            samples: Vec::new(),
        }
    }
    
    pub fn sample(&mut self) {
        if !self.enabled {
            return;
        }
        
        // Sample memory usage
        // Implementation would use heaptrack or similar
    }
    
    pub fn generate_report(&self) -> MemoryReport {
        MemoryReport {
            samples: self.samples.clone(),
            peak_memory: self.samples.iter().map(|s| s.heap_size).max().unwrap_or(0),
            total_allocations: self.samples.iter().map(|s| s.allocations).sum(),
        }
    }
}

#[derive(Debug)]
pub struct MemoryReport {
    pub samples: Vec<MemorySample>,
    pub peak_memory: usize,
    pub total_allocations: usize,
}

/// Flamegraph generator
pub struct FlamegraphGenerator;

impl FlamegraphGenerator {
    /// Generate flamegraph from trace
    pub fn generate(&self, trace_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Implementation would use inferno or similar
        // For now, return input path
        Ok(trace_path.clone())
    }
}

/// Instrument a function for profiling
#[macro_export]
macro_rules! profile {
    ($name:expr, $code:block) => {
        {
            let _guard = tracing::span!(tracing::Level::INFO, "profile", name = $name).entered();
            $code
        }
    };
}
```

---

## Implementation: Telemetry Module

**Location**: `feagi-observability/src/telemetry.rs`

```rust
//! Unified telemetry collection for FEAGI
//!
//! Combines metrics, traces, and health checks into a single interface.

use crate::metrics::*;
use crate::tracing::*;
use std::sync::Arc;

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Enable Prometheus metrics
    pub metrics_enabled: bool,
    
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    
    /// Tracing endpoint (Jaeger OTLP)
    pub tracing_endpoint: Option<String>,
    
    /// Metrics endpoint path
    pub metrics_path: String,
    
    /// Health check endpoint
    pub health_check_path: String,
    
    /// System metrics collection interval (seconds)
    pub system_metrics_interval: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        TelemetryConfig {
            metrics_enabled: true,
            tracing_enabled: false,
            tracing_endpoint: None,
            metrics_path: "/metrics".to_string(),
            health_check_path: "/health".to_string(),
            system_metrics_interval: 5,
        }
    }
}

/// Unified telemetry collector
pub struct TelemetryCollector {
    metrics_registry: Arc<prometheus::Registry>,
    tracer: Option<opentelemetry::sdk::trace::Tracer>,
    health_status: Arc<std::sync::RwLock<HealthStatus>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub checks: Vec<HealthCheck>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: String,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new(config: &TelemetryConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let metrics_registry = Arc::new(prometheus::Registry::new());
        
        // Register all metrics
        crate::metrics::register_all_metrics(&metrics_registry);
        
        // Initialize tracer if enabled
        let tracer = if config.tracing_enabled {
            Some(init_tracer(config.tracing_endpoint.as_ref())?)
        } else {
            None
        };
        
        Ok(TelemetryCollector {
            metrics_registry,
            tracer,
            health_status: Arc::new(std::sync::RwLock::new(HealthStatus {
                status: "healthy".to_string(),
                checks: Vec::new(),
                timestamp: chrono::Utc::now(),
            })),
        })
    }
    
    /// Get metrics registry
    pub fn metrics_registry(&self) -> &Arc<prometheus::Registry> {
        &self.metrics_registry
    }
    
    /// Export metrics as Prometheus text format
    pub fn export_metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder.encode(&self.metrics_registry.gather(), &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
    
    /// Update health status
    pub fn update_health(&self, check: HealthCheck) {
        let mut status = self.health_status.write().unwrap();
        // Update or add check
        if let Some(existing) = status.checks.iter_mut().find(|c| c.name == check.name) {
            *existing = check.clone();
        } else {
            status.checks.push(check);
        }
        
        // Update overall status
        status.status = if status.checks.iter().all(|c| c.status == "healthy") {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        };
        status.timestamp = chrono::Utc::now();
    }
    
    /// Get health status
    pub fn get_health(&self) -> HealthStatus {
        self.health_status.read().unwrap().clone()
    }
}

/// Unified instrumentation macro
/// Creates log entry, trace span, and metric update
#[macro_export]
macro_rules! instrument {
    ($name:expr, $($field:ident = $value:expr),* $(,)?) => {
        {
            // Create trace span
            let span = tracing::span!(
                tracing::Level::INFO,
                $name,
                $($field = $value),*
            );
            let _guard = span.entered();
            
            // Log entry
            tracing::info!(
                $($field = $value),*,
                "{}", $name
            );
            
            // Metric update (if applicable)
            // This would be context-aware based on the operation
            
            // Profiling sample (if enabled)
            // This would be handled by the profiling layer
        }
    };
}
```

---

## Implementation: Unified Initialization

**Location**: `feagi-observability/src/init.rs`

```rust
//! Unified initialization for all observability systems

use crate::logging::*;
use crate::telemetry::*;
use crate::profiling::*;
use crate::tracing::*;
use std::sync::Arc;

/// Unified observability configuration
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    pub logging: LoggingConfig,
    pub telemetry: TelemetryConfig,
    pub profiling: ProfilingConfig,
}

/// Unified observability manager
pub struct ObservabilityManager {
    pub telemetry: Arc<TelemetryCollector>,
    pub profiler: Option<CpuProfiler>,
    pub memory_profiler: Option<MemoryProfiler>,
}

impl ObservabilityManager {
    /// Initialize all observability systems
    pub fn init(config: &ObservabilityConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize logging
        init_logging(&config.logging)?;
        
        // Initialize telemetry
        let telemetry = Arc::new(TelemetryCollector::new(&config.telemetry)?);
        
        // Initialize profiling
        let profiler = if config.profiling.cpu_profiling {
            Some(CpuProfiler::new(&config.profiling)?)
        } else {
            None
        };
        
        let memory_profiler = if config.profiling.memory_profiling {
            Some(MemoryProfiler::new(true))
        } else {
            None
        };
        
        Ok(ObservabilityManager {
            telemetry,
            profiler,
            memory_profiler,
        })
    }
    
    /// Get telemetry collector
    pub fn telemetry(&self) -> &Arc<TelemetryCollector> {
        &self.telemetry
    }
    
    /// Start profiling
    pub fn start_profiling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut profiler) = self.profiler {
            profiler.start()?;
        }
        Ok(())
    }
    
    /// Stop profiling and save
    pub fn stop_profiling(&mut self) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        if let Some(ref mut profiler) = self.profiler {
            Ok(Some(profiler.stop()?))
        } else {
            Ok(None)
        }
    }
}

/// Convenience function to initialize all observability
pub fn init_observability(config: &ObservabilityConfig) -> Result<ObservabilityManager, Box<dyn std::error::Error>> {
    ObservabilityManager::init(config)
}
```

---

## Updated Cargo.toml

```toml
[package]
name = "feagi-observability"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Unified observability infrastructure for FEAGI (logging, telemetry, profiling)"

[dependencies]
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Telemetry
prometheus = "0.13"
opentelemetry = { version = "0.21", optional = true }
opentelemetry-sdk = { version = "0.21", optional = true }
opentelemetry-otlp = { version = "0.14", optional = true }
tracing-opentelemetry = { version = "0.21", optional = true }

# Profiling
tracing-chrome = { version = "0.6", optional = true }
pprof = { version = "0.12", optional = true }

# Errors
anyhow = "1.0"
thiserror.workspace = true

# Utilities
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }

[features]
default = []
opentelemetry = [
    "opentelemetry",
    "opentelemetry-sdk",
    "opentelemetry-otlp",
    "tracing-opentelemetry"
]
profiling = ["tracing-chrome", "pprof"]
```

---

## Usage Example: Unified Observability

```rust
use feagi_observability::{
    init_observability, ObservabilityConfig,
    LoggingConfig, LogFormat,
    TelemetryConfig,
    ProfilingConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize all observability systems at once
    let observability = init_observability(&ObservabilityConfig {
        logging: LoggingConfig {
            level: "info".to_string(),
            format: LogFormat::Json,
        },
        telemetry: TelemetryConfig {
            metrics_enabled: true,
            tracing_enabled: true,
            tracing_endpoint: Some("http://jaeger:4317".to_string()),
            ..Default::default()
        },
        profiling: ProfilingConfig {
            cpu_profiling: std::env::var("ENABLE_PROFILING").is_ok(),
            chrome_tracing: std::env::var("CHROME_TRACING").is_ok(),
            ..Default::default()
        },
    })?;
    
    // Use unified macros
    use feagi_observability::{burst_info, instrument};
    
    // This automatically creates:
    // - Log entry
    // - Trace span
    // - Metric update
    // - Profiling sample (if enabled)
    burst_info!(
        burst_id = 42,
        neurons_fired = 1000,
        synapses_activated = 5000
    );
    
    // Start profiling for specific operation
    observability.start_profiling()?;
    
    // ... perform operation ...
    
    // Stop profiling
    let profile_path = observability.stop_profiling()?;
    if let Some(path) = profile_path {
        println!("Profile saved to: {}", path.display());
    }
    
    Ok(())
}
```

---

## Benefits of Unified Infrastructure

### 1. Consistency
- ✅ Same correlation IDs across logs, traces, metrics, profiles
- ✅ Unified initialization pattern
- ✅ Consistent API design

### 2. Performance
- ✅ Shared infrastructure reduces overhead
- ✅ Zero-cost when disabled (compile-time)
- ✅ Efficient data collection

### 3. Developer Experience
- ✅ Learn one API, not three
- ✅ Single initialization call
- ✅ Clear examples and documentation

### 4. Correlation
- ✅ Can trace from log → trace → metric → profile
- ✅ Same context propagated everywhere
- ✅ Unified debugging experience

### 5. Maintainability
- ✅ Single crate to update
- ✅ Consistent patterns across codebase
- ✅ Easier to add new observability features

---

## Migration Path

1. **Create `feagi-observability`** with logging, telemetry, and profiling modules
2. **Implement unified initialization** (`init_observability`)
3. **Migrate crates** to use unified APIs
4. **Add profiling** to performance-critical paths
5. **Enable telemetry** in production deployments

---

## Conclusion

**Yes, profiling and telemetry should share infrastructure with logging.**

They benefit from:
- Shared correlation IDs
- Unified initialization
- Consistent patterns
- Better performance
- Easier maintenance

The unified `feagi-observability` crate provides all three in a single, consistent API.



