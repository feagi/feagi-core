# FEAGI Agent SDK - User Guide

**Version:** 0.1.0-beta  
**Author:** Neuraville Inc.  
**License:** Apache-2.0

---

## Welcome to FEAGI Agent SDK

The FEAGI Agent SDK is a world-class Rust library for building intelligent agents that connect to FEAGI (Framework for Evolutionary Artificial General Intelligence). Whether you're building robots, simulations, or custom AI systems, this SDK provides the tools you need.

### What You'll Learn

- **Core Concepts**: Understand agents, encoders, and decoders
- **Quick Start**: Build your first agent in minutes
- **Video Processing**: Send camera feeds to FEAGI
- **Text Processing**: Process language with FEAGI
- **Motor Control**: Receive commands from FEAGI
- **Multi-Modal Agents**: Combine multiple sensory modalities
- **Production Deployment**: Best practices for real-world systems

---

## Table of Contents

1. [Installation](#installation)
2. [Core Concepts](#core-concepts)
3. [Quick Start](#quick-start)
4. [Video Agents](#video-agents)
5. [Text Agents](#text-agents)
6. [Motor Agents](#motor-agents)
7. [Multi-Modal Agents](#multi-modal-agents)
8. [Production Best Practices](#production-best-practices)
9. [API Reference](#api-reference)
10. [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

- **Rust**: Version 1.75 or later
- **FEAGI Server**: Running and accessible
- **Operating System**: Linux, macOS, or Windows

### Add to Your Project

```toml
# Cargo.toml
[dependencies]
feagi-agent = "0.1"  # Full SDK (recommended for most users)

# Or for minimal embedded use:
feagi-agent = { version = "0.1", default-features = false }
```

### Verify Installation

```bash
cargo build
```

---

## Core Concepts

### Architecture Overview

```
Your Application
       ↓
┌──────────────────────────────────┐
│     FEAGI Agent SDK             │
│                                  │
│  ┌────────────┐  ┌────────────┐ │
│  │    Core    │  │    SDK     │ │
│  │ (Protocol) │  │  (Tools)   │ │
│  └─────┬──────┘  └─────┬──────┘ │
└────────┼────────────────┼────────┘
         ↓                ↓
    ┌────────────────────────┐
    │    FEAGI Server        │
    │  (Neural Processing)   │
    └────────────────────────┘
```

### Key Components

#### 1. **Core Module** (Low-Level)

The core handles the FEAGI protocol:
- Agent registration
- Heartbeat keepalive
- ZMQ communication
- Connection management

**Use core directly when:**
- Building embedded systems
- Implementing custom protocols
- Need minimal dependencies

```rust
use feagi_agent::core::{AgentClient, AgentConfig, AgentType};
```

#### 2. **SDK Module** (High-Level)

The SDK provides building blocks:
- **Encoders**: Convert your data → FEAGI format
- **Decoders**: Convert FEAGI output → your format
- **Topology Cache**: Fetch cortical area dimensions
- **Controllers**: Optional patterns for common use cases

**Use SDK when:**
- Building standard agents (video, text, audio)
- Want ready-made encoders/decoders
- Need topology management

---

## SDK Facade / Stability Model (Commercial Use)

For production and commercial deployments, controller code must remain stable even as FEAGI internals evolve.
The SDK therefore follows a **hybrid facade** strategy:

- **Controllers should depend only on `feagi-agent`**.
- FEAGI “model types” used by controllers (IDs, descriptors, frame types) are exposed via:
  - `feagi_agent::sdk::types::*`
- Internal crates like `feagi-structures` and `feagi-sensorimotor` may evolve, but as long as the
  SDK surface remains stable, controller code should not need changes.

**Rule of thumb:** if you feel you need to import `feagi_structures::*` or `feagi_sensorimotor::*`
from controller code, request that the required type/function be added to `sdk::types` instead.

### Stability Contract (SemVer + Deprecations + Contract Tests)

To treat `feagi_agent::sdk::types` as a **stable API contract** suitable for commercial use:

- **SemVer rules**
  - **PATCH**: bugfixes only (no public API changes).
  - **MINOR**: additive-only changes to the public API (new exports, new functions).
  - **MAJOR**: any breaking change (removals/renames/signature changes, or semantic behavior changes).

- **Deprecations**
  - Items are **not removed immediately**. They are marked with `#[deprecated(since = \"x.y.z\", note = \"...\")]`.
  - A supported replacement is provided.
  - Removal happens only on the next **MAJOR** release after a deprecation window.

- **Contract tests**
  - We maintain Rust tests that assert the **SDK surface compiles** against `sdk::types`.
  - We maintain parsing/shape tests for FEAGI HTTP responses that the SDK relies on (e.g., topology schema),
    so internal/backend refactors don't silently break controller behavior.

```rust
use feagi_agent::sdk::sensory::video::VideoEncoder;
use feagi_agent::sdk::base::TopologyCache;
```

### Data Flow

**Sensory Agent** (Sensor → FEAGI):
```
Camera → ImageFrame → VideoEncoder → AgentClient → FEAGI
```

**Motor Agent** (FEAGI → Actuator):
```
FEAGI → AgentClient → PerceptionDecoder → Commands → Robot
```

---

## Quick Start

### Your First Agent (5 Minutes)

Let's build a simple sensory agent that sends random data to FEAGI:

```rust
use feagi_agent::core::{AgentClient, AgentConfig, AgentType, SensoryCapability};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure the agent
    let mut cortical_mappings = HashMap::new();
    cortical_mappings.insert("aVJpbWcKAAAA".to_string(), 0);  // Example cortical ID
    
    let config = AgentConfig::new("my-first-agent", AgentType::Sensory)
        .with_registration_endpoint("tcp://localhost:30001")
        .with_sensory_endpoint("tcp://localhost:5555")
        .with_heartbeat_interval(5.0)
        .with_sensory_capability(60.0, None, cortical_mappings);
    
    // 2. Create and connect the client
    let mut client = AgentClient::new(config)?;
    client.connect()?;
    println!("✓ Connected to FEAGI!");
    
    // 3. Send data (in a real app, this would be in a loop)
    let dummy_data = vec![1, 2, 3, 4, 5];  // Replace with encoded sensory data
    client.send_sensory_bytes(dummy_data)?;
    println!("✓ Sent data to FEAGI!");
    
    // Client automatically deregisters on drop
    Ok(())
}
```

**Run it:**
```bash
cargo run
```

**What just happened?**
1. You created an agent configuration
2. Connected to FEAGI (with automatic registration)
3. Sent binary data to FEAGI
4. Disconnected gracefully (automatic on drop)

---

## Video Agents

### Complete Video Agent Example

This example shows how to build a video camera agent using the SDK's `VideoEncoder`:

```rust
use feagi_agent::core::{AgentClient, AgentConfig};
use feagi_agent::sdk::sensory::video::{VideoEncoder, VideoEncoderConfig, VideoEncodingStrategy};
use feagi_agent::sdk::base::TopologyCache;
use feagi_sensorimotor::data_types::ImageFrame;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎥 Starting Video Agent...");
    
    // 1. Configure the video encoder
    let encoder_config = VideoEncoderConfig {
        agent_id: "video-camera-01".to_string(),
        cortical_group_id: 0,
        encoding_strategy: VideoEncodingStrategy::SimpleVision,
        source_width: 640,
        source_height: 480,
        feagi_host: "localhost".to_string(),
        feagi_api_port: 8080,
        feagi_zmq_registration_port: 30001,
        feagi_zmq_sensory_port: 5555,
        feagi_zmq_motor_port: 5564,
        feagi_tick_hz: 60,
        feagi_heartbeat_interval_s: 5.0,
        feagi_connection_timeout_ms: 5000,
        feagi_registration_retries: 3,
        diff_threshold: 10,
        brightness: 0,
        contrast: 1.0,
    };
    
    // 2. Create topology cache (fetches cortical dimensions from FEAGI)
    let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
    println!("✓ Created topology cache");
    
    // 3. Create video encoder (fetches topologies automatically)
    let encoder = VideoEncoder::new(encoder_config.clone(), &topology_cache).await?;
    println!("✓ Video encoder ready");
    
    // 4. Create and connect agent client
    let agent_config = encoder_config.to_agent_config()?;
    let mut client = AgentClient::new(agent_config)?;
    client.connect()?;
    println!("✓ Connected to FEAGI");
    
    // 5. Main loop: capture frames and send to FEAGI
    loop {
        // Get frame from camera (pseudo-code, replace with actual camera)
        let frame = capture_frame_from_camera(640, 480)?;
        
        // Encode frame to FEAGI format
        let encoded = encoder.encode(&frame)?;
        
        // Send to FEAGI
        client.send_sensory_bytes(encoded)?;
        
        // Control frame rate (60 FPS)
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}

// Placeholder - replace with actual camera code
fn capture_frame_from_camera(width: u32, height: u32) -> Result<ImageFrame, Box<dyn std::error::Error>> {
    // Your camera capture logic here
    todo!("Implement camera capture")
}
```

### Video Encoding Strategies

#### Simple Vision (Full Frame)

Best for:
- Single camera feeds
- Full-frame analysis
- Simple scenarios

```rust
let config = VideoEncoderConfig {
    encoding_strategy: VideoEncodingStrategy::SimpleVision,
    // ...
};
```

FEAGI receives: **1 cortical area** (iimg) with full frame

#### Segmented Vision (with Gaze)

Best for:
- Foveated vision (human-like attention)
- Multi-resolution processing
- Robotic vision systems

```rust
let config = VideoEncoderConfig {
    encoding_strategy: VideoEncodingStrategy::SegmentedVision,
    // ...
};

// Update gaze dynamically
encoder.set_gaze(0.5, 0.5, 0.75)?;  // center, 75% modulation
```

FEAGI receives: **9 cortical areas** (isvi) - 1 high-res center + 8 low-res peripheral

---

## Text Agents

### Complete Text Agent Example

```rust
use feagi_agent::core::{AgentClient, AgentConfig};
use feagi_agent::sdk::sensory::text::{TextEncoder, TextEncoderConfig};
use feagi_agent::sdk::base::TopologyCache;
use tokenizers::Tokenizer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Starting Text Agent...");
    
    // 1. Configure text encoder
    let encoder_config = TextEncoderConfig {
        agent_id: "text-input-01".to_string(),
        cortical_group_id: 0,
        feagi_host: "localhost".to_string(),
        feagi_api_port: 8080,
        feagi_zmq_registration_port: 30001,
        feagi_zmq_sensory_port: 5555,
        feagi_tick_hz: 60,
        feagi_heartbeat_interval_s: 5.0,
        feagi_connection_timeout_ms: 5000,
        feagi_registration_retries: 3,
    };
    
    // 2. Create topology cache
    let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
    
    // 3. Create text encoder
    let encoder = TextEncoder::new(encoder_config.clone(), &topology_cache).await?;
    println!("✓ Text encoder ready (depth: {})", encoder.depth());
    
    // 4. Load tokenizer (GPT-2 example)
    let tokenizer = Tokenizer::from_file("tokenizers/gpt2/tokenizer.json")?;
    println!("✓ Tokenizer loaded");
    
    // 5. Create and connect agent
    let agent_config = encoder_config.to_agent_config()?;
    let mut client = AgentClient::new(agent_config)?;
    client.connect()?;
    println!("✓ Connected to FEAGI");
    
    // 6. Process text
    let text = "Hello, FEAGI!";
    let tokens = tokenizer.encode(text, false)?;
    
    // Send one token per FEAGI tick
    for token_id in tokens.get_ids() {
        let encoded = encoder.encode(token_id)?;
        client.send_sensory_bytes(encoded)?;
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }
    
    println!("✓ Sent {} tokens", tokens.get_ids().len());
    Ok(())
}
```

### Token Processing Tips

1. **One token per tick**: FEAGI processes tokens sequentially
2. **Use standard tokenizers**: GPT-2, BERT, etc.
3. **Handle special tokens**: `<PAD>`, `<UNK>`, `<EOS>`

---

## Motor Agents

### Perception Inspector Example

Motor agents receive output from FEAGI and convert it to actions:

```rust
use feagi_agent::core::{AgentClient, AgentConfig};
use feagi_agent::sdk::motor::perception::{PerceptionDecoder, PerceptionDecoderConfig};
use feagi_agent::sdk::base::TopologyCache;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("👁️ Starting Perception Inspector...");
    
    // 1. Configure perception decoder
    let decoder_config = PerceptionDecoderConfig {
        agent_id: "perception-inspector".to_string(),
        cortical_group_id: 0,
        feagi_host: "localhost".to_string(),
        feagi_api_port: 8080,
        feagi_zmq_registration_port: 30001,
        feagi_zmq_agent_sensory_port: 5555,
        feagi_zmq_motor_port: 5564,
        feagi_heartbeat_interval_s: 5.0,
        feagi_connection_timeout_ms: 5000,
        feagi_registration_retries: 3,
        feagi_motor_poll_interval_s: 0.01,
    };
    
    // 2. Create topology cache
    let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
    
    // 3. Create decoder with tokenizer
    let tokenizer_path = std::path::PathBuf::from("tokenizers/gpt2/tokenizer.json");
    let decoder = PerceptionDecoder::new(
        decoder_config.clone(),
        &topology_cache,
        Some(tokenizer_path)
    ).await?;
    println!("✓ Decoder ready");
    
    // 4. Create and connect agent
    let agent_config = decoder_config.to_agent_config()?;
    let mut client = AgentClient::new(agent_config)?;
    client.connect()?;
    println!("✓ Connected to FEAGI");
    
    // 5. Receive loop
    loop {
        if let Some(motor_data) = client.receive_motor_data()? {
            // Decode FEAGI output
            let frame = decoder.decode(&motor_data)?;
            
            // Process results
            if let Some(text) = frame.oten_text {
                println!("💭 FEAGI thought: {}", text);
            }
            
            if let Some(oimg) = frame.oimg {
                println!("🖼️ Vision output: {} voxels", oimg.x.len());
            }
        }
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
```

---

## Multi-Modal Agents

### Building a Robot Controller

Combine multiple modalities in one agent:

```rust
use feagi_agent::core::{AgentClient, AgentConfig, AgentType};
use feagi_agent::sdk::sensory::video::VideoEncoder;
use feagi_agent::sdk::sensory::text::TextEncoder;
use feagi_agent::sdk::base::TopologyCache;

struct RobotController {
    client: AgentClient,
    video_encoder: VideoEncoder,
    text_encoder: TextEncoder,
}

impl RobotController {
    pub async fn new(/* config */) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Create shared topology cache
        let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
        
        // 2. Create encoders
        let video_encoder = VideoEncoder::new(video_config, &topology_cache).await?;
        let text_encoder = TextEncoder::new(text_config, &topology_cache).await?;
        
        // 3. Build combined cortical mappings
        let mut all_cortical_mappings = video_encoder.cortical_id_mappings();
        all_cortical_mappings.extend(text_encoder.cortical_id_mappings());
        
        // 4. Create multi-modal agent
        let agent_config = AgentConfig::new("robot-01", AgentType::Sensory)
            .with_registration_endpoint("tcp://localhost:30001")
            .with_sensory_endpoint("tcp://localhost:5555")
            .with_sensory_capability(60.0, None, all_cortical_mappings);
        
        let mut client = AgentClient::new(agent_config)?;
        client.connect()?;
        
        Ok(Self {
            client,
            video_encoder,
            text_encoder,
        })
    }
    
    pub fn send_vision(&mut self, frame: ImageFrame) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = self.video_encoder.encode(&frame)?;
        self.client.send_sensory_bytes(encoded)?;
        Ok(())
    }
    
    pub fn send_text_token(&mut self, token_id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = self.text_encoder.encode(&token_id)?;
        self.client.send_sensory_bytes(encoded)?;
        Ok(())
    }
}
```

**Key Pattern**: One agent, multiple encoders, shared topology cache.

---

## Production Best Practices

### 1. Deterministic Initialization

```rust
// ❌ BAD: Network calls during real-time operation
async fn process_frame(frame: ImageFrame) {
    let encoder = VideoEncoder::new(config, &cache).await?;  // Network call!
    encoder.encode(&frame)?;
}

// ✅ GOOD: Pre-warm topology cache during startup
async fn initialize_system() -> Result<VideoEncoder> {
    let cache = TopologyCache::new("localhost", 8080, 5.0)?;
    
    // Fetch all topologies upfront (non-time-critical)
    let cortical_ids = get_all_required_ids();
    cache.prefetch(&cortical_ids).await?;
    
    // Now encoder creation is fast (no network)
    let encoder = VideoEncoder::new(config, &cache).await?;
    Ok(encoder)
}
```

### 2. Error Handling

```rust
use feagi_agent::sdk::error::{SdkError, Result};

fn handle_encoding(frame: ImageFrame) -> Result<()> {
    match encoder.encode(&frame) {
        Ok(data) => {
            client.send_sensory_bytes(data)?;
            Ok(())
        }
        Err(SdkError::EncodingFailed(msg)) => {
            // Log and continue with next frame
            eprintln!("Encoding failed: {}, skipping frame", msg);
            Ok(())
        }
        Err(SdkError::Core(core_err)) => {
            // Network issue, attempt reconnect
            Err(SdkError::Core(core_err))
        }
        Err(e) => Err(e),
    }
}
```

### 3. Resource Management

```rust
// Explicit topology cache (testable, deterministic)
let cache = TopologyCache::new("localhost", 8080, 5.0)?;

// Share cache across controllers
let video_encoder = VideoEncoder::new(video_config, &cache).await?;
let text_encoder = TextEncoder::new(text_config, &cache).await?;

// Cache statistics
println!("Cached topologies: {}", cache.cache_size());

// Clear cache if FEAGI topology changes
cache.clear_cache();
```

### 4. Performance Monitoring

```rust
use std::time::Instant;

let start = Instant::now();

// Encode
let encoded = encoder.encode(&frame)?;
let encode_time = start.elapsed();

// Send
client.send_sensory_bytes(encoded)?;
let total_time = start.elapsed();

if total_time.as_millis() > 16 {
    eprintln!("⚠️ Slow frame: {}ms (encode: {}ms)", 
        total_time.as_millis(),
        encode_time.as_millis()
    );
}
```

---

## API Reference

### Core Module

#### `AgentClient`
```rust
pub struct AgentClient { /* ... */ }

impl AgentClient {
    pub fn new(config: AgentConfig) -> Result<Self>;
    pub fn connect(&mut self) -> Result<()>;
    pub fn send_sensory_bytes(&mut self, data: Vec<u8>) -> Result<()>;
    pub fn receive_motor_data(&mut self) -> Result<Option<CorticalMappedXYZPNeuronVoxels>>;
    pub fn is_connected(&self) -> bool;
}
```

#### `AgentConfig`
```rust
pub struct AgentConfig { /* ... */ }

impl AgentConfig {
    pub fn new(agent_id: impl Into<String>, agent_type: AgentType) -> Self;
    pub fn with_registration_endpoint(self, endpoint: impl Into<String>) -> Self;
    pub fn with_sensory_endpoint(self, endpoint: impl Into<String>) -> Self;
    pub fn with_motor_endpoint(self, endpoint: impl Into<String>) -> Self;
    pub fn with_heartbeat_interval(self, seconds: f64) -> Self;
    pub fn with_connection_timeout_ms(self, ms: u64) -> Self;
    pub fn with_sensory_capability(self, rate_hz: f64, shm: Option<String>, mappings: HashMap<String, u32>) -> Self;
}
```

### SDK Module

#### `TopologyCache`
```rust
pub struct TopologyCache { /* ... */ }

impl TopologyCache {
    pub fn new(host: impl Into<String>, port: u16, timeout_s: f64) -> Result<Self>;
    pub async fn get_topology(&self, id: &CorticalID) -> Result<CorticalTopology>;
    pub async fn get_topologies(&self, ids: &[CorticalID]) -> Result<Vec<CorticalTopology>>;
    pub async fn prefetch(&self, ids: &[CorticalID]) -> Result<()>;
    pub fn clear_cache(&self);
    pub fn cache_size(&self) -> usize;
}
```

#### `VideoEncoder`
```rust
pub struct VideoEncoder { /* ... */ }

impl VideoEncoder {
    pub async fn new(config: VideoEncoderConfig, cache: &TopologyCache) -> Result<Self>;
    pub fn set_gaze(&mut self, x: f32, y: f32, modulation: f32) -> Result<()>;
    pub fn set_brightness(&mut self, brightness: i32) -> Result<()>;
    pub fn set_contrast(&mut self, contrast: f32) -> Result<()>;
}

impl SensoryEncoder for VideoEncoder {
    type Input = ImageFrame;
    fn encode(&self, input: &Self::Input) -> Result<Vec<u8>>;
    fn cortical_ids(&self) -> &[CorticalID];
}
```

#### `TextEncoder`
```rust
pub struct TextEncoder { /* ... */ }

impl TextEncoder {
    pub async fn new(config: TextEncoderConfig, cache: &TopologyCache) -> Result<Self>;
    pub fn depth(&self) -> u32;
}

impl SensoryEncoder for TextEncoder {
    type Input = u32;  // Token ID
    fn encode(&self, token_id: &Self::Input) -> Result<Vec<u8>>;
    fn cortical_ids(&self) -> &[CorticalID];
}
```

---

## Troubleshooting

### Connection Issues

**Problem:** `Failed to connect to FEAGI`

**Solutions:**
1. Check FEAGI is running: `curl http://localhost:8080/health`
2. Verify ports in config match FEAGI settings
3. Check firewall rules
4. Increase `connection_timeout_ms`

### Topology Errors

**Problem:** `Topology not found for cortical ID`

**Solutions:**
1. Ensure cortical area exists in FEAGI genome
2. Check cortical group ID matches FEAGI configuration
3. Verify HTTP API is accessible: `curl http://localhost:8080/v1/genome`

### Performance Issues

**Problem:** Agent can't keep up with desired frame rate

**Solutions:**
1. Use topology cache prefetching during initialization
2. Enable diff thresholding to reduce data
3. Lower resolution or frame rate
4. Profile with timing measurements

### Encoding Errors

**Problem:** `Encoding failed: ...`

**Solutions:**
1. Verify input dimensions match expected size
2. Check image format (RGB vs grayscale)
3. Validate token IDs are within vocabulary range
4. Enable debug logging to see detailed errors

---

## Next Steps

### Learn More

- **Examples**: See `/examples` directory for complete applications
- **API Docs**: Run `cargo doc --open`
- **FEAGI Docs**: Visit [FEAGI Documentation](https://feagi.org/docs)

### Get Help

- **Issues**: [GitHub Issues](https://github.com/feagi/feagi-core/issues)
- **Discord**: [FEAGI Community](https://discord.gg/feagi)
- **Email**: support@neuraville.com

### Contribute

We welcome contributions! See [CONTRIBUTING.md](../CONTRIBUTING.md)

---

**Built with ❤️ by Neuraville Inc.**  
**Designed for millions of developers worldwide**

