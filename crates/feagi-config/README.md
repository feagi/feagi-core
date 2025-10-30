# feagi-config

Cross-platform, type-safe configuration loader for FEAGI with support for TOML files, environment variable overrides, and CLI argument overrides.

## Features

- ✅ **Type-safe**: All configuration values are strongly typed Rust structs
- ✅ **3-tier override system**: TOML file → Environment variables → CLI arguments
- ✅ **Automatic file discovery**: Searches common locations for `feagi_configuration.toml`
- ✅ **Comprehensive validation**: Port ranges, conflicts, required fields, value ranges
- ✅ **Cross-platform**: Works on Linux, macOS, Windows, and embedded (with `no_std`)
- ✅ **Architecture compliance**: Enforces FEAGI 2.0 "no hardcoding" principles

## Usage

### Basic Usage

```rust
use feagi_config::{load_config, validate_config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration with automatic file discovery and overrides
    let config = load_config(None, None)?;
    
    // Validate configuration
    validate_config(&config)?;
    
    // Access type-safe configuration values
    println!("API Host: {}", config.api.host);
    println!("API Port: {}", config.api.port);
    println!("ZMQ Host: {}", config.zmq.host);
    
    Ok(())
}
```

### With CLI Argument Overrides

```rust
use feagi_config::load_config;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prepare CLI overrides
    let mut cli_args = HashMap::new();
    cli_args.insert("api_host".to_string(), "192.168.1.100".to_string());
    cli_args.insert("api_port".to_string(), "9000".to_string());
    
    // Load with overrides
    let config = load_config(None, Some(&cli_args))?;
    
    assert_eq!(config.api.host, "192.168.1.100");
    assert_eq!(config.api.port, 9000);
    
    Ok(())
}
```

### Environment Variable Overrides

```bash
# Set environment variables
export FEAGI_API_HOST=0.0.0.0
export FEAGI_API_PORT=8080
export FEAGI_ZMQ_HOST=127.0.0.1

# Run your application
cargo run
```

```rust
use feagi_config::load_config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Environment variables automatically applied
    let config = load_config(None, None)?;
    
    // config.api.host will be "0.0.0.0" from environment
    // config.api.port will be 8080 from environment
    
    Ok(())
}
```

## Configuration File

Place `feagi_configuration.toml` in one of these locations:
1. Path specified by `FEAGI_CONFIG_PATH` environment variable
2. Current working directory
3. Parent directory
4. Workspace root (searches up to 5 levels)

Example `feagi_configuration.toml`:

```toml
[system]
max_cores = 0  # 0 = auto-detect
debug = true
log_level = "WARNING"

[api]
host = "0.0.0.0"
port = 8000
workers = 1

[ports]
zmq_req_rep_port = 5555
zmq_pub_sub_port = 5556
zmq_sensory_port = 5558

[timeouts]
graceful_shutdown = 8.0
service_startup = 3.0

[neural]
burst_engine_timestep = 0.1
batch_size = 1000
```

## Supported Environment Variables

| Environment Variable | Config Path | Type |
|---------------------|-------------|------|
| `FEAGI_API_HOST` | `api.host` | String |
| `FEAGI_API_PORT` | `api.port` | u16 |
| `FEAGI_API_WORKERS` | `api.workers` | usize |
| `FEAGI_ZMQ_HOST` | `zmq.host` | String |
| `FEAGI_DATA_DIR` | `system.data_dir` | PathBuf |
| `FEAGI_MAX_CORES` | `system.max_cores` | usize |
| `FEAGI_LOG_LEVEL` | `system.log_level` | String |
| `FEAGI_AGENT_DEFAULT_HOST` | `agents.default_host` | String |
| `FEAGI_ZMQ_REQ_REP_PORT` | `ports.zmq_req_rep_port` | u16 |
| `FEAGI_ZMQ_PUB_SUB_PORT` | `ports.zmq_pub_sub_port` | u16 |
| `FEAGI_ZMQ_SENSORY_PORT` | `ports.zmq_sensory_port` | u16 |
| ... and more |

## Validation

The validation module checks for:
- ✅ Port ranges (1024-65535 for non-root ports)
- ✅ Port conflicts (no two services using the same port)
- ✅ Required fields (hosts cannot be empty)
- ✅ Value ranges (e.g., GPU memory fraction between 0.0-1.0)
- ✅ Valid enums (e.g., burst_engine.mode must be "inference" or "design")

## Feature Flags

### `std` (default)
Full configuration loading with TOML file parsing and filesystem access.

```toml
[dependencies]
feagi-config = "2.0"
```

### `no_std`
For embedded/RTOS targets. Configuration must be provided at compile-time.

```toml
[dependencies]
feagi-config = { version = "2.0", default-features = false }
```

### `wasm`
For WebAssembly targets. Configuration provided via JavaScript.

```toml
[dependencies]
feagi-config = { version = "2.0", default-features = false, features = ["wasm"] }
```

## Architecture Compliance

This crate enforces FEAGI 2.0 architecture principles:

❌ **FORBIDDEN** (Hardcoded values):
```rust
// DON'T DO THIS
let host = "127.0.0.1";
let timeout = 30;
```

✅ **REQUIRED** (Configuration-driven):
```rust
// DO THIS INSTEAD
let config = load_config(None, None)?;
let host = &config.api.host;
let timeout = config.timeouts.graceful_shutdown;
```

## License

Apache-2.0

## Authors

Neuraville Inc. <feagi@neuraville.com>

