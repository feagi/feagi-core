// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Unified logging initialization for FEAGI
//!
//! Provides file logging with rotation, per-crate log files, and configurable retention.

#[cfg(feature = "file-logging")]
use anyhow::Context;
use anyhow::anyhow;
use anyhow::Result;
#[cfg(feature = "file-logging")]
use chrono::Utc;
use std::path::{Path, PathBuf};
#[cfg(feature = "file-logging")]
use tracing_appender::rolling;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

use crate::cli::CrateDebugFlags;

/// Resolve tracing EnvFilter with explicit RUST_LOG precedence.
///
/// If RUST_LOG is present, it is parsed and used verbatim.
/// Otherwise, fall back to the per-crate debug flags filter string.
fn resolve_env_filter(debug_flags: &CrateDebugFlags) -> Result<EnvFilter> {
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        return EnvFilter::try_new(rust_log.clone())
            .map_err(|e| anyhow!("Invalid RUST_LOG '{}': {}", rust_log, e));
    }

    let filter = debug_flags.to_filter_string();
    Ok(EnvFilter::new(&filter))
}

/// Logging initialization result
pub struct LoggingGuard {
    #[cfg(feature = "file-logging")]
    _file_guards: Vec<tracing_appender::non_blocking::WorkerGuard>,
    #[cfg(feature = "file-logging")]
    log_dir: PathBuf,
}

impl LoggingGuard {
    /// Get the log directory path (desktop only)
    #[cfg(feature = "file-logging")]
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    #[cfg(not(feature = "file-logging"))]
    pub fn log_dir(&self) -> &Path {
        // WASM builds don't have file logging
        Path::new(".")
    }
}

/// Initialize logging with file output and console output
///
/// Creates a timestamped folder structure:
/// ```
/// ./logs/
///   └── run_20250101_120000/
///       ├── feagi-api.log
///       ├── feagi-services.log
///       ├── feagi-bdu.log
///       └── feagi.log (combined)
/// ```
///
/// # Arguments
/// * `debug_flags` - Per-crate debug flags for filtering
/// * `log_dir` - Base directory for logs (default: `./logs`)
/// * `retention_days` - Keep logs for N days (default: 30)
/// * `retention_runs` - Keep N most recent runs (default: 10)
#[cfg(feature = "file-logging")]
pub fn init_logging(
    debug_flags: &CrateDebugFlags,
    log_dir: Option<PathBuf>,
    retention_days: Option<u64>,
    retention_runs: Option<usize>,
) -> Result<LoggingGuard> {
    let base_log_dir = log_dir.unwrap_or_else(|| PathBuf::from("./logs"));

    // Create timestamped run folder
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let run_folder = base_log_dir.join(format!("run_{}", timestamp));
    std::fs::create_dir_all(&run_folder)
        .with_context(|| format!("Failed to create log directory: {}", run_folder.display()))?;

    // Clean up old logs based on retention policy
    cleanup_old_logs(&base_log_dir, retention_days, retention_runs)?;

    let env_filter = resolve_env_filter(debug_flags)?;

    // Create per-crate log files
    let mut layers = Vec::new();
    let mut file_guards = Vec::new();

    // Console layer (human-readable)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_filter(env_filter.clone());
    layers.push(console_layer.boxed());

    // File layers - one per crate
    for crate_name in crate::KNOWN_CRATES {
        // Create file appender with daily rotation
        let file_appender = rolling::daily(&run_folder, format!("{}.log", crate_name));

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        file_guards.push(guard);

        // JSON formatter for file
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .json()
            // Filter only this crate's logs
            .with_filter(EnvFilter::new(format!("{}=debug,info", crate_name)))
            .boxed();

        layers.push(file_layer);
    }

    // Combined log file (all crates)
    let combined_appender = rolling::daily(&run_folder, "feagi.log");
    let (combined_non_blocking, combined_guard) = tracing_appender::non_blocking(combined_appender);

    let combined_layer = tracing_subscriber::fmt::layer()
        .with_writer(combined_non_blocking)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .with_filter(env_filter.clone())
        .boxed();

    layers.push(combined_layer);

    // Initialize subscriber with all layers
    Registry::default().with(layers).init();

    // Keep all guards alive (they flush logs on drop)
    file_guards.push(combined_guard);

    Ok(LoggingGuard {
        _file_guards: file_guards,
        log_dir: run_folder,
    })
}

/// Initialize logging with console output only (WASM-compatible)
///
/// For WASM builds, file logging is not available. This function provides
/// console-only logging that works in browsers.
#[cfg(not(feature = "file-logging"))]
pub fn init_logging(
    debug_flags: &CrateDebugFlags,
    _log_dir: Option<PathBuf>,
    _retention_days: Option<u64>,
    _retention_runs: Option<usize>,
) -> Result<LoggingGuard> {
    let env_filter = resolve_env_filter(debug_flags)?;

    // Console layer only (human-readable)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_filter(env_filter);

    // Initialize subscriber with console layer only
    Registry::default().with(console_layer.boxed()).init();

    Ok(LoggingGuard {})
}

/// Clean up old log directories based on retention policy (desktop only)
#[cfg(feature = "file-logging")]
fn cleanup_old_logs(
    base_log_dir: &Path,
    retention_days: Option<u64>,
    retention_runs: Option<usize>,
) -> Result<()> {
    if !base_log_dir.exists() {
        return Ok(());
    }

    let retention_days = retention_days.unwrap_or(30);
    let retention_runs = retention_runs.unwrap_or(10);
    let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

    // Collect all run directories
    let mut runs: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

    for entry in std::fs::read_dir(base_log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if dir_name.starts_with("run_") {
                    // Parse timestamp from folder name: run_20250101_120000
                    if let Some(timestamp_str) = dir_name.strip_prefix("run_") {
                        if let Ok(dt) = DateTime::parse_from_str(timestamp_str, "%Y%m%d_%H%M%S") {
                            runs.push((path, dt.with_timezone(&Utc)));
                        }
                    }
                }
            }
        }
    }

    // Sort by date (oldest first)
    runs.sort_by_key(|(_, dt)| *dt);

    // Remove runs older than retention_days
    let mut removed_count = 0;
    for (path, dt) in &runs {
        if *dt < cutoff_date {
            if let Err(e) = std::fs::remove_dir_all(path) {
                eprintln!(
                    "Warning: Failed to remove old log directory {}: {}",
                    path.display(),
                    e
                );
            } else {
                removed_count += 1;
            }
        }
    }

    // Keep only the most recent N runs (after removing old ones)
    if runs.len() - removed_count > retention_runs {
        let to_remove = runs.len() - removed_count - retention_runs;
        for (path, dt) in runs.iter().take(to_remove) {
            if *dt >= cutoff_date {
                // Only remove if not already removed by date-based cleanup
                if path.exists() {
                    if let Err(e) = std::fs::remove_dir_all(path) {
                        eprintln!(
                            "Warning: Failed to remove old log directory {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Initialize logging with default settings
pub fn init_logging_default(debug_flags: &CrateDebugFlags) -> Result<LoggingGuard> {
    init_logging(debug_flags, None, None, None)
}
