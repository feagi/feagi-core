# Per-Crate Debug Flags Usage Guide

## Overview

The `feagi-observability` crate provides per-crate debug flag support, allowing you to enable debug logging for specific crates using CLI flags like `--debug-feagi-api` or `--debug-feagi-burst-engine`.

## Usage in Binaries

### Example: `feagi` binary

```rust
use clap::Parser;
use feagi_observability::{CrateDebugFlags, parse_debug_flags};

#[derive(Parser, Debug)]
#[command(name = "feagi")]
struct Args {
    // ... other args ...
    
    /// Enable debug logging for specific crates
    /// Example: --debug-feagi-api --debug-feagi-burst-engine
    #[arg(long, action = clap::ArgAction::Append)]
    debug: Vec<String>,
    
    /// Enable debug logging for all crates
    #[arg(long)]
    debug_all: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Parse debug flags
    let debug_flags = if args.debug_all {
        // Create flags with all crates enabled
        let mut flags = CrateDebugFlags::default();
        for crate_name in feagi_observability::KNOWN_CRATES {
            flags.enabled_crates.insert(crate_name.to_string(), true);
        }
        flags
    } else {
        // Parse individual flags
        let mut cli_args = vec!["feagi".to_string()]; // Program name
        for crate_name in &args.debug {
            cli_args.push(format!("--debug-{}", crate_name));
        }
        CrateDebugFlags::from_args(cli_args)
    };
    
    // Initialize observability with debug flags
    let filter = debug_flags.to_filter_string();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&filter))
        .init();
    
    // Your application code...
    
    Ok(())
}
```

### Simpler Approach: Use `parse_debug_flags()`

```rust
use feagi_observability::parse_debug_flags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically parses from env::args() and FEAGI_DEBUG env var
    let debug_flags = parse_debug_flags();
    
    // Initialize logging with debug flags
    let filter = debug_flags.to_filter_string();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&filter))
        .init();
    
    // Your application code...
    
    Ok(())
}
```

## CLI Flags

### Single Crate
```bash
./feagi --debug-feagi-api
```

### Multiple Crates
```bash
./feagi --debug-feagi-api --debug-feagi-burst-engine --debug-feagi-bdu
```

### All Crates
```bash
./feagi --debug-all
```

## Environment Variable

You can also use the `FEAGI_DEBUG` environment variable:

```bash
# Single crate
FEAGI_DEBUG=feagi-api ./feagi

# Multiple crates (comma-separated)
FEAGI_DEBUG=feagi-api,feagi-burst-engine ./feagi

# All crates
FEAGI_DEBUG=all ./feagi
```

## Available Crates

- `feagi-api`
- `feagi-burst-engine`
- `feagi-bdu`
- `feagi-services`
- `feagi-evo`
- `feagi-config`
- `feagi-io`
- `feagi-transports`
- `feagi-agent`
- `feagi-state-manager`
- `feagi-plasticity`
- `feagi-connectome-serialization`

## Integration Examples

### Example 1: feagi binary

```rust
use feagi_observability::parse_debug_flags;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let debug_flags = parse_debug_flags();
    
    // Initialize tracing with crate-specific debug levels
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            &debug_flags.to_filter_string()
        ))
        .init();
    
    // Check if specific crate debugging is enabled
    if debug_flags.is_enabled("feagi-api") {
        println!("Debug logging enabled for feagi-api");
    }
    
    // Your application code...
    
    Ok(())
}
```

### Example 2: feagi-inference-engine

```rust
use feagi_observability::{parse_debug_flags, CrateDebugFlags};

#[derive(Parser)]
struct Args {
    #[arg(long, action = clap::ArgAction::Append)]
    debug: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Build debug flags from CLI args
    let mut cli_args = vec!["feagi-inference-engine".to_string()];
    for crate_name in &args.debug {
        cli_args.push(format!("--debug-{}", crate_name));
    }
    let debug_flags = CrateDebugFlags::from_args(cli_args);
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            &debug_flags.to_filter_string()
        ))
        .init();
    
    // Your application code...
    
    Ok(())
}
```

## Benefits

1. **Granular Control**: Enable debug logging only for crates you're debugging
2. **Performance**: No overhead from debug logging in other crates
3. **Flexibility**: Use CLI flags or environment variables
4. **Consistency**: Same pattern across all FEAGI binaries



