# `feagi-agent` (Rust)

Rust client library for building FEAGI agents.

This crate is designed to remain **runtime-agnostic** and compatible with future targets such as
WASM and Embassy/RTOS environments.

## What this crate provides

- **Poll-based client primitives** (registration/control + data channels) built on `feagi-io`.
- **A runtime-agnostic session orchestration state machine** (`clients::SessionStateMachine`).
- **Optional Tokio adapter layer** (`clients::async_helpers::tokio_generic_implementations`) behind the
  `agent-client-asynchelper-tokio` feature.

## Quick Start

```rust
use feagi_agent::clients::async_helpers::tokio_generic_implementations::{
    TokioDriverConfig, TokioEmbodimentAgent,
};
use feagi_agent::clients::SessionTimingConfig;
use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken};
use feagi_io::protocol_implementations::zmq::FeagiZmqClientRequesterProperties;
use std::time::Duration;

// Registration endpoint (example uses ZMQ; WebSocket is also supported).
let registration_endpoint = "<transport endpoint string>";
let registration_properties = Box::new(FeagiZmqClientRequesterProperties::new(registration_endpoint)?);

let agent_descriptor = AgentDescriptor::new("manufacturer", "agent_name", 1)?;
let auth_token = AuthToken::new([0u8; 32]);

let driver = TokioDriverConfig {
    poll_interval: Duration::from_millis(5),
    timing: SessionTimingConfig {
        heartbeat_interval_ms: 1000,
        registration_deadline_ms: Some(10_000),
    },
    sensory_rate_negotiation: None,
};

let requested = vec![
    AgentCapabilities::SendSensorData,
    AgentCapabilities::ReceiveMotorData,
];

let mut agent = TokioEmbodimentAgent::new_connect_and_register(
    registration_properties,
    agent_descriptor,
    auth_token,
    requested,
    driver,
).await?;

// Drive maintenance explicitly.
agent.tick()?;
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
feagi-agent = { path = "../feagi-agent" }
```

## Examples

This crate includes examples under `examples/`. Some examples require enabling feature flags.

## Architecture

### Session orchestration

The preferred orchestration abstraction is `clients::SessionStateMachine`:

- It is **pure logic** (no I/O, no sleeps).
- A driver (Tokio/WASM/Embassy/RTOS) feeds events and executes actions.
- ZMQ and WebSocket are both supported via `TransportProtocolEndpoint`.

## Error Handling

Errors use `feagi_agent::FeagiAgentError` and `feagi_io::FeagiNetworkError` (via `From` conversions).

## Testing

Run unit tests:
```bash
cargo test
```

## License

Apache-2.0

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

