# Phase 0: Dependency Analysis

**Generated:** 2025-10-29  
**Purpose:** Map Python dependencies to Rust equivalents for migration

---

## 1. Service Architecture

### Python Service Hierarchy

```
CoreAPIService (Facade)
├── SystemService          (health, metrics)
├── GenomeService          (genome CRUD)
├── CorticalAreaService    (cortical area management)
├── ConnectomeService      (connectome operations)
├── BrainService           (brain lifecycle)
├── AgentsService          (agent registration)
├── NetworkService         (network config)
└── NPUService             (burst engine interface)
```

**All services extend:** `BaseService`
- Provides: connectome_manager, state_manager, logger

**Key Dependencies:**
- `ConnectomeManager` (BDU layer)
- `FeagiStateManager` (shared state)
- `NPUInterface` (Rust NPU bridge)

---

## 2. External Library Dependencies

### Core Runtime Dependencies

| Python Library | Version | Purpose | Rust Equivalent | Migration Notes |
|----------------|---------|---------|-----------------|-----------------|
| **fastapi** | >=0.95.0 | REST API framework | **axum** 0.7 + **utoipa** | ✅ Axum is production-ready, tokio-backed |
| **uvicorn** | >=0.21.0 | ASGI server | **axum** (built-in) | ✅ Axum has its own HTTP server |
| **pydantic** | >=2.0.0 | Data validation | **serde** + **validator** | ✅ `serde` for serialization, `validator` for validation |
| **pydantic-settings** | >=2.0.0 | Config management | **config** crate | ✅ `config` crate for TOML/env |
| **numpy** | >=1.24.0 | Numerical arrays | **ndarray** 0.15 | ✅ `ndarray` is mature, GPU-ready |
| **scipy** | >=1.10.0 | Scientific computing | **ndarray** + **statrs** | ⚠️ Partial coverage, may need custom impl |
| **httpx** | >=0.24.0 | HTTP client | **reqwest** 0.11 | ✅ `reqwest` is tokio-based |
| **PyYAML** | >=6.0.0 | YAML parsing | **serde_yaml** 0.9 | ✅ Full coverage |
| **tomli** | >=2.0.0 | TOML parsing | **toml** 0.8 | ✅ Standard library in Rust |
| **python-jose** | >=3.3.0 | JWT/crypto | **jsonwebtoken** 9.2 | ✅ Production-ready |
| **passlib** | >=1.7.4 | Password hashing | **argon2** 0.5 | ✅ Recommended by OWASP |
| **psutil** | >=5.8.0 | System monitoring | **sysinfo** 0.30 | ✅ Cross-platform |
| **packaging** | >=21.0.0 | Version parsing | **semver** 1.0 | ✅ Semantic versioning |

### Development Dependencies (Not Needed in Rust)

| Python Tool | Purpose | Rust Equivalent | Notes |
|-------------|---------|-----------------|-------|
| **pytest** | Testing | **cargo test** | Built-in |
| **black** | Formatting | **rustfmt** | Built-in |
| **isort** | Import sorting | **rustfmt** | Built-in |
| **mypy** | Type checking | **rust compiler** | Built-in |
| **ruff** | Linting | **clippy** | Built-in |
| **pre-commit** | Git hooks | **husky** (optional) | N/A |

---

## 3. Python Standard Library Usage

### Heavy Usage (Need Rust Equivalents)

| Python Module | Usage | Rust Equivalent |
|---------------|-------|-----------------|
| `json` | JSON serialization | `serde_json` |
| `datetime` | Date/time handling | `chrono` 0.4 |
| `time` | Time measurement | `std::time` |
| `logging` | Structured logging | `tracing` 0.1 |
| `os` / `pathlib` | File paths | `std::path` |
| `tempfile` | Temp files | `tempfile` 0.3 |
| `threading` | Concurrency | `tokio` 1.35 |
| `copy` | Deep copy | `.clone()` |
| `random` | Random numbers | `rand` 0.8 |
| `string` | String manipulation | `std::string` |
| `enum` | Enumerations | `enum` |
| `dataclasses` | Data structures | `struct` + `serde` |
| `typing` | Type hints | Native types |

### Minimal Usage (Direct Port)

- `collections.defaultdict` → `HashMap::default()`
- `sys` → `std::env`
- `math` → `std::f64`

---

## 4. Service-Level Dependencies

### SystemService
**Dependencies:**
- `psutil` → **sysinfo** ✅
- `logging` → **tracing** ✅
- State manager (already Rust)

**Migration Complexity:** 🟢 Low

---

### GenomeService
**Dependencies:**
- `json` → **serde_json** ✅
- `PyYAML` → **serde_yaml** ✅
- `datetime` → **chrono** ✅
- File I/O → **std::fs** ✅

**Migration Complexity:** 🟢 Low

---

### CorticalAreaService
**Dependencies:**
- `numpy` → **ndarray** ✅
- ConnectomeManager (Rust BDU)
- State manager (Rust)

**Migration Complexity:** 🟡 Medium (depends on BDU completion)

---

### ConnectomeService
**Dependencies:**
- ConnectomeManager (Rust BDU) - **CORE DEPENDENCY**
- `numpy` → **ndarray** ✅

**Migration Complexity:** 🔴 High (blocked by BDU migration)

---

### BrainService
**Dependencies:**
- Genome loading (GenomeService)
- NPU initialization (Rust NPU)
- State manager (Rust)

**Migration Complexity:** 🟢 Low (depends on GenomeService)

---

### AgentsService
**Dependencies:**
- State manager (Rust)
- `httpx` → **reqwest** ✅ (for agent heartbeat)

**Migration Complexity:** 🟢 Low

---

### NetworkService
**Dependencies:**
- Config management → **config** crate ✅
- State manager (Rust)

**Migration Complexity:** 🟢 Low

---

### NPUService
**Dependencies:**
- **Already Rust** (`feagi_rust_py_libs`)
- Direct FFI bridge

**Migration Complexity:** 🟢 Low (thin wrapper)

---

## 5. Critical Migration Blockers

### ❌ **ConnectomeManager (BDU Layer)**
- **Status:** Python (~7,000 LOC after cleanup)
- **Blocking:** ConnectomeService, CorticalAreaService
- **Priority:** Phase 1-2 (Weeks 2-7)

### ✅ **NPUInterface**
- **Status:** Already Rust
- **Provides:** Neuron/synapse operations, burst engine
- **Unblocked**

### ✅ **State Manager**
- **Status:** Rust (`feagi-state-manager`)
- **Provides:** Global state access
- **Unblocked**

---

## 6. Migration Order (Service Layer)

### Phase 1 - Independent Services (Week 11)
**Can migrate immediately (no BDU dependency):**
1. SystemService ✅
2. GenomeService ✅  
3. BrainService ✅
4. AgentsService ✅
5. NetworkService ✅
6. NPUService ✅ (thin wrapper)

**Estimated time:** 1 week

---

### Phase 2 - BDU-Dependent Services (Week 13)
**Requires ConnectomeManager migration:**
7. CorticalAreaService ⚠️
8. ConnectomeService ⚠️

**Estimated time:** 2 weeks (after BDU completion)

---

### Phase 3 - API Layer (Week 12)
**REST API Migration:**
- Axum setup
- OpenAPI/utoipa
- Middleware (auth, CORS, error handling)
- Route handlers

**Estimated time:** 1 week (parallel with Phase 1)

---

## 7. Rust Crate Recommendations

### API Layer
```toml
axum = "0.7"
tower = "0.4"  # Middleware
tower-http = "0.5"  # CORS, tracing
utoipa = "4.2"  # OpenAPI
utoipa-swagger-ui = "6.0"  # Swagger UI
```

### Data & Serialization
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
validator = { version = "0.18", features = ["derive"] }
```

### Numerical Computing
```toml
ndarray = "0.15"
ndarray-rand = "0.14"  # Random arrays
statrs = "0.17"  # Statistics
```

### Async Runtime
```toml
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"
futures = "0.3"
```

### Logging & Observability
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### Utilities
```toml
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["serde", "v4"] }
config = "0.14"  # Config management
tempfile = "3.8"
sysinfo = "0.30"
```

### Security & Auth
```toml
jsonwebtoken = "9.2"
argon2 = "0.5"
```

### HTTP Client
```toml
reqwest = { version = "0.11", features = ["json"] }
```

---

## 8. Python-Only Components (Not Migrating)

### Training & Evolution
- **Location:** `feagi/evo/`
- **Status:** Keeping in Python for flexibility
- **Access:** Via Rust → Python FFI (PyO3) if needed

### Legacy Tools
- **Location:** `feagi/legacy/`
- **Status:** Deprecated, delete

---

## 9. Summary

### ✅ Ready to Migrate (No Blockers)
- **6 services:** System, Genome, Brain, Agents, Network, NPU
- **API layer:** Axum + Utoipa
- **All Rust crates available**

### ⚠️ Blocked by BDU
- **2 services:** CorticalArea, Connectome
- **Unblocks in:** Week 7 (after BDU migration)

### 🎯 Total Migration Timeline
- **Phase 1 (API + 6 services):** Weeks 11-12 (2 weeks)
- **Phase 2 (2 BDU services):** Week 13 (1 week)
- **Total:** 3 weeks (after BDU completion)

---

## 10. Conclusion

**All external dependencies have Rust equivalents.** No compatibility blockers exist. The migration is straightforward for the service layer, with the primary dependency being the completion of the Rust BDU (Phase 1-2).

**Next Steps:**
1. ✅ Delete dead code (completed: 930 lines)
2. ✅ Document dependencies (this document)
3. → Begin Phase 1: BDU migration (Weeks 2-7)
4. → Begin Phase 3: Service migration (Weeks 11-13)





