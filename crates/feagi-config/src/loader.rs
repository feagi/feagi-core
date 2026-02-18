// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Configuration file loading with override support
//!
//! This module implements the 3-tier configuration loading system:
//! 1. TOML file (base defaults)
//! 2. Environment variables (runtime overrides)
//! 3. CLI arguments (explicit user overrides)

use crate::{ConfigError, ConfigResult, FeagiConfig};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Find the FEAGI configuration file
///
/// Search order:
/// 1. `FEAGI_CONFIG_PATH` environment variable
/// 2. Current working directory: `./feagi_configuration.toml`
/// 3. Parent directory: `../feagi_configuration.toml`
/// 4. Workspace root (searches up to 5 levels)
///
/// # Errors
///
/// Returns `ConfigError::FileNotFound` if no config file is found in any location
pub fn find_config_file() -> ConfigResult<PathBuf> {
    // 1. Check environment variable first
    if let Ok(env_path) = env::var("FEAGI_CONFIG_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return Ok(path);
        } else {
            return Err(ConfigError::FileNotFound(format!(
                "Config file specified by FEAGI_CONFIG_PATH not found: {}",
                path.display()
            )));
        }
    }

    // 2. Search in common locations
    let mut search_paths = Vec::new();

    // Current directory
    if let Ok(cwd) = env::current_dir() {
        search_paths.push(cwd.join("feagi_configuration.toml"));

        // Parent directory
        if let Some(parent) = cwd.parent() {
            search_paths.push(parent.join("feagi_configuration.toml"));
        }

        // Search up to 5 levels for workspace root
        let mut current = cwd.clone();
        for _ in 0..5 {
            if let Some(parent) = current.parent() {
                search_paths.push(parent.join("feagi_configuration.toml"));
                current = parent.to_path_buf();
            }
        }
    }

    // Check each path
    for path in &search_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    // Not found
    let search_list = search_paths
        .iter()
        .map(|p| format!("  - {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Err(ConfigError::FileNotFound(format!(
        "FEAGI configuration file 'feagi_configuration.toml' not found in any of these locations:\n{}\n\nSet FEAGI_CONFIG_PATH environment variable to specify custom location.",
        search_list
    )))
}

/// Load configuration from TOML file
///
/// # Arguments
///
/// * `config_path` - Optional path to config file. If `None`, will search for config file.
/// * `cli_args` - Optional CLI argument overrides
///
/// # Returns
///
/// Complete `FeagiConfig` with all overrides applied
///
/// # Errors
///
/// Returns error if config file is not found, contains invalid TOML, or fails validation
pub fn load_config(
    config_path: Option<&Path>,
    cli_args: Option<&HashMap<String, String>>,
) -> ConfigResult<FeagiConfig> {
    // Find config file
    let config_file = if let Some(path) = config_path {
        path.to_path_buf()
    } else {
        find_config_file()?
    };

    // Read file
    let content = fs::read_to_string(&config_file)?;

    // Parse TOML
    let mut config: FeagiConfig = toml::from_str(&content)?;

    // Apply overrides in order
    apply_environment_overrides(&mut config);

    if let Some(cli) = cli_args {
        apply_cli_overrides(&mut config, cli);
    }

    Ok(config)
}

/// Apply environment variable overrides to configuration
///
/// Supported environment variables:
/// - `FEAGI_API_HOST` -> `api.bind_host` + `api.advertised_host` (sets both)
/// - `FEAGI_API_BIND_HOST` -> `api.bind_host`
/// - `FEAGI_API_ADVERTISED_HOST` -> `api.advertised_host`
/// - `FEAGI_API_PORT` -> `api.port`
/// - `FEAGI_API_WORKERS` -> `api.workers`
/// - `FEAGI_API_RELOAD` -> `api.reload`
/// - `FEAGI_ZMQ_HOST` -> `zmq.bind_host` + `zmq.advertised_host` (sets both)
/// - `FEAGI_ZMQ_BIND_HOST` -> `zmq.bind_host`
/// - `FEAGI_ZMQ_ADVERTISED_HOST` -> `zmq.advertised_host`
/// - `FEAGI_WS_HOST` -> `websocket.bind_host` + `websocket.advertised_host` (sets both)
/// - `FEAGI_WS_BIND_HOST` -> `websocket.bind_host`
/// - `FEAGI_WS_ADVERTISED_HOST` -> `websocket.advertised_host`
/// - `FEAGI_AGENT_HOST` -> `agent.bind_host` + `agent.advertised_host` (sets both)
/// - `FEAGI_AGENT_BIND_HOST` -> `agent.bind_host`
/// - `FEAGI_AGENT_ADVERTISED_HOST` -> `agent.advertised_host`
/// - `FEAGI_DATA_DIR` -> `system.data_dir`
/// - `FEAGI_MAX_CORES` -> `system.max_cores`
/// - `FEAGI_LOG_LEVEL` -> `system.log_level`
/// - `FEAGI_AGENT_DEFAULT_HOST` -> `agents.default_host`
pub fn apply_environment_overrides(config: &mut FeagiConfig) {
    // API settings
    if let Ok(value) = env::var("FEAGI_API_HOST") {
        config.api.bind_host = value.clone();
        config.api.advertised_host = value;
    }
    if let Ok(value) = env::var("FEAGI_API_BIND_HOST") {
        config.api.bind_host = value;
    }
    if let Ok(value) = env::var("FEAGI_API_ADVERTISED_HOST") {
        config.api.advertised_host = value;
    }
    if let Ok(value) = env::var("FEAGI_API_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.api.port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_API_WORKERS") {
        if let Ok(workers) = value.parse::<usize>() {
            config.api.workers = workers;
        }
    }
    if let Ok(value) = env::var("FEAGI_API_RELOAD") {
        config.api.reload =
            value.to_lowercase() == "true" || value == "1" || value.to_lowercase() == "yes";
    }

    // ZMQ settings
    if let Ok(value) = env::var("FEAGI_ZMQ_HOST") {
        config.zmq.bind_host = value.clone();
        config.zmq.advertised_host = value;
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_BIND_HOST") {
        config.zmq.bind_host = value;
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_ADVERTISED_HOST") {
        config.zmq.advertised_host = value;
    }

    // WebSocket settings
    if let Ok(value) = env::var("FEAGI_WS_HOST") {
        config.websocket.bind_host = value.clone();
        config.websocket.advertised_host = value;
    }
    if let Ok(value) = env::var("FEAGI_WS_BIND_HOST") {
        config.websocket.bind_host = value;
    }
    if let Ok(value) = env::var("FEAGI_WS_ADVERTISED_HOST") {
        config.websocket.advertised_host = value;
    }

    // Agent endpoint settings
    if let Ok(value) = env::var("FEAGI_AGENT_HOST") {
        config.agent.bind_host = value.clone();
        config.agent.advertised_host = value;
    }
    if let Ok(value) = env::var("FEAGI_AGENT_BIND_HOST") {
        config.agent.bind_host = value;
    }
    if let Ok(value) = env::var("FEAGI_AGENT_ADVERTISED_HOST") {
        config.agent.advertised_host = value;
    }

    // System settings
    if let Ok(value) = env::var("FEAGI_DATA_DIR") {
        config.system.data_dir = PathBuf::from(value);
    }
    if let Ok(value) = env::var("FEAGI_MAX_CORES") {
        if let Ok(cores) = value.parse::<usize>() {
            config.system.max_cores = cores;
        }
    }
    if let Ok(value) = env::var("FEAGI_LOG_LEVEL") {
        config.system.log_level = value;
    }

    // Agent settings
    if let Ok(value) = env::var("FEAGI_AGENT_DEFAULT_HOST") {
        config.agents.default_host = value;
    }

    // Port overrides
    if let Ok(value) = env::var("FEAGI_ZMQ_REQ_REP_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_req_rep_port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_PUB_SUB_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_pub_sub_port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_PUSH_PULL_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_push_pull_port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_SENSORY_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_sensory_port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_VISUALIZATION_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_visualization_port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_REST_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_rest_port = port;
        }
    }
    if let Ok(value) = env::var("FEAGI_ZMQ_MOTOR_PORT") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_motor_port = port;
        }
    }
}

/// Apply CLI argument overrides to configuration
///
/// # Arguments
///
/// * `config` - Configuration to modify
/// * `cli_args` - HashMap of CLI arguments (e.g., `{"api_host": "192.168.1.1", "api_port": "9000"}`)
pub fn apply_cli_overrides(config: &mut FeagiConfig, cli_args: &HashMap<String, String>) {
    // API settings
    if let Some(value) = cli_args.get("api_host") {
        config.api.bind_host = value.clone();
        config.api.advertised_host = value.clone();
    }
    if let Some(value) = cli_args.get("api_bind_host") {
        config.api.bind_host = value.clone();
    }
    if let Some(value) = cli_args.get("api_advertised_host") {
        config.api.advertised_host = value.clone();
    }
    if let Some(value) = cli_args.get("api_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.api.port = port;
        }
    }
    if let Some(value) = cli_args.get("api_workers") {
        if let Ok(workers) = value.parse::<usize>() {
            config.api.workers = workers;
        }
    }

    // ZMQ settings
    if let Some(value) = cli_args.get("zmq_host") {
        config.zmq.bind_host = value.clone();
        config.zmq.advertised_host = value.clone();
    }
    if let Some(value) = cli_args.get("zmq_bind_host") {
        config.zmq.bind_host = value.clone();
    }
    if let Some(value) = cli_args.get("zmq_advertised_host") {
        config.zmq.advertised_host = value.clone();
    }

    // WebSocket settings
    if let Some(value) = cli_args.get("ws_host") {
        config.websocket.bind_host = value.clone();
        config.websocket.advertised_host = value.clone();
    }
    if let Some(value) = cli_args.get("ws_bind_host") {
        config.websocket.bind_host = value.clone();
    }
    if let Some(value) = cli_args.get("ws_advertised_host") {
        config.websocket.advertised_host = value.clone();
    }

    // Agent endpoint settings
    if let Some(value) = cli_args.get("agent_host") {
        config.agent.bind_host = value.clone();
        config.agent.advertised_host = value.clone();
    }
    if let Some(value) = cli_args.get("agent_bind_host") {
        config.agent.bind_host = value.clone();
    }
    if let Some(value) = cli_args.get("agent_advertised_host") {
        config.agent.advertised_host = value.clone();
    }

    // System settings
    if let Some(value) = cli_args.get("debug") {
        config.system.debug = value.to_lowercase() == "true" || value == "1";
    }
    if let Some(value) = cli_args.get("log_level") {
        config.system.log_level = value.clone();
    }

    // Port overrides
    if let Some(value) = cli_args.get("zmq_req_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_req_rep_port = port;
        }
    }
    if let Some(value) = cli_args.get("zmq_pub_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_pub_sub_port = port;
        }
    }
    if let Some(value) = cli_args.get("zmq_push_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_push_pull_port = port;
        }
    }
    if let Some(value) = cli_args.get("zmq_sensory_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_sensory_port = port;
        }
    }
    if let Some(value) = cli_args.get("zmq_visualization_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_visualization_port = port;
        }
    }
    if let Some(value) = cli_args.get("zmq_rest_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_rest_port = port;
        }
    }
    if let Some(value) = cli_args.get("zmq_motor_port") {
        if let Ok(port) = value.parse::<u16>() {
            config.ports.zmq_motor_port = port;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_find_config_file_env_var() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("custom_config.toml");
        File::create(&config_path).unwrap();

        env::set_var("FEAGI_CONFIG_PATH", config_path.to_str().unwrap());
        let result = find_config_file();
        env::remove_var("FEAGI_CONFIG_PATH");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), config_path);
    }

    #[test]
    fn test_load_minimal_config() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let saved_api_host = env::var("FEAGI_API_HOST").ok();
        let saved_api_port = env::var("FEAGI_API_PORT").ok();
        env::remove_var("FEAGI_API_HOST");
        env::remove_var("FEAGI_API_PORT");
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("feagi_configuration.toml");

        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "[system]").unwrap();
        writeln!(file, "max_cores = 4").unwrap();
        writeln!(file, "[api]").unwrap();
        writeln!(file, "port = 9000").unwrap();

        let config = load_config(Some(&config_path), None).unwrap();

        assert_eq!(config.system.max_cores, 4);
        assert_eq!(config.api.port, 9000);

        if let Some(value) = saved_api_host {
            env::set_var("FEAGI_API_HOST", value);
        }
        if let Some(value) = saved_api_port {
            env::set_var("FEAGI_API_PORT", value);
        }
    }

    #[test]
    fn test_environment_overrides() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        let mut config = FeagiConfig::default();

        env::set_var("FEAGI_API_HOST", "192.168.1.100");
        env::set_var("FEAGI_API_PORT", "9999");
        env::set_var("FEAGI_AGENT_HOST", "10.0.0.5");

        apply_environment_overrides(&mut config);

        env::remove_var("FEAGI_API_HOST");
        env::remove_var("FEAGI_API_PORT");
        env::remove_var("FEAGI_AGENT_HOST");

        assert_eq!(config.api.bind_host, "192.168.1.100");
        assert_eq!(config.api.advertised_host, "192.168.1.100");
        assert_eq!(config.api.port, 9999);
        assert_eq!(config.agent.bind_host, "10.0.0.5");
        assert_eq!(config.agent.advertised_host, "10.0.0.5");
    }

    #[test]
    fn test_cli_overrides() {
        let mut config = FeagiConfig::default();
        let mut cli_args = HashMap::new();
        cli_args.insert("api_host".to_string(), "10.0.0.1".to_string());
        cli_args.insert("api_port".to_string(), "7777".to_string());
        cli_args.insert("agent_host".to_string(), "10.0.0.7".to_string());

        apply_cli_overrides(&mut config, &cli_args);

        assert_eq!(config.api.bind_host, "10.0.0.1");
        assert_eq!(config.api.advertised_host, "10.0.0.1");
        assert_eq!(config.api.port, 7777);
        assert_eq!(config.agent.bind_host, "10.0.0.7");
        assert_eq!(config.agent.advertised_host, "10.0.0.7");
    }

    #[test]
    fn test_override_precedence() {
        let _env_lock = ENV_LOCK.lock().unwrap();
        // Test that CLI overrides take precedence over environment variables
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("feagi_configuration.toml");

        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "[api]").unwrap();
        writeln!(file, "bind_host = \"file-bind-host\"").unwrap();
        writeln!(file, "advertised_host = \"file-adv-host\"").unwrap();
        writeln!(file, "port = 8000").unwrap();

        env::set_var("FEAGI_API_HOST", "env-host");
        env::set_var("FEAGI_API_PORT", "9000");

        let mut cli_args = HashMap::new();
        cli_args.insert("api_host".to_string(), "cli-host".to_string());

        let config = load_config(Some(&config_path), Some(&cli_args)).unwrap();

        env::remove_var("FEAGI_API_HOST");
        env::remove_var("FEAGI_API_PORT");

        // CLI wins for host, env wins for port (no CLI override)
        assert_eq!(config.api.bind_host, "cli-host");
        assert_eq!(config.api.advertised_host, "cli-host");
        assert_eq!(config.api.port, 9000);
    }
}
