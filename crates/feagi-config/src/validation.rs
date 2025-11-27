//! Configuration validation
//!
//! This module provides validation logic to ensure configuration values are
//! consistent, within valid ranges, and don't conflict with each other.

use crate::{ConfigError, ConfigResult, FeagiConfig};

/// Validation errors that can occur during config validation
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    InvalidPortRange { port_name: String, port: u16 },
    PortConflict { port1: String, port2: String, port: u16 },
    MissingRequired { field: String },
    InvalidValue { field: String, reason: String },
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPortRange { port_name, port } => {
                write!(
                    f,
                    "Port {} = {} is outside valid range (1024-65535)",
                    port_name, port
                )
            }
            Self::PortConflict { port1, port2, port } => {
                write!(
                    f,
                    "Port conflict: {} and {} both use port {}",
                    port1, port2, port
                )
            }
            Self::MissingRequired { field } => {
                write!(f, "Missing required configuration: {}", field)
            }
            Self::InvalidValue { field, reason } => {
                write!(f, "Invalid configuration value for {}: {}", field, reason)
            }
        }
    }
}

/// Validate the complete configuration
///
/// Checks for:
/// - Port ranges (1024-65535 for non-root ports)
/// - Port conflicts (no two services using the same port)
/// - Required fields
/// - Valid value ranges
///
/// # Errors
///
/// Returns `ConfigError::ValidationError` with details if validation fails
pub fn validate_config(config: &FeagiConfig) -> ConfigResult<()> {
    let mut errors = Vec::new();

    // Validate port ranges
    validate_port_ranges(config, &mut errors);

    // Validate port conflicts
    validate_port_conflicts(config, &mut errors);

    // Validate required fields
    validate_required_fields(config, &mut errors);

    // Validate value ranges
    validate_value_ranges(config, &mut errors);

    // If any errors, return them
    if !errors.is_empty() {
        let error_messages = errors
            .iter()
            .map(|e| format!("  - {}", e))
            .collect::<Vec<_>>()
            .join("\n");
        
        return Err(ConfigError::ValidationError(format!(
            "Configuration validation failed:\n{}",
            error_messages
        )));
    }

    Ok(())
}

/// Validate that all ports are within valid range (1024-65535)
fn validate_port_ranges(config: &FeagiConfig, errors: &mut Vec<ConfigValidationError>) {
    // API port
    if config.api.port < 1024 {
        errors.push(ConfigValidationError::InvalidPortRange {
            port_name: "api.port".to_string(),
            port: config.api.port,
        });
    }

    // Agent ports
    if config.agent.registration_port < 1024 {
        errors.push(ConfigValidationError::InvalidPortRange {
            port_name: "agent.registration_port".to_string(),
            port: config.agent.registration_port,
        });
    }
    if config.agent.sensory_port < 1024 {
        errors.push(ConfigValidationError::InvalidPortRange {
            port_name: "agent.sensory_port".to_string(),
            port: config.agent.sensory_port,
        });
    }
    if config.agent.motor_port < 1024 {
        errors.push(ConfigValidationError::InvalidPortRange {
            port_name: "agent.motor_port".to_string(),
            port: config.agent.motor_port,
        });
    }

    // ZMQ ports
    for (port_name, port) in config.ports.all_ports() {
        if port < 1024 {
            errors.push(ConfigValidationError::InvalidPortRange {
                port_name: format!("ports.{}", port_name),
                port,
            });
        }
    }
}

/// Validate that no two services use the same port
fn validate_port_conflicts(config: &FeagiConfig, errors: &mut Vec<ConfigValidationError>) {
    let mut port_map: std::collections::HashMap<u16, Vec<String>> = std::collections::HashMap::new();

    // Collect ports that MUST be unique
    port_map.entry(config.api.port).or_default().push("api.port".to_string());
    
    // Note: agent.*_port and ports.zmq_*_port may legitimately use the same values
    // as they refer to the same underlying ZMQ endpoints from different perspectives.
    // We only check for conflicts within each namespace.
    
    // Collect all ZMQ ports (these must be unique within the ports namespace)
    let zmq_ports = config.ports.all_ports();
    let mut zmq_port_set: std::collections::HashMap<u16, Vec<String>> = std::collections::HashMap::new();
    
    for (port_name, port) in zmq_ports {
        zmq_port_set.entry(port).or_default().push(format!("ports.{}", port_name));
    }
    
    // Check for conflicts within ZMQ ports
    for (port, services) in zmq_port_set.iter() {
        if services.len() > 1 {
            for i in 0..services.len() - 1 {
                for j in i + 1..services.len() {
                    errors.push(ConfigValidationError::PortConflict {
                        port1: services[i].clone(),
                        port2: services[j].clone(),
                        port: *port,
                    });
                }
            }
        }
    }
    
    // Check if API port conflicts with any ZMQ port
    if zmq_port_set.contains_key(&config.api.port) {
        for zmq_service in &zmq_port_set[&config.api.port] {
            errors.push(ConfigValidationError::PortConflict {
                port1: "api.port".to_string(),
                port2: zmq_service.clone(),
                port: config.api.port,
            });
        }
    }
}

/// Validate required fields are not empty
fn validate_required_fields(config: &FeagiConfig, errors: &mut Vec<ConfigValidationError>) {
    // API host required
    if config.api.host.is_empty() {
        errors.push(ConfigValidationError::MissingRequired {
            field: "api.host".to_string(),
        });
    }

    // ZMQ host required
    if config.zmq.host.is_empty() {
        errors.push(ConfigValidationError::MissingRequired {
            field: "zmq.host".to_string(),
        });
    }

    // Agent default host required
    if config.agents.default_host.is_empty() {
        errors.push(ConfigValidationError::MissingRequired {
            field: "agents.default_host".to_string(),
        });
    }
}

/// Validate value ranges and constraints
fn validate_value_ranges(config: &FeagiConfig, errors: &mut Vec<ConfigValidationError>) {
    // GPU memory fraction must be between 0.0 and 1.0
    if config.resources.gpu_memory_fraction < 0.0 || config.resources.gpu_memory_fraction > 1.0 {
        errors.push(ConfigValidationError::InvalidValue {
            field: "resources.gpu_memory_fraction".to_string(),
            reason: "must be between 0.0 and 1.0".to_string(),
        });
    }

    // Timeouts must be positive
    if config.timeouts.graceful_shutdown <= 0.0 {
        errors.push(ConfigValidationError::InvalidValue {
            field: "timeouts.graceful_shutdown".to_string(),
            reason: "must be positive".to_string(),
        });
    }

    // Burst engine timestep must be positive
    if config.neural.burst_engine_timestep <= 0.0 {
        errors.push(ConfigValidationError::InvalidValue {
            field: "neural.burst_engine_timestep".to_string(),
            reason: "must be positive".to_string(),
        });
    }

    // Burst engine mode must be "inference" or "design"
    if config.burst_engine.mode != "inference" && config.burst_engine.mode != "design" {
        errors.push(ConfigValidationError::InvalidValue {
            field: "burst_engine.mode".to_string(),
            reason: "must be 'inference' or 'design'".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FeagiConfig;

    #[test]
    fn test_default_config_is_valid() {
        let config = FeagiConfig::default();
        let result = validate_config(&config);
        if let Err(e) = &result {
            eprintln!("Validation error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_port_range() {
        let mut config = FeagiConfig::default();
        config.api.port = 80; // Too low (< 1024)

        let result = validate_config(&config);
        assert!(result.is_err());
        
        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("api.port"));
            assert!(msg.contains("1024-65535"));
        }
    }

    #[test]
    fn test_port_conflict() {
        let mut config = FeagiConfig::default();
        config.api.port = 5555;
        config.agent.sensory_port = 5555; // Conflict!

        let result = validate_config(&config);
        assert!(result.is_err());
        
        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("Port conflict"));
            assert!(msg.contains("5555"));
        }
    }

    #[test]
    fn test_missing_required_field() {
        let mut config = FeagiConfig::default();
        config.api.host = String::new(); // Empty host

        let result = validate_config(&config);
        assert!(result.is_err());
        
        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("api.host"));
        }
    }

    #[test]
    fn test_invalid_gpu_memory_fraction() {
        let mut config = FeagiConfig::default();
        config.resources.gpu_memory_fraction = 1.5; // > 1.0

        let result = validate_config(&config);
        assert!(result.is_err());
        
        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("gpu_memory_fraction"));
            assert!(msg.contains("0.0 and 1.0"));
        }
    }

    #[test]
    fn test_invalid_burst_engine_mode() {
        let mut config = FeagiConfig::default();
        config.burst_engine.mode = "invalid_mode".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        
        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("burst_engine.mode"));
            assert!(msg.contains("inference"));
            assert!(msg.contains("design"));
        }
    }
}

