# FEAGI PNS Architecture Specification

**Date**: 2025-10-27  
**Status**: Complete Architecture Design - Ready for Implementation  
**Version**: 2.5 (Final - Hybrid + Blocking/NonBlocking + FBC Reuse + SHM Bypass + FeagiSignal Events)

> **Note**: This is the **single comprehensive architecture document** for FEAGI PNS redesign. It defines:
> - **Hybrid structure**: Reusable infrastructure (`blocking/`, `nonblocking/`) + transport implementations (`transports/`)
> - **Dual-mode support**: Transports like ZMQ can be both blocking and nonblocking
> - **Zero-copy FBC**: `Arc<FeagiByteContainer>` throughout pipeline
> - **CRITICAL: FBC reuse pattern**: Burst engine reuses FBC (amortized allocation, ~0 allocations after warm-up!)
> - **CRITICAL: Ownership transfer**: Network receive uses `try_write_data_by_ownership_to_container_and_verify()` (no copy!)
> - **CRITICAL: FeagiSignal events**: Use existing battle-tested event system instead of custom traits!
> - **SHM direct mode**: Can bypass FBC entirely for same-host maximum performance
> - **All transports**: ZMQ (blocking/nonblocking), UDP, SHM (FBC/direct), WebSocket, WebRTC, RTOS
> 
> **Performance Impact**: FBC reuse + ownership transfer + signal-based decoupling = clean, efficient architecture!
> 
> All future implementations must follow this specification.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Key Architectural Decisions](#key-architectural-decisions)
   - Single Crate with Hybrid Structure
   - Infrastructure vs Implementation Separation  
   - Infrastructure Reuse Examples
   - Zero-Copy via Arc<FBC>
   - Channel-Based Decoupling
   - Compression at Transport Layer
   - SHM Bypass for Zero-Copy
3. [Complete Data Flow Pipelines](#complete-data-flow-pipelines)
   - Outgoing: FEAGI → Agent/BV
   - Incoming: Agent → FEAGI
4. [Feature Flag Strategy](#feature-flag-strategy)
5. [Transport Implementation Matrix](#transport-implementation-matrix)
6. [Event System Design - Using FeagiSignal](#event-system-design---using-feagisignal)
7. [Migration Strategy](#migration-strategy)
8. [RTOS Transport Specification](#rtos-transport-specification)
9. [Performance Expectations](#performance-expectations)
10. [Open Questions & Decisions](#open-questions--decisions-needed)
11. [Success Criteria](#success-criteria)
12. [Conclusion](#conclusion)

---

## Executive Summary

After comprehensive architectural review and discussion, we've designed a **modern, scalable PNS architecture** that:

1. ✅ **Single crate structure** - `feagi-io` with internal modules (not micro-crates)
2. ✅ **Paradigm separation** - Explicit `blocking/` and `nonblocking/` hierarchies with reusable infrastructure
3. ✅ **Zero-copy data flow** - `Arc<FeagiByteContainer>` throughout the pipeline
4. ✅ **Event-driven decoupling** - Uses existing `FeagiSignal<T>` system (no custom traits!)
5. ✅ **FBC reuse optimization** - Burst engine reuses FBC (amortized allocation)
6. ✅ **Ownership transfer** - Network RX uses `try_write_data_by_ownership_to_container_and_verify()` (no copy!)
7. ✅ **Transport flexibility** - TCP (ZMQ), UDP, SHM, WebSocket support
8. ✅ **SHM optimization** - Can bypass FBC serialization for maximum local performance
9. ✅ **RTOS ready** - Minimal `no_std` path for embedded systems

---

## Key Architectural Decisions

### Decision 1: Single Crate with Modules (Not Multiple Crates)

**Problem**: Original design had 4-5 separate crates (`feagi-io-core`, `feagi-io-zmq`, etc.)

**Solution**: Single `feagi-io` crate with **hybrid** module structure:

```
feagi-core/crates/feagi-io/
├── Cargo.toml                    # Feature flags for transports
└── src/
    ├── lib.rs                    # Main exports
    │
    ├── core/                     # Shared across ALL transports
    │   ├── mod.rs
    │   ├── types.rs              # StreamType, SharedFBC, errors
    │   ├── agent_registry.rs     # Agent management (transport-agnostic)
    │   ├── registration.rs       # Registration handler
    │   ├── heartbeat.rs          # Heartbeat monitoring
    │   └── config.rs             # PNSConfig
    │
    ├── blocking/                 # Blocking I/O infrastructure (thread-based)
    │   ├── mod.rs
    │   ├── transport.rs          # BlockingTransport trait
    │   ├── channels.rs           # Bounded channel helpers
    │   ├── worker.rs             # Worker thread patterns
    │   └── compression.rs        # LZ4 compression utilities
    │
    ├── nonblocking/              # Non-blocking I/O infrastructure (async/await)
    │   ├── mod.rs
    │   ├── transport.rs          # NonBlockingTransport trait
    │   ├── runtime.rs            # Tokio runtime helpers
    │   ├── channels.rs           # Async channel (tokio::mpsc)
    │   └── compression.rs        # Async LZ4 compression
    │
    └── transports/               # Transport-specific implementations
        ├── zmq/                  # ZMQ (dual-mode: blocking + nonblocking)
        │   ├── mod.rs            # Common: socket creation, message format
        │   ├── blocking_impl.rs  # Implements blocking::BlockingTransport (current)
        │   └── nonblocking_impl.rs # Implements nonblocking::NonBlockingTransport (future)
        │
        ├── udp/                  # UDP (nonblocking only)
        │   └── mod.rs            # Implements nonblocking::NonBlockingTransport
        │
        ├── shm/                  # Shared Memory (blocking only)
        │   └── mod.rs            # Implements blocking::BlockingTransport
        │
        ├── websocket/            # WebSocket (nonblocking only)
        │   └── mod.rs            # Implements nonblocking::NonBlockingTransport
        │
        └── rtos/                 # RTOS/embedded (special: no_std, static)
            ├── mod.rs
            ├── transport.rs      # RTOSTransport trait (different from blocking/nonblocking)
            └── hardware.rs       # Hardware drivers (UART, SPI, etc.)
```

**Benefits of Hybrid Structure**:
- ✅ **No duplication**: Common blocking/nonblocking code in reusable modules
- ✅ **Clear organization**: Transport by name (`transports/zmq/`), paradigm by folder (`blocking/`, `nonblocking/`)
- ✅ **Code reuse**: Worker patterns, channels, compression shared across transports
- ✅ **Dual-mode support**: Transports like ZMQ can have both blocking and nonblocking implementations
- ✅ **Easy to add transports**: Implement trait + reuse infrastructure
- ✅ **Simpler maintenance**: One version, one publish
- ✅ **Standard Rust pattern**: Feature flags, not micro-crates
- ✅ **Better IDE experience**: Jump to definition works perfectly

### Decision 2: Infrastructure vs Implementation Separation

**Problem**: Mixing transport implementations with common infrastructure causes duplication

**Solution**: Hybrid model with three layers:

1. **`core/`** - Shared by everyone (types, agent registry, config)
2. **`blocking/` & `nonblocking/`** - Reusable infrastructure (traits, workers, channels, compression)
3. **`transports/`** - Specific implementations that use infrastructure

**How Transports Choose Paradigm**:
- Each transport in `transports/` imports from either `blocking::` or `nonblocking::` (or both!)
- ZMQ: Has **both** `blocking_impl.rs` (current) and `nonblocking_impl.rs` (future)
- UDP: NonBlocking only (`mod.rs` imports `nonblocking::`)
- SHM: Blocking only (`mod.rs` imports `blocking::`)
- RTOS: Neither (special `no_std` trait)

**Trait Hierarchies**:

```rust
// src/blocking/transport.rs
pub trait BlockingTransport: Send + Sync {
    fn backend_name(&self) -> &str;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    
    // Blocking operations (thread-based)
    fn publish_visualization(&self, fbc: SharedFBC) -> Result<()>;
    fn publish_motor(&self, agent_id: &str, fbc: SharedFBC) -> Result<()>;
}

// src/nonblocking/transport.rs
#[async_trait]
pub trait NonBlockingTransport: Send + Sync {
    fn backend_name(&self) -> &str;
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    
    // Async operations (tokio-based)
    async fn publish_visualization(&self, fbc: SharedFBC) -> Result<()>;
    async fn publish_motor(&self, agent_id: &str, fbc: SharedFBC) -> Result<()>;
}
```

**Usage**:
```rust
// User chooses transport (imports show paradigm)
use feagi_io::transports::zmq::ZmqTransport;           // Blocking
use feagi_io::transports::zmq::AsyncZmqTransport;      // NonBlocking (future)
use feagi_io::transports::udp::UdpTransport;           // NonBlocking
use feagi_io::transports::shm::ShmTransport;           // Blocking

// Can mix both in same binary!
struct HybridPNS {
    control: Box<dyn BlockingTransport>,    // Blocking ZMQ for reliable control
    viz: Box<dyn NonBlockingTransport>,     // NonBlocking UDP for fast visualization
}
```

**Benefits**:
- ✅ **No duplication**: Common code in `blocking/` and `nonblocking/` modules
- ✅ **Easy to extend**: New transport just imports infrastructure + implements trait
- ✅ **Dual-mode support**: Transports can have both blocking and nonblocking implementations
- ✅ **Clear imports**: Transport path shows what it is (`transports/zmq/`)
- ✅ **Explicit choice**: No hidden `#[cfg]` surprises
- ✅ **Testable**: Mock transports use same infrastructure

### Infrastructure Reuse Examples

**Reusable Blocking Infrastructure** (`blocking/` modules):

```rust
// blocking/worker.rs - Reusable worker thread pattern
pub struct WorkerThread {
    handle: JoinHandle<()>,
    tx: mpsc::Sender<Vec<u8>>,
}

impl WorkerThread {
    pub fn spawn<F>(handler: F) -> Self
    where F: FnMut(Vec<u8>) -> Result<()> + Send + 'static
    {
        let (tx, rx) = mpsc::channel(512);
        let handle = thread::spawn(move || {
            while let Ok(data) = rx.recv() {
                handler(data).ok();  // Handle errors in worker
            }
        });
        Self { handle, tx }
    }
    
    pub fn send(&self, data: Vec<u8>) -> Result<()> {
        self.tx.try_send(data).map_err(|_| "Queue full")
    }
}

// blocking/compression.rs - Reusable compression
pub fn compress_lz4(data: &[u8]) -> Result<Vec<u8>> {
    lz4::block::compress(data, Some(lz4::block::CompressionMode::FAST(1)), true)
        .map_err(|e| format!("LZ4 compression failed: {}", e))
}
```

**How ZMQ Uses Blocking Infrastructure**:

```rust
// transports/zmq/blocking_impl.rs
use crate::blocking::{BlockingTransport, worker::WorkerThread, compression};
use crate::core::{SharedFBC, StreamType};

pub struct ZmqTransport {
    context: Arc<zmq::Context>,
    worker: WorkerThread,  // ← Reuses worker pattern
}

impl BlockingTransport for ZmqTransport {
    fn publish_visualization(&self, fbc: SharedFBC) -> Result<()> {
        let compressed = compression::compress_lz4(fbc.get_byte_ref())?;  // ← Reuses
        self.worker.send(compressed)?;  // ← Reuses
        Ok(())
    }
}
```

**How SHM Uses Blocking Infrastructure (with zero-copy optimization)**:

```rust
// transports/shm/mod.rs
use crate::blocking::BlockingTransport;
use crate::core::{SharedFBC, StreamType};
use shared_memory::{ShmemConf, Shmem};

pub struct ShmTransport {
    /// Shared memory region for direct struct access (bypasses FBC!)
    shm: Shmem,
    /// Ring buffer metadata (head/tail pointers)
    ring_buffer: ShmRingBuffer,
}

impl BlockingTransport for ShmTransport {
    fn publish_visualization(&self, fbc: SharedFBC) -> Result<()> {
        // OPTIMIZATION: Bypass FBC serialization for SHM!
        // Instead of writing FBC bytes, write raw neural data directly
        
        // Get raw data from FBC (zero-copy read)
        let data = fbc.get_byte_ref();
        
        // Write directly to shared memory (zero-copy write)
        self.ring_buffer.write_direct(data)?;
        
        // NO serialization, NO deserialization, NO compression!
        Ok(())
    }
}

/// Even better: Direct shared memory access (bypasses FBC completely)
pub struct ShmDirectTransport {
    /// Direct access to shared neural activity buffer
    neural_activity_shm: Shmem,
}

impl ShmDirectTransport {
    /// Write neural activity directly to shared memory
    /// Bypasses FBC entirely - agent reads raw structs
    pub fn publish_neural_activity_direct(&self, neuron_ids: &[u32], potentials: &[f32]) -> Result<()> {
        // Write header (count)
        let header = self.neural_activity_shm.as_ptr() as *mut u32;
        unsafe { *header = neuron_ids.len() as u32; }
        
        // Write neuron IDs directly
        let ids_ptr = unsafe { header.add(1) as *mut u32 };
        unsafe {
            std::ptr::copy_nonoverlapping(
                neuron_ids.as_ptr(),
                ids_ptr,
                neuron_ids.len()
            );
        }
        
        // Write potentials directly
        let potentials_ptr = unsafe { ids_ptr.add(neuron_ids.len()) as *mut f32 };
        unsafe {
            std::ptr::copy_nonoverlapping(
                potentials.as_ptr(),
                potentials_ptr,
                potentials.len()
            );
        }
        
        Ok(())
    }
}
```

**How UDP Uses NonBlocking Infrastructure**:

```rust
// transports/udp/mod.rs
use crate::nonblocking::{NonBlockingTransport, runtime, compression};
use crate::core::{SharedFBC, StreamType};

pub struct UdpTransport {
    socket: tokio::net::UdpSocket,
    runtime: runtime::RuntimeHandle,  // ← Reuses async runtime
}

#[async_trait]
impl NonBlockingTransport for UdpTransport {
    async fn publish_visualization(&self, fbc: SharedFBC) -> Result<()> {
        let compressed = compression::compress_lz4_async(fbc.get_byte_ref()).await?;
        self.socket.send(&compressed).await?;
        Ok(())
    }
}
```

**Dual-Mode Transport (ZMQ with both blocking and nonblocking)**:

```rust
// transports/zmq/mod.rs (common code - no duplication!)
pub(crate) fn create_pub_socket(ctx: &zmq::Context, endpoint: &str) -> Result<zmq::Socket> {
    let socket = ctx.socket(zmq::PUB)?;
    socket.bind(endpoint)?;
    socket.set_sndhwm(1000)?;
    Ok(socket)
}

pub(crate) fn prepare_message(topic: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(topic.len() + 1 + payload.len());
    msg.extend_from_slice(topic);
    msg.push(b' ');
    msg.extend_from_slice(payload);
    msg
}

// transports/zmq/blocking_impl.rs
use super::{create_pub_socket, prepare_message};  // ← Reuses common ZMQ code
use crate::blocking::BlockingTransport;

pub struct ZmqTransport { /* uses blocking infrastructure */ }

// transports/zmq/nonblocking_impl.rs (future)
use super::{create_pub_socket, prepare_message};  // ← Reuses same common code
use crate::nonblocking::NonBlockingTransport;

pub struct AsyncZmqTransport { /* uses nonblocking infrastructure */ }
```

### Decision 3: Zero-Copy via Arc<FeagiByteContainer>

**Critical Requirement**: All FEAGI data must use `FeagiByteContainer` format

**Implementation**:
```rust
// src/core/types.rs
use feagi_serialization::FeagiByteContainer;
use std::sync::Arc;

/// Type alias for thread-safe shared reference to FBC
/// NOT a custom type - just Arc wrapper around existing FeagiByteContainer!
pub type SharedFBC = Arc<FeagiByteContainer>;
```

**IMPORTANT**: `SharedFBC` is just a **type alias**, not a custom type!
- ✅ Uses `FeagiByteContainer` **exactly as-is** (no modifications)
- ✅ Arc wrapper only for thread-safety (Rust requirement for sharing across threads)
- ✅ All FBC methods available: `get_byte_ref()`, `overwrite_byte_data_*()`, etc.
- ✅ Zero custom code - leverages existing FBC capabilities completely

**Why the Arc wrapper?**
- Arc is **required by Rust** to share FBC across threads (PNS worker threads, async tasks)
- Without Arc, would need to copy FBC each time (expensive!)
- Arc provides zero-copy sharing via reference counting

**What we DON'T do**:
- ❌ Don't create a new struct wrapping FBC
- ❌ Don't duplicate FBC methods
- ❌ Don't modify FBC internals
- ❌ Don't add custom fields

**What we DO**:
- ✅ Wrap existing FBC in Arc for thread-safety
- ✅ Use FBC's `get_byte_ref()` for zero-copy reads
- ✅ Use FBC's `overwrite_byte_data_*()` for reuse
- ✅ Use FBC's `try_write_data_by_ownership_*()` for ownership transfer
- ✅ Leverage all existing FBC capabilities

**CRITICAL FBC Performance Optimizations** (from `FeagiByteContainer` API):

1. **FBC is designed for reuse** (see lines 343, 400 in `feagi_byte_container.rs`):
   - `overwrite_byte_data_*()` methods preserve existing capacity
   - Only resizes if needed (`total_bytes > self.bytes.capacity()`)
   - Comments explicitly say: "Just... Don't clear the bytes. We are overwriting them or expanding if needed anyways"

2. **Ownership transfer for network receive** (line 132):
   - `try_write_data_by_ownership_to_container_and_verify(Vec<u8>)` takes ownership
   - Avoids copy when receiving decompressed data from network

3. **Reuse pattern preserves allocations** (line 151):
   - `try_write_data_by_copy_and_verify(&[u8])` does NOT free allocation
   - Uses `clear()` + `extend_from_slice()` which preserves capacity

**Optimized FBC Usage Pattern**:

```rust
// ❌ BAD: Creates new FBC every burst (allocates each time)
impl BurstEngine {
    fn send_visualization(&mut self) -> Result<()> {
        let data = self.collect_neural_activity();
        let fbc = FeagiByteContainer::new_empty();  // Allocates
        fbc.overwrite_byte_data_with_single_struct_data(&data, self.counter)?;
        let shared = Arc::new(fbc);
        self.pns.publish_visualization(shared)?;
        Ok(())
    }
}

// ✅ GOOD: Reuses FBC (no allocation if capacity sufficient)
impl BurstEngine {
    reusable_fbc: FeagiByteContainer,  // Pre-allocated, reused each burst
}

impl BurstEngine {
    fn send_visualization(&mut self) -> Result<()> {
        let data = self.collect_neural_activity();
        
        // Reuses existing allocation (no new Vec if capacity OK)
        self.reusable_fbc.overwrite_byte_data_with_single_struct_data(&data, self.counter)?;
        
        // Only Arc allocation happens
        let shared = Arc::new(self.reusable_fbc.clone());  // Clone FBC struct, not bytes
        self.pns.publish_visualization(shared)?;
        Ok(())
    }
}

// ✅ BEST: Network receive with ownership transfer (no copy!)
impl SensoryStream {
    fn process_message(&mut self, compressed: &[u8]) -> Result<()> {
        // Decompress to Vec<u8>
        let decompressed = lz4::decompress(compressed)?;  // One allocation
        
        // ❌ BAD: Copy the Vec
        // self.fbc.try_write_data_by_copy_and_verify(&decompressed)?;
        
        // ✅ GOOD: Transfer ownership (NO COPY!)
        self.fbc.try_write_data_by_ownership_to_container_and_verify(decompressed)?;
        
        let shared = Arc::new(self.fbc.clone());
        self.callback.call(shared)?;
        Ok(())
    }
}
```

**Memory Allocation Analysis**:

| Operation | Old (no reuse) | Optimized (with reuse) |
|-----------|----------------|------------------------|
| **Burst 1** | Allocate FBC Vec | Allocate FBC Vec |
| **Burst 2** | Allocate FBC Vec again | Reuse Vec (no alloc if fits) |
| **Burst 1000** | Allocate FBC Vec again | Reuse Vec (no alloc if fits) |
| **Network RX** | Decompress + Copy | Decompress + Transfer ownership |
| **Total allocs** | N bursts × 2 | 1-2 (amortized) |

**Benefits**:
- ✅ **Amortized allocation**: After first few bursts, no more Vec allocations
- ✅ **Ownership transfer**: Network receive avoids copy
- ✅ **Cache-friendly**: Same memory region reused

### Decision 4: Channel-Based Decoupling

**Problem**: Sync burst engine vs async transports (impedance mismatch)

**Solution**: Channels as boundary layer:

```rust
pub struct PNS {
    // Channels hold Arc<FBC>, not Vec<u8>!
    viz_tx: mpsc::Sender<SharedFBC>,           // Burst engine writes here
    viz_rx: mpsc::Receiver<SharedFBC>,         // Transport reads here
    
    motor_tx: mpsc::Sender<(String, SharedFBC)>,  // (agent_id, fbc)
    motor_rx: mpsc::Receiver<(String, SharedFBC)>,
    
    transport: TransportHandle,                 // Enum of blocking/nonblocking
}

enum TransportHandle {
    Blocking(Arc<dyn BlockingTransport>),
    NonBlocking(Arc<dyn NonBlockingTransport>),
}
```

**Benefits**:
- ✅ **Decouples** burst engine from transport
- ✅ **Backpressure** via bounded channels (no unbounded growth)
- ✅ **Works for both** blocking and nonblocking transports
- ✅ **Zero data copy** (Arc clone is cheap)

### Decision 5: Compression at Transport Layer

**Decision**: Compression happens in transport, NOT in FBC

**Rationale**:
- FBC is transport-agnostic data format
- Some transports may not need compression (SHM, localhost)
- UDP may use different compression than TCP
- Allows transport-specific optimization

### Decision 6: SHM Bypass for Zero-Copy

**Problem**: For same-host communication, FBC serialization/deserialization is wasteful overhead

**Solution**: SHM transport offers two modes:

**Mode 1: FBC-Compatible** (default, for compatibility):
```rust
// Uses FBC format (works with any agent)
transport.publish_visualization(fbc)?;  // Writes FBC bytes to SHM
```

**Mode 2: Direct Access** (maximum performance):
```rust
// Bypasses FBC entirely - writes raw structs to SHM
transport.publish_neural_activity_direct(neuron_ids, potentials)?;
```

**Comparison**:

| Aspect | Network (ZMQ/UDP) | SHM with FBC | SHM Direct |
|--------|------------------|--------------|------------|
| **Serialization** | Required (network) | Wasteful (same process) | ❌ None |
| **Deserialization** | Required | Wasteful | ❌ None |
| **Memory Copies** | Multiple | 1 (to SHM) | ❌ 0 (direct access) |
| **Validation** | FBC validation | FBC validation | Manual/trust |
| **Compatibility** | Universal | Universal | SHM-only |
| **Performance** | Slowest | Fast | **Fastest** |

**When to use each**:
- **FBC mode**: Agent on different machine (but same host via bridge), need validation
- **Direct mode**: Agent on same machine, maximum performance, trusted code

**Implementation Strategy**:
```rust
// transports/shm/mod.rs
pub enum ShmMode {
    FbcCompatible,  // Uses FBC format
    DirectAccess,   // Bypasses FBC
}

impl ShmTransport {
    pub fn new(mode: ShmMode) -> Self {
        match mode {
            ShmMode::FbcCompatible => {
                // Standard BlockingTransport trait
                // Writes Arc<FBC> bytes to SHM
            }
            ShmMode::DirectAccess => {
                // Custom trait with direct neural data API
                // Writes raw structs (neuron_ids, potentials) to SHM
            }
        }
    }
}
```

**Benefits**:
- ✅ **Zero-copy**: Agent reads directly from shared memory
- ✅ **Zero serialization**: No FBC encoding/decoding
- ✅ **Maximum throughput**: No CPU waste on ser/deser
- ✅ **Flexible**: Can choose FBC for compatibility or direct for speed

---

## Complete Data Flow Pipelines

### Pipeline 1: Outgoing Data (FEAGI → Agent/BV)

**Use Case**: Visualization data from burst engine to Brain Visualizer

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: BURST ENGINE - Collect Neural Activity                  │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Rust native data structures (HashMap, Vec, etc.)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: SERIALIZE TO FBC - feagi-data-processing (OPTIMIZED!)   │
│                                                                  │
│   // Reuse pre-allocated FBC (amortized allocation)             │
│   self.reusable_fbc.overwrite_byte_data_with_single_struct_data(│
│       &visualization_data,                                       │
│       increment_counter                                          │
│   )?;                                                            │
│                                                                  │
│   ✅ ZERO ALLOCATION if capacity sufficient                     │
│   ✅ FBC designed for reuse (preserves capacity)                │
│   ⚡ After first few bursts: NO MORE ALLOCATIONS!               │
└─────────────────────────────────────────────────────────────────┘
    │
    │ FBC: [header|struct_lookup|serialized_data]
    │      [4 bytes|4 bytes|N bytes]
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: WRAP IN ARC - Zero-copy starts here                     │
│                                                                  │
│   let shared_fbc = Arc::new(fbc);                               │
│                                                                  │
│   ✅ ZERO-COPY FROM HERE ON                                     │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Arc<FBC> (refcount = 1)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 4: BURST ENGINE → PNS EVENT SINK                           │
│                                                                  │
│   self.event_sink.on_visualization_ready(shared_fbc)?;          │
│                                                                  │
│   ✅ Arc clone (cheap, no data copy)                            │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Arc<FBC> (refcount = 2)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 5: PNS → CHANNEL SEND                                      │
│                                                                  │
│   self.viz_tx.try_send(fbc)?;                                   │
│                                                                  │
│   ✅ Bounded channel (backpressure if full)                     │
│   ✅ Arc clone (refcount++)                                     │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Channel: [Arc<FBC>, Arc<FBC>, ...] (max 512 slots)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 6: TRANSPORT WORKER THREAD - Receives from channel         │
│                                                                  │
│   while let Ok(fbc) = viz_rx.recv() {                           │
│       transport.publish_visualization(fbc)?;                     │
│   }                                                              │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Arc<FBC> (refcount--)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 7: TRANSPORT - Compress & Send                             │
│                                                                  │
│   // ZMQ (sync)                                                 │
│   let bytes = fbc.get_byte_ref();  // ← Zero-copy read         │
│   let compressed = lz4::compress(bytes)?;  // ← Only copy      │
│   socket.send(&compressed, 0)?;                                 │
│                                                                  │
│   // UDP (async)                                                │
│   let bytes = fbc.get_byte_ref();  // ← Zero-copy read         │
│   let compressed = lz4::compress(bytes)?;  // ← Only copy      │
│   self.socket.send(&compressed).await?;                         │
│                                                                  │
│   ✅ Compression is ONLY copy that happens                      │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Compressed bytes over network
    │ TCP: tcp://192.168.1.100:5562
    │ UDP: udp://192.168.1.100:5562
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 8: BRAIN VISUALIZER - Receives via feagi_bridge            │
│                                                                  │
│   1. Bridge receives compressed bytes                           │
│   2. Decompress (LZ4)                                           │
│   3. Deserialize FBC                                            │
│   4. Extract visualization data                                 │
│   5. Forward to Godot via WebSocket                            │
└─────────────────────────────────────────────────────────────────┘
```

**Memory Efficiency** (with FBC reuse optimization):
- ✅ **~0 allocations**: FBC reused (amortized), only Arc allocated
- ✅ **1 copy**: Compression in transport layer (unavoidable)
- ✅ **N Arc clones**: Cheap refcount increments (8 bytes)
- ⚡ **First burst**: Allocates FBC Vec (~4KB typical)
- ⚡ **Subsequent bursts**: Reuses Vec if data fits (zero allocation!)
- ❌ **No Vec copies**: Arc prevents data duplication

### Pipeline 2: Incoming Data (Agent → FEAGI)

**Use Case**: Sensory data from agent to burst engine

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: AGENT - Collect Sensory Data                            │
│                                                                  │
│   Python/C++/Rust agent using feagi-agent                   │
│   Examples: camera pixels, robot sensors, game state            │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Native data (numpy array, cv::Mat, Vec<u8>, etc.)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: SERIALIZE TO FBC - Agent-side feagi-data-processing     │
│                                                                  │
│   # Python agent                                                │
│   fbc = FeagiByteContainer()                                    │
│   fbc.add_xyzp_data(cortical_id, x_coords, y_coords, potentials)│
│                                                                  │
│   // Rust agent                                                 │
│   let mut fbc = FeagiByteContainer::new_empty();                │
│   fbc.overwrite_byte_data_with_single_struct_data(&xyzp)?;     │
│                                                                  │
│   ✅ Agent uses SAME FBC format                                 │
└─────────────────────────────────────────────────────────────────┘
    │
    │ FBC bytes: [header|struct_lookup|XYZP_data]
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: COMPRESS (optional, agent-side)                         │
│                                                                  │
│   compressed = lz4.compress(fbc.get_bytes())                    │
│                                                                  │
│   ✅ Reduces network bandwidth                                  │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Compressed FBC bytes over network
    │ TCP: zmq.PUSH to tcp://feagi:5555
    │ UDP: socket.sendto(bytes, ('feagi', 5555))
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 4: PNS SENSORY STREAM - Network receive                    │
│                                                                  │
│   // ZMQ (sync)                                                 │
│   let mut msg = zmq::Message::new();                            │
│   socket.recv(&mut msg, 0)?;                                    │
│   let compressed_bytes = msg.as_ref();                          │
│                                                                  │
│   // UDP (async)                                                │
│   let mut buf = [0u8; 65535];                                   │
│   let (len, _addr) = socket.recv_from(&mut buf).await?;         │
│   let compressed_bytes = &buf[..len];                           │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Raw bytes (compressed)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 5: DECOMPRESS                                              │
│                                                                  │
│   let decompressed = lz4::decompress(compressed_bytes)?;        │
│                                                                  │
│   ✅ Now have raw FBC bytes                                     │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Decompressed FBC bytes
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 6: DESERIALIZE TO FBC (OPTIMIZED!)                         │
│                                                                  │
│   // Reuse pre-allocated FBC                                    │
│   // Transfer ownership of Vec<u8> (NO COPY!)                   │
│   self.reusable_fbc.try_write_data_by_ownership_to_container_and_verify(│
│       decompressed  // Vec<u8> moved, not copied                │
│   )?;                                                            │
│                                                                  │
│   ✅ Validates FBC format                                       │
│   ✅ Parses headers and struct references                       │
│   ⚡ NO COPY - ownership transferred from decompression         │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Valid FBC structure
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 7: WRAP IN ARC - Zero-copy starts                          │
│                                                                  │
│   let shared_fbc = Arc::new(fbc);                               │
│                                                                  │
│   ✅ ZERO-COPY FROM HERE ON                                     │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Arc<FBC> (refcount = 1)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 8: PNS → BURST ENGINE 
                             
                             │
│                                                                  │
│   self.sensory_callback.call(shared_fbc)?;                      │
│                                                                  │
│   ✅ Arc clone (cheap)                                          │
└─────────────────────────────────────────────────────────────────┘
    │
    │ Arc<FBC> (refcount = 2)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 9: BURST ENGINE - Extract XYZP Data from FBC               │
│                                                                  │
│   let xyzp = fbc.try_create_struct_from_first_found_struct_of_type(│
│       FeagiByteStructureType::CorticalMappedXYZPNeuronVoxels    │
│   )?;                                                            │
│                                                                  │
│   ✅ Deserializes structured data from FBC                      │
└─────────────────────────────────────────────────────────────────┘
    │
    │ CorticalMappedXYZPNeuronVoxels struct
    │ HashMap<CorticalID, XYZPNeuronArray>
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 10: BURST ENGINE - Inject into NPU                         │
│                                                                  │
│   for (cortical_id, neuron_array) in xyzp.mappings {            │
│       let cortical_idx = npu.get_cortical_area_id(cortical_id)?;│
│       let (x, y, z, p) = neuron_array.borrow_xyzp_vectors();   │
│                                                                  │
│       for i in 0..neuron_array.len() {                          │
│           let neuron_id = npu.get_neuron_at_coordinates(        │
│               cortical_idx, x[i], y[i], z[i]                    │
│           )?;                                                    │
│           neuron_potential_pairs.push((neuron_id, p[i]));       │
│       }                                                          │
│                                                                  │
│       npu.inject_sensory_with_potentials(&neuron_potential_pairs);│
│   }                                                              │
│                                                                  │
│   ✅ Sensory data now in NPU, ready for next burst              │
└─────────────────────────────────────────────────────────────────┘
```

**Memory Efficiency** (with ownership transfer optimization):
- ✅ **Network → Decompress**: 1 allocation (unavoidable)
- ✅ **Decompress → FBC**: Ownership transfer (NO COPY!)
- ✅ **FBC → Burst Engine**: Arc clone (no data copy)
- ✅ **FBC → XYZP**: Deserialization (necessary for NPU injection)
- ⚡ **FBC reuse**: Sensory stream reuses FBC across receives (amortized allocation)
- ❌ **No intermediate buffers**: Direct ownership transfer from decompression to FBC

---

## Feature Flag Strategy

### Cargo.toml
```toml
[features]
default = ["std", "transport-zmq"]

# Core features
std = []
async = ["dep:async-trait", "dep:tokio"]

# Transport implementations
transport-zmq = ["std", "dep:zmq", "dep:lz4"]              # Sync ZMQ (blocking)
transport-zmq-async = ["async", "std", "dep:async-zmq"]    # Async ZMQ (future)
transport-udp = ["async", "std"]                            # Async UDP
transport-shm = ["std", "dep:shared_memory"]                # Sync SHM
transport-websocket = ["async", "std", "dep:tokio-tungstenite"]  # Async WebSocket
transport-rtos = []                                         # RTOS (no_std, special)

# Convenience groups
all-transports = [
    "transport-zmq",
    "transport-udp",
    "transport-shm",
    "transport-websocket"
]

# Common infrastructure (automatically enabled by transports)
sync-infra = ["std"]   # Enabled by any sync transport
async-infra = ["async", "std"]  # Enabled by any async transport

[dependencies]
# Core (always)
feagi-serialization = { workspace = true }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
parking_lot = "0.12"
ahash = "0.8"

# Sync infrastructure
crossbeam-queue = { version = "0.3", optional = false }  # Used by sync transports

# Async infrastructure  
async-trait = { version = "0.1", optional = true }
tokio = { version = "1.0", features = ["net", "sync", "rt"], optional = true }

# Transport-specific
zmq = { version = "0.10", optional = true }
async-zmq = { version = "0.4", optional = true }
lz4 = { version = "1.24", optional = true }
shared_memory = { version = "0.12", optional = true }
tokio-tungstenite = { version = "0.20", optional = true }

# RTOS (no_std)
[target.'cfg(not(feature = "std"))'.dependencies]
# Minimal dependencies for embedded
```

### Build Examples
```bash
# Default: Sync ZMQ only (current)
cargo build

# Async UDP (for high-throughput viz)
cargo build --features "transport-udp"

# Hybrid: Sync ZMQ + Async UDP
cargo build --features "transport-zmq,transport-udp"

# All transports (development)
cargo build --features "all-transports"

# Dual-mode ZMQ (both sync and async)
cargo build --features "transport-zmq,transport-zmq-async"

# RTOS (minimal, embedded)
cargo build --no-default-features --features "transport-rtos"
```

---

## Transport Implementation Matrix

| Transport | Module Path | Paradigm | Protocol | Feature Flag | Status | Notes |
|-----------|-------------|----------|----------|--------------|--------|-------|
| **ZMQ (Sync)** | `transports/zmq/sync_impl.rs` | Sync | TCP | `transport-zmq` | ✅ Existing | Blocking I/O, worker threads |
| **ZMQ (Async)** | `transports/zmq/async_impl.rs` | Async | TCP | `transport-zmq-async` | 🔜 Phase 6 | Future async-zmq wrapper |
| **UDP** | `transports/udp/mod.rs` | Async | UDP | `transport-udp` | 🔨 Phase 4 | Lossy, real-time, tokio::net |
| **SHM** | `transports/shm/mod.rs` | Sync | IPC | `transport-shm` | 🔜 Phase 5 | **Can bypass FBC!** Direct struct access |
| **WebSocket** | `transports/websocket/mod.rs` | Async | TCP/WS | `transport-websocket` | 🔜 Phase 6 | Browser agents, tokio-tungstenite |
| **WebRTC** | `transports/webrtc/mod.rs` | Async | UDP | `transport-webrtc` | 🔜 Future | Data channels, NAT traversal |
| **RTOS** | `transports/rtos/` | Special | Hardware | `transport-rtos` | 🔜 Phase 7 | `no_std`, static, UART/SPI |

### Transport Capabilities

**Dual-Mode Transports** (support both sync and async):
- ✅ ZMQ: `transport-zmq` (sync) + `transport-zmq-async` (async)
- 🔜 NATS: Future (can support both)
- 🔜 HTTP: Future (reqwest::blocking vs reqwest)

**Single-Mode Transports**:
- **Blocking only**: SHM (inherently blocking), RTOS (special)
- **NonBlocking only**: UDP (best with tokio), WebSocket (async-first), WebRTC (async-only)

**Infrastructure Usage**:
| Transport | Uses `blocking/` | Uses `nonblocking/` | Uses `core/` | Special |
|-----------|-----------------|---------------------|--------------|---------|
| ZMQ (Blocking) | ✅ Worker, compression | ❌ | ✅ SharedFBC | - |
| ZMQ (NonBlocking) | ❌ | ✅ Runtime, compression | ✅ SharedFBC | - |
| UDP | ❌ | ✅ Runtime, compression | ✅ SharedFBC | - |
| SHM (FBC mode) | ✅ Channels | ❌ | ✅ SharedFBC | No compression |
| SHM (Direct mode) | ✅ Channels | ❌ | ⚠️ **Bypasses FBC!** | Direct struct access |
| WebSocket | ❌ | ✅ Runtime | ✅ SharedFBC | - |
| RTOS | ❌ | ❌ | ⚠️ StaticFBC only | No Arc, no heap |

---

## Event System Design - Using FeagiSignal

**CRITICAL**: FEAGI already has a production-ready event system: `FeagiSignal<T>` from `feagi-data-structures`!

### Why FeagiSignal is Perfect

✅ **Thread-safe**: `Send` + `Sync` bounds  
✅ **Multiple subscribers**: One signal, many listeners  
✅ **Type-safe**: Generic `FeagiSignal<T>`  
✅ **Arc<Mutex<>> helper**: Built-in `connect_with_shared_state()`  
✅ **Disconnect support**: Can unsubscribe with handle  
✅ **Battle-tested**: Already used in FEAGI data processing  

### Event Types

```rust
// src/core/events.rs
use feagi-data-structures::FeagiSignal;
use crate::core::{SharedFBC, AgentInfo};

/// Event: Visualization data ready to send
pub type VisualizationReadyEvent = SharedFBC;

/// Event: Motor commands ready for specific agent
#[derive(Clone)]
pub struct MotorCommandEvent {
    pub agent_id: String,
    pub fbc: SharedFBC,
}

/// Event: Sensory data received from agent
#[derive(Clone)]
pub struct SensoryDataEvent {
    pub agent_id: String,
    pub fbc: SharedFBC,
}

/// Event: Agent registered/connected
#[derive(Clone)]
pub struct AgentRegisteredEvent {
    pub info: AgentInfo,
}

/// Event: Agent disconnected
#[derive(Clone)]
pub struct AgentDisconnectedEvent {
    pub agent_id: String,
}
```

### Burst Engine with Signals

```rust
// Burst engine emits events via signals
pub struct BurstEngine {
    npu: NPU,
    reusable_fbc: FeagiByteContainer,
    
    // Outgoing signals (PNS subscribes)
    pub visualization_ready: Arc<Mutex<FeagiSignal<VisualizationReadyEvent>>>,
    pub motor_commands: Arc<Mutex<FeagiSignal<MotorCommandEvent>>>,
}

impl BurstEngine {
    fn send_visualization(&mut self) -> Result<()> {
        // Serialize to FBC (reused allocation)
        let data = self.collect_neural_activity();
        self.reusable_fbc.overwrite_byte_data_with_single_struct_data(&data, self.counter)?;
        
        // Wrap in Arc
        let shared_fbc = Arc::new(self.reusable_fbc.clone());
        
        // Emit signal (PNS receives it)
        self.visualization_ready.lock().unwrap().emit(&shared_fbc);
        
        Ok(())
    }
    
    fn send_motor_commands(&mut self, agent_id: &str) -> Result<()> {
        let commands = self.generate_motor_commands(agent_id)?;
        self.reusable_fbc.overwrite_byte_data_with_single_struct_data(&commands, self.counter)?;
        let shared_fbc = Arc::new(self.reusable_fbc.clone());
        
        let event = MotorCommandEvent {
            agent_id: agent_id.to_string(),
            fbc: shared_fbc,
        };
        
        self.motor_commands.lock().unwrap().emit(&event);
        Ok(())
    }
}
```

### PNS Subscribes to Burst Engine

```rust
pub struct PNS {
    transport: TransportHandle,
    burst_engine_signals: BurstEngineSignals,
}

pub struct BurstEngineSignals {
    visualization_subscription: FeagiSignalIndex,
    motor_subscription: FeagiSignalIndex,
}

impl PNS {
    pub fn connect_to_burst_engine(&mut self, engine: &mut BurstEngine) -> Result<()> {
        // Subscribe to visualization events
        let transport = Arc::clone(&self.transport);
        let viz_sub = engine.visualization_ready.lock().unwrap().connect(
            move |fbc: &SharedFBC| {
                let _ = transport.publish_visualization(Arc::clone(fbc));
            }
        );
        
        // Subscribe to motor events
        let transport = Arc::clone(&self.transport);
        let motor_sub = engine.motor_commands.lock().unwrap().connect(
            move |event: &MotorCommandEvent| {
                let _ = transport.publish_motor(&event.agent_id, Arc::clone(&event.fbc));
            }
        );
        
        self.burst_engine_signals = BurstEngineSignals {
            visualization_subscription: viz_sub,
            motor_subscription: motor_sub,
        };
        
        Ok(())
    }
}
```

### PNS Emits Events for Burst Engine

```rust
pub struct PNS {
    // Signals PNS emits (burst engine subscribes)
    pub sensory_data_received: Arc<Mutex<FeagiSignal<SensoryDataEvent>>>,
    pub agent_registered: Arc<Mutex<FeagiSignal<AgentRegisteredEvent>>>,
    pub agent_disconnected: Arc<Mutex<FeagiSignal<AgentDisconnectedEvent>>>,
}

impl PNS {
    fn handle_sensory_message(&mut self, agent_id: &str, compressed: &[u8]) -> Result<()> {
        // Decompress
        let decompressed = lz4::decompress(compressed)?;
        
        // Ownership transfer to FBC (no copy!)
        self.reusable_fbc.try_write_data_by_ownership_to_container_and_verify(decompressed)?;
        
        // Emit event
        let event = SensoryDataEvent {
            agent_id: agent_id.to_string(),
            fbc: Arc::new(self.reusable_fbc.clone()),
        };
        
        self.sensory_data_received.lock().unwrap().emit(&event);
        Ok(())
    }
}
```

### Burst Engine Subscribes to PNS

```rust
impl BurstEngine {
    pub fn connect_to_pns(&mut self, pns: &mut PNS) -> Result<()> {
        // Subscribe to sensory data
        let npu = Arc::clone(&self.npu);
        pns.sensory_data_received.lock().unwrap().connect_with_shared_state(
            npu,
            |npu, event: &SensoryDataEvent| {
                // Extract XYZP from FBC
                if let Ok(Some(xyzp)) = event.fbc.try_create_struct_from_first_found_struct_of_type(
                    FeagiByteStructureType::CorticalMappedXYZPNeuronVoxels
                ) {
                    npu.inject_sensory(xyzp).ok();
                }
            }
        );
        
        // Subscribe to agent events
        pns.agent_registered.lock().unwrap().connect(
            move |event: &AgentRegisteredEvent| {
                println!("Agent registered: {:?}", event.info);
            }
        );
        
        Ok(())
    }
}
```

### Benefits of FeagiSignal-Based Design

**vs. Custom Trait Approach**:

| Aspect | Custom Traits | FeagiSignal |
|--------|--------------|-------------|
| **Code to write** | New traits + impls | Reuse existing |
| **Multiple subscribers** | Need Vec of handlers | Built-in |
| **Unsubscribe** | Manual management | `disconnect(handle)` |
| **Shared state** | Manual Arc<Mutex<>> | `connect_with_shared_state()` helper |
| **Testing** | Mock trait impls | Signals + assertions |
| **Thread-safety** | Manual Send + Sync | Already guaranteed |
| **Battle-tested** | New code, new bugs | Production-proven |

**vs. Channel-Based Approach**:

| Aspect | Channels | FeagiSignal |
|--------|----------|-------------|
| **Fan-out** | Need broadcast channel | One signal, many listeners |
| **Backpressure** | Bounded queues | Immediate invocation (no queue) |
| **Latency** | Queue latency | Direct function call |
| **Memory** | Queue allocations | Zero allocations (just Arc clones) |
| **Ordering** | FIFO guaranteed | Invocation order (deterministic) |
| **Setup** | Channel + worker threads | Just connect() |

**Real-World Benefits**:

✅ **Zero custom code**: No need to define `BlockingBurstEngineEvents`, `NonBlockingBurstEngineEvents`, `PNSEvents` traits  
✅ **Multiple outputs**: Burst engine can emit to PNS + file logger + metrics + debugger (4+ subscribers)  
✅ **Testable**: Test can subscribe to signals and assert events fired  
✅ **Clean lifecycle**: `disconnect()` when component shuts down  
✅ **Type-safe**: Compiler checks event types  
✅ **Godot-like**: Familiar pattern for game engine developers  

**Example: Multiple Subscribers**:

```rust
// Burst engine emits visualization once
engine.visualization_ready.lock().unwrap().emit(&fbc);

// Multiple handlers receive it:
// 1. PNS sends over network
// 2. File logger saves to disk
// 3. Metrics tracks size/rate
// 4. Debugger displays in UI
// All happen automatically via signals!
```

---

## Migration Strategy

### Phase 1: Restructure (1 week)
- ✅ Move current code into `src/blocking/zmq/`
- ✅ Extract shared types to `src/core/`
- ✅ Create trait definitions
- ✅ Update imports

**Risk**: Low (code movement, no logic changes)

### Phase 2: FeagiSignal Integration (1 week)
- 🔨 Replace direct calls with FeagiSignal events
- 🔨 Connect PNS to burst engine signals
- 🔨 Add bounded queues with backpressure
- 🔨 Update burst engine to emit signals

**Risk**: Medium (runtime behavior changes)

### Phase 3: UDP Implementation (1 week)
- 🔜 Create `src/nonblocking/udp/`
- 🔜 Implement `NonBlockingTransport` trait
- 🔜 Add UDP-specific config
- 🔜 Integrate with signal system

**Risk**: Medium (new transport type)

### Phase 4: Testing & Optimization (1 week)
- 🔜 Benchmark ZMQ vs UDP throughput
- 🔜 Test 200K neuron activations
- 🔜 Measure zero-copy effectiveness
- 🔜 Profile memory usage

**Risk**: Low (non-breaking)

**Total MVP Timeline**: **4 weeks**

---

## Backward Compatibility

### Python API
```python
# Old API (still works, uses ZMQ)
pns = feagi_rust.create_pns(
    zmq_rest="tcp://0.0.0.0:30001",
    zmq_motor="tcp://0.0.0.0:5564",
    zmq_viz="tcp://0.0.0.0:5562",
    zmq_sensory="tcp://0.0.0.0:5555"
)

# New API (opt-in, supports UDP)
config = PNSConfig.udp_optimized()
pns = feagi_rust.create_pns_with_config(config)
```

### Rust API
```rust
// Old code continues to work
let config = PNSConfig::default();  // Uses sync-zmq backend
let pns = PNS::with_config(config)?;

// New code gains flexibility
let config = PNSConfig::udp_optimized();  // TCP control + UDP viz
let pns = PNS::with_config(config)?;
```

---

## Performance Expectations

### Current (ZMQ/TCP)
- ✅ Works well up to ~8K neurons
- ❌ Drops frames beyond 10K neurons
- ❌ Cannot handle 200K neuron bursts

### Target (UDP)
- 🎯 Handle 200K neuron activations per burst
- 🎯 < 10ms latency for visualization data
- 🎯 Graceful degradation (drop frames, not crash)
- 🎯 Zero-copy through pipeline (except compression)

### Benchmarks to Track
- Burst engine → PNS latency
- PNS queue saturation rate
- Network throughput (MB/s)
- Memory usage per frame
- Arc refcount overhead

---

## RTOS Transport Specification

### Overview

The RTOS transport provides a minimal, `no_std` communication layer for embedded systems. Designed for resource-constrained environments with deterministic timing requirements.

### Constraints

**Memory**:
- ✅ Static buffers only (no `Vec`, no `Box`, no `Arc`)
- ✅ Fixed-size `FeagiByteContainer` preallocated at compile time
- ✅ Zero allocation after initialization
- ❌ No heap, no dynamic dispatch

**Concurrency**:
- ✅ Interrupt-safe primitives
- ✅ Lock-free where possible
- ✅ Static task assignment (RTOS tasks, not threads)
- ❌ No async/await (no executor)

**I/O**:
- ✅ Direct hardware register access (UART, SPI, I2C)
- ✅ DMA transfers for large payloads
- ✅ Blocking send/receive with timeouts
- ❌ No network stack (unless provided by RTOS)

### RTOS-Specific Trait

```rust
// src/transports/rtos/transport.rs
pub trait RTOSTransport: Send {
    /// Send blocking with timeout (in RTOS ticks)
    fn send_blocking(
        &mut self,
        stream: StreamType,
        fbc: &StaticFBC,
        timeout_ticks: u32
    ) -> Result<usize, RTOSError>;
    
    /// Receive blocking with timeout
    fn recv_blocking(
        &mut self,
        stream: StreamType,
        fbc: &mut StaticFBC,
        timeout_ticks: u32
    ) -> Result<usize, RTOSError>;
    
    /// Poll without blocking (for interrupt handlers)
    fn poll(&mut self, stream: StreamType) -> Result<Option<&StaticFBC>, RTOSError>;
}

/// Static FBC (no heap allocation)
pub struct StaticFBC {
    storage: &'static mut [u8; MAX_FBC_SIZE],
    container: FeagiByteContainer,
}

/// Compile-time configuration
pub const MAX_FBC_SIZE: usize = 4096;
pub const FBC_POOL_SIZE: usize = 4;
pub const UART_BAUD_RATE: u32 = 115200;
pub const DEFAULT_TIMEOUT_TICKS: u32 = 10;
```

### Static Dispatch Pattern

```rust
// ❌ NOT possible on RTOS (requires heap)
pub struct PNS {
    transport: Box<dyn BlockingTransport>,
}

// ✅ Use static dispatch instead
pub struct PNS<T: RTOSTransport> {
    transport: T,  // Monomorphized at compile time
}
```

### Example RTOS Usage

```rust
use feagi_io::transports::rtos::{RTOSTransport, StaticFBC, MAX_FBC_SIZE};

// Static buffers (no heap!)
static mut FBC_STORAGE_0: [u8; MAX_FBC_SIZE] = [0; MAX_FBC_SIZE];
static mut FBC_STORAGE_1: [u8; MAX_FBC_SIZE] = [0; MAX_FBC_SIZE];

#[entry]
fn main() -> ! {
    let mut fbc_send = unsafe { StaticFBC::new(&mut FBC_STORAGE_0) };
    let mut fbc_recv = unsafe { StaticFBC::new(&mut FBC_STORAGE_1) };
    
    let mut transport = RTOSTransport::new(UartDriver::new(UART1));
    
    loop {
        // Poll for incoming data
        if let Ok(Some(fbc)) = transport.poll(StreamType::Sensory) {
            process_sensory(fbc);
        }
        
        // Send visualization (if ready)
        if should_send_viz() {
            prepare_viz_fbc(&mut fbc_send);
            transport.send_blocking(
                StreamType::Visualization,
                &fbc_send,
                DEFAULT_TIMEOUT_TICKS
            )?;
        }
        
        rtos::delay_ticks(1);
    }
}
```

### Target Platforms

**Tier 1** (High Priority):
- FreeRTOS - Most widely used
- Zephyr - Modern, well-supported
- Embassy - Rust-native async RTOS

**Tier 2** (Medium Priority):
- ThreadX (Azure RTOS)
- VxWorks (Industrial/aerospace)
- QNX (Automotive)

**Tier 3** (Low Priority):
- RIOT (IoT-focused)
- Mbed OS (ARM Cortex-M)
- Bare-metal (no RTOS)

### Hardware Abstraction

```rust
// src/sync/rtos/hardware.rs
pub trait RTOSDriver: Send {
    fn init(&mut self) -> Result<(), RTOSError>;
    fn send_bytes(&mut self, data: &[u8], timeout: u32) -> Result<usize, RTOSError>;
    fn recv_bytes(&mut self, buf: &mut [u8], timeout: u32) -> Result<usize, RTOSError>;
    fn data_available(&self) -> bool;
}

// Example UART implementation
pub struct UartDriver {
    uart: stm32f4xx_hal::serial::Serial<UART1>,
    rx_buffer: [u8; 256],
}

impl RTOSDriver for UartDriver {
    fn send_bytes(&mut self, data: &[u8], _timeout: u32) -> Result<usize, RTOSError> {
        for &byte in data {
            nb::block!(self.uart.write(byte))?;
        }
        Ok(data.len())
    }
    // ... other methods
}
```

### Performance Requirements

**Memory Footprint**:
- Target: < 64 KB Flash, < 16 KB RAM
- Stretch: < 32 KB Flash, < 8 KB RAM

**Latency**:
- Sensory → NPU: < 1 ms
- NPU → Visualization: < 1 ms
- Total round-trip: < 5 ms

**Determinism**:
- 99.9% of bursts complete within deadline
- No unbounded loops
- No dynamic allocation after init

### Build for RTOS

```bash
# Bare-metal ARM Cortex-M4
cargo build \
    --target thumbv7em-none-eabihf \
    --no-default-features \
    --features "rtos,rtos-freertos"

# Check binary size
arm-none-eabi-size target/thumbv7em-none-eabihf/release/feagi-io
```

### Feature Flags

```toml
[features]
# RTOS-specific
rtos = ["sync-rtos"]
sync-rtos = ["sync"]

# RTOS platforms
rtos-freertos = ["rtos", "dep:freertos-rust"]
rtos-zephyr = ["rtos", "dep:zephyr-sys"]
rtos-embassy = ["rtos", "dep:embassy-executor"]
```

### Implementation Timeline (Phase 7)

**Phase 7a: Design** (2 weeks)
- Finalize RTOSTransport trait API
- Create StaticFBC implementation
- Document hardware requirements

**Phase 7b: HAL** (2 weeks)
- Implement UART driver
- Implement SPI driver
- Create test harness

**Phase 7c: Integration** (2 weeks)
- Port burst engine to RTOS
- Add FreeRTOS example
- Performance tuning

**Phase 7d: Validation** (2 weeks)
- Hardware testing on dev boards
- Timing analysis with logic analyzer
- Documentation

**Total Phase 7**: 8 weeks (post-MVP)

---

## Open Questions & Decisions Needed

### 1. ✅ Compression Strategy - RESOLVED
**Decision**: Transport layer compresses, not FBC  
**Rationale**: Transport-specific optimization, keeps FBC generic

### 2. ✅ FBC Pooling - RESOLVED
**Decision**: No pooling (to avoid data corruption)  
**Rationale**: Safety over performance, allocation cost acceptable

### 3. ❓ UDP Packet Size
**Question**: Should we fragment large FBCs or drop oversized?  
**Options**:
- A) Drop frames > 1400 bytes (simple, lossy)
- B) Fragment and reassemble (complex, adds latency)
- C) Compress first, then fragment if needed (hybrid)

**Recommendation**: Start with A, measure real-world FBC sizes

### 4. ❓ Channel Capacity
**Question**: What should bounded channel capacity be?  
**Current**: 512 slots (from visualization queue)  
**Consider**: Make configurable per stream?

### 5. ❓ feagi-zmq-server Crate
**Question**: Is this crate needed or redundant?  
**Action**: Review and remove if obsolete

---

## Success Criteria

### Must Have (Phase 1-4)
- ✅ Zero-copy FBC pipeline working
- ✅ ZMQ transport refactored under sync/
- ✅ UDP transport implemented and tested
- ✅ 200K neuron bursts handled without drops
- ✅ Backward compatible Python/Rust APIs

### Nice to Have (Phase 5+)
- ⭐ SHM transport for local deployments
- ⭐ WebSocket transport for browser agents
- ⭐ RTOS transport for embedded systems
- ⭐ Per-transport telemetry (drops, latency, throughput)

### Documentation
- 📚 Module-level docs for each transport
- 📚 Migration guide for existing code
- 📚 Performance tuning guide
- 📚 Transport selection decision tree

---

## Files to Create/Modify

### New Structure
```
feagi-core/crates/feagi-io/
├── Cargo.toml (update features)
└── src/
    ├── lib.rs (rewrite exports)
    ├── core/ (new)
    │   ├── types.rs (new - SharedFBC, errors)
    │   ├── events.rs (new - PNSEvents trait)
    │   ├── agent_registry.rs (move from src/)
    │   ├── registration.rs (move from src/)
    │   ├── heartbeat.rs (move from src/)
    │   └── config.rs (move from src/)
    ├── blocking/ (new)
    │   ├── mod.rs (new)
    │   ├── transport.rs (new - BlockingTransport trait)
    │   ├── channels.rs (new - bounded channel helpers)
    │   ├── worker.rs (new - worker thread patterns)
    │   ├── compression.rs (new - LZ4 compression)
    │   └── transports/zmq/ (move from src/zmq/)
    │       ├── mod.rs (common ZMQ code)
    │       ├── blocking_impl.rs (BlockingTransport impl)
    │       ├── visualization.rs (update for SharedFBC)
    │       ├── motor.rs (update for SharedFBC)
    │       ├── rest.rs (minor updates)
    │       └── sensory.rs (update for SharedFBC)
    └── nonblocking/ (new)
        ├── mod.rs (new)
        ├── transport.rs (new - NonBlockingTransport trait)
        ├── runtime.rs (new - tokio runtime helpers)
        ├── channels.rs (new - async channel helpers)
        ├── compression.rs (new - async LZ4)
        └── transports/udp/ (new)
            └── mod.rs (new - UDP implementation)
```

### Delete
- ❌ `feagi-core/crates/feagi-zmq-server/` (if redundant)
- ❌ `feagi-core/crates/feagi-io-core/` (never implemented)
- ❌ `feagi-core/crates/feagi-io-zmq/` (never implemented)

---

## Conclusion

This architecture provides:

1. ✅ **Scalability**: Handles 200K neurons via UDP
2. ✅ **Maintainability**: Single crate, clear module structure
3. ✅ **Flexibility**: Mix blocking/nonblocking transports as needed
4. ✅ **Performance**: Zero-copy FBC pipeline with reuse optimizations
5. ✅ **Future-proof**: Easy to add WebSocket, WebRTC, RTOS
6. ✅ **Standards-compliant**: FBC enforces FEAGI data format
7. ✅ **Event-driven**: Leverages battle-tested FeagiSignal system

**Ready to proceed with Phase 1 implementation.**

---

**Approval Required**: Review data flow pipelines and architectural decisions before implementation begins.
