// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # feagi-observability
//!
//! Unified observability infrastructure for FEAGI (logging, telemetry, profiling).
//!
//! Provides consistent observability patterns across all FEAGI crates with
//! per-crate debug flag support.
//!
//! ## Features
//! - `file-logging`: File-based log rotation (desktop only)
//! - `metrics`: Prometheus metrics collection (desktop only)
//! - `opentelemetry`: OpenTelemetry exporter support
//! - `profiling`: Chrome tracing and pprof profiling support

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod cli;
pub mod config;
pub mod init;

// Placeholder modules - to be implemented
pub mod context {
    //! Correlation IDs and context propagation
    // TODO: Implement
}

pub mod errors {
    //! Error handling and reporting
    // TODO: Implement
}

pub mod logging {
    //! Structured logging with spans
    // TODO: Implement
}

pub mod metrics {
    //! Prometheus metrics
    // TODO: Implement
}

pub mod profiling {
    //! CPU/Memory profiling
    // TODO: Implement
}

pub mod telemetry {
    //! Unified telemetry collection
    // TODO: Implement
}

pub mod tracing {
    //! Distributed tracing (OpenTelemetry)
    // TODO: Implement
}

// Re-export commonly used items
pub use cli::*;
pub use config::*;
pub use init::*;

/// Known FEAGI crate names for debug flags
pub const KNOWN_CRATES: &[&str] = &[
    "feagi-api",
    "feagi-burst-engine",
    "feagi-bdu",
    "feagi-services",
    "feagi-evo",
    "feagi-config",
    "feagi-io",
    "feagi-transports",
    "feagi-agent-sdk",
    "feagi-state-manager",
    "feagi-plasticity",
    "feagi-connectome-serialization",
];
