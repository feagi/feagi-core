// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # FEAGI Configuration System
//!
//! Cross-platform, type-safe configuration loader for FEAGI with support for:
//! - TOML file parsing
//! - Environment variable overrides
//! - CLI argument overrides
//! - Multiple deployment targets (std, no_std, wasm)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use feagi_config::{load_config, FeagiConfig};
//!
//! // Load configuration with automatic file discovery and overrides
//! let config = load_config(None, None).expect("Failed to load config");
//!
//! // Access type-safe configuration values
//! println!("API Host: {}", config.api.host);
//! println!("API Port: {}", config.api.port);
//! ```
//!
//! ## Architecture Compliance
//!
//! This crate enforces FEAGI 2.0 architecture principles:
//! - ❌ No hardcoded values (hosts, ports, timeouts)
//! - ✅ Single source of truth (feagi_configuration.toml)
//! - ✅ Environment-specific overrides
//! - ✅ Cross-platform compatibility (Docker, K8s, embedded, desktop)

#![cfg_attr(not(feature = "std"), no_std)]

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "std")]
pub mod loader;

pub mod types;
pub mod validation;

#[cfg(feature = "std")]
pub use loader::{find_config_file, load_config, apply_environment_overrides, apply_cli_overrides};

pub use types::*;
pub use validation::{validate_config, ConfigValidationError};

/// Re-export for convenience
pub use serde;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[cfg(feature = "std")]
    #[error("Config file not found. Searched: {0}")]
    FileNotFound(String),
    
    #[cfg(feature = "std")]
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[cfg(feature = "std")]
    #[error("Invalid TOML syntax: {0}")]
    ParseError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Port conflict: {0} and {1} both use port {2}")]
    PortConflict(String, String, u16),
    
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

#[cfg(feature = "std")]
impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_types_compile() {
        // Smoke test to ensure types are properly defined
        let _config = FeagiConfig::default();
    }
}

