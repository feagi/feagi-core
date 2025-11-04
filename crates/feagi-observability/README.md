# feagi-observability

Standardized logging, error handling, metrics, and tracing infrastructure for FEAGI.

## Purpose

This crate provides a **single source of truth** for all observability concerns across FEAGI crates. All crates MUST use this crate instead of directly depending on `tracing`, `log`, `anyhow`, etc.

## Usage

### Logging

```rust
use feagi_observability::{burst_info, api_info, feagi_error};

// Structured logging with consistent fields
burst_info!(
    burst_id = 42,
    neurons_fired = 1000,
    synapses_activated = 5000
);

api_info!("GET", "/v1/genome/file_name", status = 200);

feagi_error!(cortical_area_id = "v1"; "Failed to load cortical area");
```

### Errors

```rust
use feagi_observability::{FeagiResult, FeagiErrorContext};

fn load_genome(path: &Path) -> FeagiResult<Genome> {
    std::fs::read_to_string(path)
        .with_feagi_context("Reading genome file")?;
    // ...
}
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

init_tracing(&TracingConfig {
    level: "info".to_string(),
    format: LogFormat::Json,
    tracing_endpoint: Some("http://jaeger:4317".to_string()),
})?;
```

## Structure

```
feagi-observability/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs          # Re-exports
    ├── logging.rs      # Standardized logging macros
    ├── errors.rs       # Error types and context
    ├── metrics.rs      # Prometheus metrics
    ├── tracing.rs      # Tracing initialization
    └── debug.rs        # Debugging utilities
```

## Dependencies

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
anyhow = "1.0"
thiserror = "1.0"
prometheus = "0.13"
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }

[features]
default = []
opentelemetry = ["opentelemetry", "opentelemetry-sdk", "opentelemetry-otlp", "tracing-opentelemetry"]
```

## Benefits

1. **Consistency**: All crates use same logging patterns
2. **Maintainability**: Single place to update observability behavior
3. **Enforcement**: Can check for direct `tracing::*!` usage in CI
4. **Evolution**: Easy to add new logging patterns/metrics



