# feagi-transports

Transport abstraction layer for FEAGI, providing a unified interface for multiple transport protocols including ZMQ, UDP, and shared memory.

## Features

- **ZMQ Support**: Complete implementation of ZMQ socket patterns (ROUTER, DEALER, PUB, SUB, PUSH, PULL)
- **WebSocket Support**: Full async WebSocket implementation for all patterns (NEW!)
- **Client & Server**: Both sides of the communication in a single crate
- **Feature Flags**: Granular control over which transports and roles are compiled
- **Type-Safe**: Strong typing and trait-based abstractions
- **Cross-Platform**: Works on Linux, macOS, Windows, Docker, embedded systems, and web browsers
- **Well-Tested**: Comprehensive integration tests and examples

## Installation

```toml
# Server-side (FEAGI core)
[dependencies]
feagi-transports = { version = "2.0", features = ["server"] }

# Client-side (Rust agents)
[dependencies]
feagi-transports = { version = "2.0", features = ["client"] }

# Both
[dependencies]
feagi-transports = { version = "2.0", features = ["all"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `zmq-server` | ZMQ server patterns (ROUTER, PUB, PULL) |
| `zmq-client` | ZMQ client patterns (DEALER, SUB, PUSH) |
| `websocket-server` | WebSocket server patterns (ROUTER, PUB, PULL) |
| `websocket-client` | WebSocket client patterns (DEALER, SUB, PUSH) |
| `udp-server` | UDP server (planned) |
| `udp-client` | UDP client (planned) |
| `shm-server` | Shared memory server (planned) |
| `shm-client` | Shared memory client (planned) |
| `zmq` | Both ZMQ server and client |
| `websocket` | Both WebSocket server and client |
| `udp` | Both UDP server and client |
| `shm` | Both SHM server and client |
| `server` | All server implementations |
| `client` | All client implementations |
| `all` | Everything |

## Quick Start

### Request-Reply Pattern

#### Server (ROUTER)

```rust
use feagi_transports::prelude::*;

let mut server = ZmqRouter::with_address("tcp://*:5555")?;
server.start()?;

loop {
    let (request, reply_handle) = server.receive()?;
    println!("Received: {:?}", request);
    reply_handle.send(b"OK")?;
}
```

#### Client (DEALER)

```rust
use feagi_transports::prelude::*;

let mut client = ZmqDealer::with_address("tcp://localhost:5555")?;
client.start()?;

let response = client.request(b"Hello!")?;
println!("Response: {:?}", response);
```

### Publish-Subscribe Pattern

#### Publisher (PUB)

```rust
use feagi_transports::prelude::*;

let mut publisher = ZmqPub::with_address("tcp://*:5556")?;
publisher.start()?;

loop {
    publisher.publish(b"topic", b"data")?;
    std::thread::sleep(std::time::Duration::from_millis(100));
}
```

#### Subscriber (SUB)

```rust
use feagi_transports::prelude::*;

let mut subscriber = ZmqSub::with_address("tcp://localhost:5556")?;
subscriber.start()?;
subscriber.subscribe(b"topic")?;

loop {
    let (topic, data) = subscriber.receive()?;
    println!("Received: {:?} - {:?}", topic, data);
}
```

### Push-Pull Pattern

#### Pull (Receiver)

```rust
use feagi_transports::prelude::*;

let mut pull = ZmqPull::with_address("tcp://*:5557")?;
pull.start()?;

loop {
    let data = pull.pull()?;
    println!("Received: {:?}", data);
}
```

#### Push (Sender)

```rust
use feagi_transports::prelude::*;

let mut push = ZmqPush::with_address("tcp://localhost:5557")?;
push.start()?;

push.push(b"data")?;
```

## Examples

Run the examples to see the transports in action:

```bash
# Terminal 1: Start server
cargo run --example request_reply_server --features=zmq-server

# Terminal 2: Start client
cargo run --example request_reply_client --features=zmq-client
```

```bash
# Terminal 1: Start publisher
cargo run --example publisher --features=zmq-server

# Terminal 2: Start subscriber
cargo run --example subscriber --features=zmq-client
```

## Architecture

```
feagi-transports/
├── common/          # Shared types (errors, configs, messages)
├── traits/          # Transport-agnostic interfaces
├── zmq/
│   ├── server/      # ROUTER, PUB, PULL
│   └── client/      # DEALER, SUB, PUSH
├── udp/             # Planned
└── shm/             # Planned
```

## Design Principles

1. **Transport Agnostic**: Business logic doesn't depend on transport implementation
2. **Zero-Copy**: Where possible, data is passed by reference
3. **Symmetric API**: Client and server have similar interfaces
4. **Fail-Fast**: Errors are explicit, no hidden fallbacks
5. **Feature-Gated**: Only compile what you need

## Use Cases

### FEAGI Core (Server)
- **REST API Control**: ROUTER for request-reply
- **Brain Visualization**: PUB for broadcasting neuron activity
- **Sensory Input**: PULL for receiving data from agents

### Rust Agents (Client)
- **Sensory Data**: PUSH to send sensor readings
- **Motor Commands**: SUB to receive motor instructions
- **API Calls**: DEALER for control plane communication

## Testing

```bash
# Run all tests
cargo test -p feagi-transports --features all

# Run integration tests only
cargo test -p feagi-transports --features all --test zmq_integration_test
```

## Performance

ZMQ transports are highly optimized:
- **Latency**: <100μs for local IPC
- **Throughput**: >1M messages/second
- **Scalability**: Supports thousands of concurrent clients

## WebSocket Transport (NEW!)

### Quick Start: WebSocket Pub/Sub

#### Server (Publisher)

```rust
use feagi_transports::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut publisher = WsPub::with_address("127.0.0.1:9050").await?;
    publisher.start_async().await?;
    
    loop {
        publisher.publish(b"topic", b"data")?;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
```

#### Client (Subscriber)

```rust
use feagi_transports::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut subscriber = WsSub::with_address("ws://127.0.0.1:9050").await?;
    subscriber.start_async().await?;
    subscriber.subscribe(b"topic")?;
    
    loop {
        let (topic, data) = subscriber.receive_timeout(1000)?;
        println!("Received: {:?} - {:?}", topic, data);
    }
}
```

### WebSocket Examples

Run the WebSocket examples to see all patterns in action:

```bash
# Terminal 1: Start WebSocket publisher
cargo run --example ws_publisher --features=websocket-server

# Terminal 2: Start WebSocket subscriber
cargo run --example ws_subscriber --features=websocket-client
```

```bash
# Terminal 1: Start WebSocket pull server
cargo run --example ws_pull_server --features=websocket-server

# Terminal 2: Start WebSocket push client
cargo run --example ws_push_client --features=websocket-client
```

```bash
# Terminal 1: Start WebSocket router server
cargo run --example ws_router_server --features=websocket-server

# Terminal 2: Start WebSocket dealer client
cargo run --example ws_dealer_client --features=websocket-client
```

### WebSocket Features

- **Async-first**: Built on tokio and tokio-tungstenite
- **Browser-compatible**: Standard WebSocket protocol
- **Topic filtering**: PUB/SUB pattern with topic-based routing
- **Multiple clients**: Supports many concurrent connections
- **Binary & Text**: Handles both message types
- **Automatic reconnection**: Robust connection handling

### When to Use WebSocket vs ZMQ

**Use WebSocket when:**
- Need browser/web client support
- Cross-platform compatibility is critical
- Simple deployment (no external dependencies)
- HTTP/HTTPS infrastructure already in place
- Working with firewalls (WebSocket uses standard ports)

**Use ZMQ when:**
- Maximum performance is required
- Complex routing patterns needed
- Working in pure backend environments
- Need advanced ZMQ features (multipart, etc.)

## Future Work

- [x] WebSocket transport implementation (COMPLETED!)
- [ ] UDP transport implementation
- [ ] Shared memory transport for embedded systems
- [ ] TLS/WSS encryption support for WebSocket
- [ ] gRPC transport adapter
- [ ] WebRTC for browser-based agents
- [ ] MQTT for IoT devices

## Contributing

This crate follows FEAGI's architecture compliance rules:
- No hardcoded values (hosts, ports, timeouts)
- All configuration via `TransportConfig`
- Cross-platform compatibility
- No fallbacks in production code

## License

Apache-2.0

## Related Crates

- `feagi-io`: Peripheral Nervous System (uses server features)
- `feagi-api`: REST API layer (uses server features)
- `feagi-agent-sdk`: Rust agent library (uses client features)




