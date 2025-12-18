# FEAGI-Core Crates.io Publishing Order

This document defines the correct dependency order for publishing all crates in the `feagi-core` workspace to crates.io.

**Last Updated:** December 18, 2024  
**Workspace Version:** 2.0.0-beta.1  
**Total Crates:** 23

---

## 📦 Publication Strategy

### Automated Publishing
Use the automated script for safe, dependency-ordered publication:

```bash
# Dry run (test without publishing)
DRY_RUN=true ./scripts/publish-crates.sh

# Actual publish (requires CARGO_REGISTRY_TOKEN)
export CARGO_REGISTRY_TOKEN="your-token-here"
./scripts/publish-crates.sh
```

### Manual Publishing (Not Recommended)
If you must publish manually, follow the layer order below exactly.

---

## 🏗️ Dependency Layers

### **Layer 1: Foundation** (No internal dependencies)

#### `feagi-observability`
- **Path:** `crates/feagi-observability`
- **Dependencies:** None (workspace level)
- **Purpose:** Logging, tracing, metrics, errors
- **Publish First:** Yes

---

### **Layer 2: Core Data Structures**

#### `feagi-data-structures`
- **Path:** `crates/feagi-data-structures`
- **Dependencies:** `feagi-observability`
- **Purpose:** Neurons, synapses, cortical areas, genome structures

#### `feagi-config`
- **Path:** `crates/feagi-config`
- **Dependencies:** `feagi-observability`
- **Purpose:** TOML configuration loading, validation

---

### **Layer 3: Neural Foundations**

#### `feagi-npu-neural`
- **Path:** `crates/feagi-npu/neural`
- **Package Name:** `feagi-npu-neural`
- **Dependencies:** `feagi-observability`, `feagi-data-structures`
- **Purpose:** Core neural types (NeuronId, SynapseId, membrane potentials)

---

### **Layer 4: Runtime Abstractions**

#### `feagi-npu-runtime`
- **Path:** `crates/feagi-npu/runtime`
- **Package Name:** `feagi-npu-runtime`
- **Dependencies:** `feagi-npu-neural`
- **Purpose:** Platform-agnostic runtime traits (NeuronStorage, SynapseStorage)

---

### **Layer 5: Runtime Implementations**

#### `feagi-npu-runtime-std`
- **Path:** `crates/feagi-npu/runtime-std`
- **Package Name:** `feagi-npu-runtime-std`
- **Dependencies:** `feagi-npu-runtime`, `feagi-npu-neural`
- **Purpose:** Standard library runtime (desktop/server)

#### `feagi-npu-runtime-embedded`
- **Path:** `crates/feagi-npu/runtime-embedded`
- **Package Name:** `feagi-npu-runtime-embedded`
- **Dependencies:** `feagi-npu-runtime`, `feagi-npu-neural`
- **Purpose:** Embedded/no_std runtime (ESP32, STM32, etc.)

---

### **Layer 6: Serialization & State**

#### `feagi-connectome-serialization`
- **Path:** `crates/feagi-connectome-serialization`
- **Dependencies:** `feagi-data-structures`
- **Purpose:** Connectome save/load (MessagePack, JSON)

#### `feagi-state-manager`
- **Path:** `crates/feagi-state-manager`
- **Dependencies:** `feagi-observability`, `feagi-data-structures`
- **Purpose:** Lock-free runtime state, agent registry

---

### **Layer 7: High-Performance Processing**

#### `feagi-npu-burst-engine`
- **Path:** `crates/feagi-npu/burst-engine`
- **Package Name:** `feagi-npu-burst-engine`
- **Dependencies:** 
  - `feagi-npu-neural`
  - `feagi-npu-runtime`
  - `feagi-npu-runtime-std` (optional, via `std` feature)
  - `feagi-connectome-serialization` (optional)
  - `feagi-data-structures`
- **Purpose:** Neural burst processing engine (CPU/GPU)

#### `feagi-npu-plasticity`
- **Path:** `crates/feagi-npu/plasticity`
- **Package Name:** `feagi-npu-plasticity`
- **Dependencies:** `feagi-npu-neural`
- **Purpose:** Synaptic plasticity (STDP, Hebbian learning)

---

### **Layer 8: Evolutionary & Development**

#### `feagi-evo`
- **Path:** `crates/feagi-evo`
- **Dependencies:** 
  - `feagi-npu-neural`
  - `feagi-data-structures`
  - `feagi-observability`
- **Purpose:** Genome management, evolution, validation

#### `feagi-bdu`
- **Path:** `crates/feagi-bdu`
- **Dependencies:**
  - `feagi-npu-neural`
  - `feagi-npu-burst-engine`
  - `feagi-evo`
  - `feagi-data-structures`
  - `feagi-observability`
- **Purpose:** Brain Development Utilities (synaptogenesis, connectivity)

---

### **Layer 9: Async Runtime**

#### `feagi-async`
- **Path:** `crates/feagi-async`
- **Dependencies:** `feagi-observability`
- **Purpose:** Platform-agnostic async runtime traits (Tokio, WASM)

---

### **Layer 10: Transport & I/O**

#### `feagi-transports`
- **Path:** `crates/feagi-transports`
- **Dependencies:** 
  - `feagi-observability`
  - `feagi-async`
- **Purpose:** ZMQ, WebSocket transports

#### `feagi-io`
- **Path:** `crates/feagi-io`
- **Dependencies:**
  - `feagi-data-structures`
  - `feagi-observability`
  - `feagi-transports`
- **Purpose:** I/O type validation, sensory/motor data processing

#### `feagi-connector-core`
- **Path:** `crates/feagi-connector-core`
- **Dependencies:**
  - `feagi-data-structures`
  - `feagi-io`
  - `feagi-transports`
  - `feagi-observability`
- **Purpose:** Agent connection, data pipelines, caching

---

### **Layer 11: Agent & Services**

#### `feagi-agent`
- **Path:** `crates/feagi-agent`
- **Dependencies:**
  - `feagi-connector-core`
  - `feagi-transports`
  - `feagi-data-structures`
  - `feagi-observability`
- **Purpose:** Agent connection lifecycle, reconnection, heartbeat

#### `feagi-services`
- **Path:** `crates/feagi-services`
- **Dependencies:**
  - `feagi-state-manager`
  - `feagi-npu-burst-engine`
  - `feagi-bdu`
  - `feagi-evo`
  - `feagi-observability`
- **Purpose:** Service trait definitions, runtime services

---

### **Layer 12: API Server**

#### `feagi-api`
- **Path:** `crates/feagi-api`
- **Dependencies:**
  - `feagi-services`
  - `feagi-state-manager`
  - `feagi-bdu`
  - `feagi-transports`
  - `feagi-data-structures`
  - `feagi-observability`
- **Purpose:** REST API, WebSocket API, OpenAPI spec

---

### **Layer 13: Platform-Specific**

#### `feagi-embedded`
- **Path:** `crates/feagi-embedded`
- **Dependencies:**
  - `feagi-npu-runtime-embedded`
  - `feagi-npu-neural`
- **Purpose:** Platform HAL abstractions (ESP32, Arduino, STM32)

---

### **Layer 14: Root Meta-Crate** (Publish Last)

#### `feagi` (workspace root)
- **Path:** `.` (root)
- **Dependencies:** ALL workspace crates
- **Purpose:** Umbrella crate with feature flags for selective compilation

**⚠️ IMPORTANT:** Must be published LAST as it depends on all other crates being available on crates.io.

---

## ⏱️ Timing Considerations

### Crates.io Indexing Delay
- **Required:** 30 seconds between each publish
- **Reason:** Crates.io needs time to index each crate before dependents can find it
- **Automated:** The `publish-crates.sh` script handles this automatically

### Total Publish Time
- **23 crates** × 30 seconds = ~12 minutes minimum
- Add 2-3 minutes for actual publish operations
- **Total:** ~15 minutes for complete workspace publication

---

## 🔄 Version Synchronization

### Current Strategy: Hybrid Versioning
- **Major version:** Synchronized across all crates (`2.x.x`)
- **Minor/Patch:** Can vary independently
- **Beta suffix:** Applied to all crates during staging releases

### Example:
```
Staging:  2.0.0-beta.1, 2.0.0-beta.2, ...
Main:     2.0.0 (production)
```

---

## ✅ Pre-Publication Checklist

Before publishing, ensure:

- [ ] All library tests pass (`cargo test --workspace --lib`)
- [ ] Clippy checks pass (`cargo clippy --workspace --lib --tests`)
- [ ] All crates have required metadata (name, version, description, license, authors)
- [ ] All crates package successfully (`cargo package` in each crate dir)
- [ ] Version numbers are consistent with release strategy
- [ ] `CARGO_REGISTRY_TOKEN` environment variable is set
- [ ] No path dependencies in published crates (use workspace dependencies)

---

## 🚨 Common Issues

### Issue: "crate not found" during publish
**Cause:** Previous crate in dependency chain not indexed yet  
**Solution:** Wait 30 seconds between publishes (automated in script)

### Issue: "failed to verify package tarball"
**Cause:** Missing required metadata or invalid Cargo.toml  
**Solution:** Run metadata verification workflow first

### Issue: "version already published"
**Cause:** Attempting to republish existing version  
**Solution:** Bump version number, crates.io versions are immutable

---

## 📚 Additional Resources

- **Publish Script:** `scripts/publish-crates.sh`
- **CI Workflows:** `.github/workflows/`
- **Crates.io Guide:** https://doc.rust-lang.org/cargo/reference/publishing.html
- **Workspace Guide:** `REPOSITORY_MERGE_COMPLETE.md`


