//! Observability configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unified observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub logging: LoggingConfig,
    pub telemetry: TelemetryConfig,
    pub profiling: ProfilingConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Log format (text or json)
    pub format: LogFormat,
    
    /// Output destination
    pub output: LogOutput,
    
    /// File path (if output is file)
    pub file_path: Option<PathBuf>,
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
}

/// Log output destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File(String),
    Syslog,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for ObservabilityConfig {
    fn default() -> Self {
        ObservabilityConfig {
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Text,
                output: LogOutput::Stdout,
                file_path: None,
            },
            telemetry: TelemetryConfig {
                metrics_enabled: true,
                tracing_enabled: false,
                tracing_endpoint: None,
                metrics_path: "/metrics".to_string(),
                health_check_path: "/health".to_string(),
                system_metrics_interval: 5,
            },
            profiling: ProfilingConfig {
                cpu_profiling: false,
                memory_profiling: false,
                output_dir: PathBuf::from("./profiles"),
                sample_rate: 1.0,
                chrome_tracing: false,
                perf_profiling: false,
            },
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            format: LogFormat::Text,
            output: LogOutput::Stdout,
            file_path: None,
        }
    }
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



