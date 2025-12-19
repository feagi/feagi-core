# FEAGI-Core Crates.io Publishing Order

This document defines the correct dependency order for publishing all crates in the `feagi-core` workspace to crates.io.

**Last Updated:** January 2025  
**Workspace Version:** 0.0.1  
**Total Crates:** 19

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
- **Features:** `async` (platform-agnostic async runtime abstraction with `async-tokio`, `async-wasm`, `async-wasi` sub-features)

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
- **Note:** Includes both std and embedded implementations via features

---

### **Layer 5: Serialization & State**

#### `feagi-data-serialization`
- **Path:** `crates/feagi-data-serialization`
- **Dependencies:** `feagi-data-structures`
- **Purpose:** FEAGI Byte Container (FBC) format for binary serialization

#### `feagi-state-manager`
- **Path:** `crates/feagi-state-manager`
- **Dependencies:** `feagi-observability`, `feagi-data-structures`
- **Purpose:** Lock-free runtime state, agent registry

---

### **Layer 6: High-Performance Processing**

#### `feagi-npu-burst-engine`
- **Path:** `crates/feagi-npu/burst-engine`
- **Package Name:** `feagi-npu-burst-engine`
- **Dependencies:** 
  - `feagi-npu-neural`
  - `feagi-npu-runtime` (optional, via `std` feature)
  - `feagi-data-serialization`
  - `feagi-data-structures`
  - `feagi-state-manager`
- **Purpose:** Neural burst processing engine (CPU/GPU)

#### `feagi-npu-plasticity`
- **Path:** `crates/feagi-npu/plasticity`
- **Package Name:** `feagi-npu-plasticity`
- **Dependencies:** `feagi-npu-neural`
- **Purpose:** Synaptic plasticity (STDP, Hebbian learning)

---

### **Layer 7: Evolutionary & Development**

#### `feagi-evolutionary`
- **Path:** `crates/feagi-evolutionary`
- **Dependencies:** 
  - `feagi-npu-neural`
  - `feagi-data-structures`
  - `feagi-observability`
- **Purpose:** Genome management, evolution, validation

#### `feagi-brain-development`
- **Path:** `crates/feagi-brain-development`
- **Dependencies:**
  - `feagi-npu-neural`
  - `feagi-npu-burst-engine`
  - `feagi-evolutionary`
  - `feagi-data-structures`
  - `feagi-observability`
- **Purpose:** Brain Development Utilities (synaptogenesis, connectivity)

---

### **Layer 8: I/O Layer**

#### `feagi-io`
- **Path:** `crates/feagi-io`
- **Dependencies:**
  - `feagi-npu-burst-engine`
  - `feagi-brain-development`
  - `feagi-services`
  - `feagi-npu-neural`
  - `feagi-data-structures`
  - `feagi-data-serialization`
- **Purpose:** Agent I/O, registration, ZMQ/UDP/WebSocket transports, connectome file I/O
- **Note:** Includes consolidated transport primitives (formerly feagi-transports)

#### `feagi-pns`
- **Path:** `crates/feagi-pns`
- **Dependencies:**
  - `feagi-data-structures`
  - `feagi-data-serialization`
- **Purpose:** Peripheral Nervous System - data processing, caching, neuron voxel encoding
- **Note:** Renamed from feagi-connector-core

---

### **Layer 9: Services & API**

#### `feagi-services`
- **Path:** `crates/feagi-services`
- **Dependencies:**
  - `feagi-state-manager`
  - `feagi-npu-burst-engine`
  - `feagi-brain-development`
  - `feagi-evolutionary`
  - `feagi-npu-neural`
  - `feagi-observability`
- **Purpose:** Service trait definitions, runtime services

#### `feagi-api`
- **Path:** `crates/feagi-api`
- **Dependencies:**
  - `feagi-services`
  - `feagi-io`
  - `feagi-npu-neural`
  - `feagi-evolutionary`
  - `feagi-brain-development`
  - `feagi-npu-burst-engine`
  - `feagi-npu-runtime`
- **Purpose:** REST API, WebSocket API, OpenAPI spec

---

### **Layer 10: Agent & Platform**

#### `feagi-agent`
- **Path:** `crates/feagi-agent`
- **Dependencies:**
  - `feagi-io`
  - `feagi-data-structures`
  - `feagi-data-serialization`
  - `feagi-observability`
- **Purpose:** Agent connection lifecycle, reconnection, heartbeat

#### `feagi-embedded`
- **Path:** `crates/feagi-embedded`
- **Dependencies:**
  - `feagi-npu-runtime` (embedded feature)
  - `feagi-npu-neural`
- **Purpose:** Platform HAL abstractions (ESP32, Arduino, STM32)

---

### **Layer 11: Root Meta-Crate** (Publish Last)

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
- **19 crates** × 30 seconds = ~9.5 minutes minimum
- Add 2-3 minutes for actual publish operations
- **Total:** ~12-15 minutes for complete workspace publication

---

## 🔄 Version Synchronization

### Current Strategy: Unified Versioning
- **All crates:** Use `0.0.1` for first publication
- **Workspace inheritance:** Most crates use `version.workspace = true`
- **Future versions:** Can increment independently (e.g., `0.0.2`, `0.0.3`)

### Example:
```
First Publication:  0.0.1 (all crates)
Future Updates:     0.0.2, 0.0.3, etc. (can vary per crate)
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


