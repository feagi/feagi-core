# feagi-io

FEAGI Peripheral Nervous System - Agent I/O, registration, and communication.

## Overview

Handles communication between FEAGI brain and agents:
- Agent registration and heartbeat
- Sensory input injection
- Motor output extraction
- Visualization data streaming
- ZMQ and shared memory transports

## Installation

```toml
[dependencies]
feagi-io = "2.0"
```

## Features

- `zmq-server` - ZMQ server support
- `websocket-server` - WebSocket support for brain visualizer
- `shm` - Shared memory for high-performance local communication

## Usage

```rust
use feagi_io::{AgentRegistry, IOConfig};

// Register agents and handle I/O
```

## Note

Per architecture docs, this crate will be moved to a separate `feagi-io` repository in the future.

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

