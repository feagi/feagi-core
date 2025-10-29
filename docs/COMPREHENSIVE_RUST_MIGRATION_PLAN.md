# Comprehensive Rust Migration Plan - Full Stack

**Date:** 2025-10-28  
**Strategy:** Complete Python → Rust migration (API + Services + BDU)  
**Goal:** Pure Rust stack, no intermediate hybrid state  
**Timeline:** 16-20 weeks (4-5 months)

---

## Executive Summary

### Migration Scope

**Everything moves to Rust in coordinated phases:**
- ✅ API Layer: FastAPI → Axum
- ✅ Service Layer: Python services → Rust services
- ✅ Business Logic: Python BDU → Rust BDU
- ✅ State Management: Python → Rust (already done)
- ✅ Burst Engine: Python → Rust (already done)

**NO intermediate states:** Direct Python → Rust migration

---

## Strategic Benefits

### Why Full Rust Migration?

1. **No Wasted Effort**
   - Skip PyO3 FFI bridges (would be deleted later)
   - Skip hybrid Python/Rust complexity
   - One migration, done right

2. **Performance**
   - 10-100x faster across all layers
   - No FFI boundary overhead
   - Native async/await (Tokio)

3. **Type Safety**
   - End-to-end type checking
   - Compile-time error detection
   - No runtime type mismatches

4. **RTOS/Embedded Ready**
   - `no_std` compatible from day 1
   - Deploy on embedded hardware
   - Minimal resource footprint

5. **Maintainability**
   - Single language
   - Unified tooling (cargo, clippy, rustfmt)
   - Better refactoring

---

## Current Python Architecture

```
┌─────────────────────────────────────────┐
│  API Layer (FastAPI)                    │
│  feagi/api/v1/*.py                      │
│  - REST endpoints                       │
│  - Request validation                   │
│  - Response serialization               │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│  Service Layer (Business Logic)         │
│  feagi/api/core/services/               │
│                                          │
│  CoreAPIService (Facade)                │
│  ├── SystemService                      │
│  ├── GenomeService                      │
│  ├── CorticalAreaService                │
│  ├── ConnectomeService                  │
│  ├── BrainService                       │
│  ├── AgentsService                      │
│  ├── NetworkService                     │
│  └── NPUService                         │
│                                          │
│  Responsibilities:                       │
│  - Complex validation                   │
│  - State synchronization                │
│  - Multi-manager orchestration          │
│  - Error handling & recovery            │
└─────────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────────┐
│  Data Layer (BDU)                       │
│  feagi/bdu/connectome_manager.py        │
│                                          │
│  ConnectomeManager                      │
│  - CRUD operations                      │
│  - Data structures                      │
│  - Business rules                       │
└─────────────────────────────────────────┘
```

---

## Target Rust Architecture (Modular Subcrates)

```
┌───────────────────────────────────────────────────────────────────┐
│  feagi-core (workspace with 7 subcrates)                          │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Full Stack Subcrates (server only):                     │    │
│  │  • feagi-api         (REST API - Axum)                  │    │
│  │  • feagi-services    (Service layer)                    │    │
│  │  • feagi-pns         (I/O - ZMQ)                        │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Core Subcrates (reusable, modular):                     │    │
│  │  • feagi-bdu         (Business logic)                   │    │
│  │  • feagi-npu         (Burst engine)                     │    │
│  │  • feagi-state       (State manager)                    │    │
│  │  • feagi-config      (Config loader)                    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
    ↓ (core subcrates consumed by)
┌─────────────────────┬─────────────────────┬───────────────────────┐
│ feagi-inference-    │ feagi-web           │ feagi-py              │
│ engine (embedded)   │ (WASM for browser)  │ (Python bindings)     │
│                     │                     │                       │
│ Uses: npu, state,   │ Uses: npu, bdu,     │ Uses: ALL             │
│       bdu, config   │       state         │                       │
└─────────────────────┴─────────────────────┴───────────────────────┘
```

**Same 3-tier architecture, pure Rust, modular subcrates!**

**Key architectural decision:** 7 subcrates for modularity:
- **Full Stack (3):** api, services, pns → Server only
- **Core (4):** bdu, npu, state, config → Reusable by inference-engine, web, py

---

## Migration Phases

### Phase 0: Preparation (Week 1)

**Goal:** Clean Python codebase and analyze dependencies

#### 0.1 Delete Dead Code
- Delete 106 unused BDU methods (54% of BDU)
- Identify unused service methods
- Remove deprecated features
- **Result:** ~50% smaller codebase to migrate

#### 0.2 Dependency Analysis
- Map all Python service dependencies
- Identify external libraries used
- Find Rust equivalents
- Document API contracts

#### 0.3 Testing Infrastructure
- Document all Python tests
- Create test migration plan
- Set up Rust test framework

---

### Phase 1: Core Data Layer (Weeks 2-3)

**Goal:** Rust BDU foundation and data structures

#### 1.1 Core Data Structures

```rust
// feagi-core/crates/feagi-bdu/src/models/mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalArea {
    pub cortical_id: String,
    pub cortical_idx: u32,
    pub cortical_name: String,
    pub coordinates_2d: Option<(i32, i32)>,
    pub coordinates_3d: (i32, i32, i32),
    pub dimensions: (u32, u32, u32),
    pub group_id: Option<String>,
    pub sub_group_id: Option<u32>,
    pub neuron_count: u32,
    pub synaptic_attractivity: f32,
    pub neuron_params: NeuronParameters,
    // ... 15 more fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegion {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub coordinates_2d: Option<(i32, i32)>,
    pub coordinates_3d: (i32, i32, i32),
    pub sub_regions: Vec<String>,
    pub cortical_areas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BrainRegionHierarchy {
    regions: HashMap<String, BrainRegion>,
    hierarchy: petgraph::Graph<String, ()>,
    input_areas: HashSet<String>,
    output_areas: HashSet<String>,
}

#[derive(Debug)]
pub struct BiDirectionalCorticalMap {
    id_to_idx: HashMap<String, u32>,
    idx_to_id: HashMap<u32, String>,
}
```

**Deliverables:**
- ✅ All BDU data structures in Rust
- ✅ Serde serialization/deserialization
- ✅ Unit tests for data structures

**Timeline:** 2 weeks

---

### Phase 2: BDU Business Logic (Weeks 4-7)

**Goal:** Migrate ConnectomeManager (89 methods)

#### 2.1 ConnectomeManager Singleton

```rust
// feagi-core/crates/feagi-bdu/src/connectome_manager.rs

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

static CONNECTOME_MANAGER: Lazy<Arc<RwLock<ConnectomeManager>>> = 
    Lazy::new(|| Arc::new(RwLock::new(ConnectomeManager::new(10_000_000, 100_000_000))));

pub struct ConnectomeManager {
    cortical_areas: HashMap<String, CorticalArea>,
    cortical_map: BiDirectionalCorticalMap,
    brain_regions: BrainRegionHierarchy,
    max_neurons: usize,
    max_synapses: usize,
    neuron_count: AtomicU64,
    synapse_count: AtomicU64,
    npu: Option<Arc<RwLock<RustNPU>>>,
}

impl ConnectomeManager {
    pub fn instance() -> Arc<RwLock<Self>> {
        CONNECTOME_MANAGER.clone()
    }
    
    fn new(max_neurons: usize, max_synapses: usize) -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_map: BiDirectionalCorticalMap::new(),
            brain_regions: BrainRegionHierarchy::new(),
            max_neurons,
            max_synapses,
            neuron_count: AtomicU64::new(0),
            synapse_count: AtomicU64::new(0),
            npu: None,
        }
    }
}
```

#### 2.2 Core Methods (Week 4)

**Cortical Area CRUD:**
```rust
impl ConnectomeManager {
    pub fn add_cortical_area(&mut self, area: CorticalArea) -> Result<(), BduError> {
        if self.cortical_areas.contains_key(&area.cortical_id) {
            return Err(BduError::AreaAlreadyExists(area.cortical_id.clone()));
        }
        
        self.cortical_map.add_mapping(area.cortical_id.clone(), area.cortical_idx);
        self.cortical_areas.insert(area.cortical_id.clone(), area);
        
        Ok(())
    }
    
    pub fn delete_cortical_area(&mut self, cortical_id: &str) -> Result<(), BduError> {
        let area = self.cortical_areas.remove(cortical_id)
            .ok_or_else(|| BduError::AreaNotFound(cortical_id.to_string()))?;
        
        self.cortical_map.remove_by_id(cortical_id);
        
        Ok(())
    }
    
    pub fn get_cortical_area(&self, cortical_id: &str) -> Option<&CorticalArea> {
        self.cortical_areas.get(cortical_id)
    }
    
    pub fn update_cortical_area_properties(
        &mut self,
        cortical_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<(), BduError> {
        let area = self.cortical_areas.get_mut(cortical_id)
            .ok_or_else(|| BduError::AreaNotFound(cortical_id.to_string()))?;
        
        // Apply updates (similar to Python implementation)
        for (key, value) in updates {
            match key.as_str() {
                "cortical_name" => area.cortical_name = value.as_str().unwrap().to_string(),
                "dimensions" => { /* ... */ },
                // ... handle all properties
                _ => return Err(BduError::InvalidProperty(key)),
            }
        }
        
        Ok(())
    }
}
```

**Deliverables:**
- ✅ 30 core CRUD methods
- ✅ Unit tests for each method
- ✅ Error handling

**Timeline:** 1 week

#### 2.3 Neuron & Synapse Management (Week 5)

```rust
impl ConnectomeManager {
    pub fn batch_create_neurons(
        &mut self,
        cortical_id: &str,
        positions: Vec<(u32, u32, u32)>,
        params: NeuronParameters,
    ) -> Result<Vec<u64>, BduError> {
        let area = self.cortical_areas.get(cortical_id)
            .ok_or_else(|| BduError::AreaNotFound(cortical_id.to_string()))?;
        
        // Allocate neuron IDs and register with NPU
        let npu = self.npu.as_ref()
            .ok_or(BduError::NpuNotInitialized)?;
        
        let neuron_ids = npu.write().create_neurons(cortical_id, &positions, &params)?;
        
        self.neuron_count.fetch_add(neuron_ids.len() as u64, Ordering::Relaxed);
        
        Ok(neuron_ids)
    }
    
    pub fn batch_create_synapses(
        &mut self,
        src_neurons: &[u64],
        dst_neurons: &[u64],
        weights: &[f32],
    ) -> Result<usize, BduError> {
        let npu = self.npu.as_ref()
            .ok_or(BduError::NpuNotInitialized)?;
        
        let count = npu.write().create_synapses(src_neurons, dst_neurons, weights)?;
        
        self.synapse_count.fetch_add(count as u64, Ordering::Relaxed);
        
        Ok(count)
    }
}
```

**Deliverables:**
- ✅ 22 neuron/synapse methods
- ✅ NPU integration
- ✅ Unit tests

**Timeline:** 1 week

#### 2.4 Genome Loading (Week 6)

```rust
// feagi-core/crates/feagi-bdu/src/embryogenesis/neuroembryogenesis.rs

pub struct Neuroembryogenesis {
    morphology_registry: HashMap<String, Box<dyn MorphologyFunction>>,
}

impl Neuroembryogenesis {
    pub fn load_genome(&mut self, genome_data: GenomeData) -> Result<(), BduError> {
        // Parse and validate genome
        self.validate_genome(&genome_data)?;
        
        Ok(())
    }
    
    pub fn develop_brain_from_genome_data(
        &mut self,
        genome: &GenomeData,
        connectome: &mut ConnectomeManager,
    ) -> Result<DevelopmentStats, BduError> {
        let start = std::time::Instant::now();
        
        // Create cortical areas from genome
        let areas_created = self.create_cortical_areas(genome, connectome)?;
        
        // Apply connectivity rules
        let synapses_created = self.apply_connectivity_rules(genome, connectome)?;
        
        // Populate neurons
        let neurons_created = self.populate_neurons(genome, connectome)?;
        
        Ok(DevelopmentStats {
            areas_created,
            neurons_created,
            synapses_created,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}
```

**Deliverables:**
- ✅ Genome loading logic
- ✅ Brain development algorithm
- ✅ Integration tests

**Timeline:** 1 week

#### 2.5 Remaining Methods (Week 7)

- Mapping & positioning (3 methods)
- State queries (7 methods)
- Utilities (21 methods)

**Timeline:** 1 week

**Phase 2 Total: 4 weeks**

---

### Phase 3: Service Layer (Weeks 8-11)

**Goal:** Migrate all 8 domain services to Rust

#### 3.1 Base Service (Week 8)

```rust
// feagi-core/crates/feagi-services/src/base_service.rs

use std::sync::Arc;
use parking_lot::RwLock;
use crate::bdu::ConnectomeManager;
use crate::state::RustStateManager;

pub trait BaseService {
    fn connectome_manager(&self) -> Arc<RwLock<ConnectomeManager>>;
    fn state_manager(&self) -> Arc<RwLock<RustStateManager>>;
    
    fn validate_connectome_ready(&self) -> Result<(), ServiceError> {
        let mgr = self.connectome_manager();
        let guard = mgr.read();
        
        if guard.get_neuron_count() == 0 && guard.get_cortical_area_count() == 0 {
            return Err(ServiceError::ConnectomeNotReady);
        }
        
        Ok(())
    }
    
    fn validate_connectome_stable(&self) -> Result<(), ServiceError> {
        let state = self.state_manager();
        let guard = state.read();
        
        match guard.get_connectome_state() {
            ConnectomeState::Ready => Ok(()),
            state => Err(ServiceError::ConnectomeNotStable(state)),
        }
    }
    
    fn validate_genome_loaded(&self) -> Result<(), ServiceError> {
        let state = self.state_manager();
        let guard = state.read();
        
        if !guard.is_genome_loaded() {
            return Err(ServiceError::GenomeNotLoaded);
        }
        
        Ok(())
    }
    
    fn sync_state_if_needed(&self) -> Result<(), ServiceError> {
        // Synchronize state manager with connectome manager
        let connectome = self.connectome_manager();
        let state = self.state_manager();
        
        let conn_guard = connectome.read();
        let mut state_guard = state.write();
        
        state_guard.set_brain_stats(BrainStats {
            neuron_count: conn_guard.get_neuron_count(),
            synapse_count: conn_guard.get_synapse_count(),
            cortical_area_count: conn_guard.get_cortical_area_count(),
        })?;
        
        Ok(())
    }
}
```

**Deliverables:**
- ✅ BaseService trait
- ✅ Common validation methods
- ✅ State synchronization

**Timeline:** 3 days

#### 3.2 Core Services (Week 8-9)

**CorticalAreaService:**
```rust
// feagi-core/crates/feagi-services/src/cortical_area_service.rs

pub struct CorticalAreaService {
    connectome_manager: Arc<RwLock<ConnectomeManager>>,
    state_manager: Arc<RwLock<RustStateManager>>,
    genome_service: Arc<GenomeService>,
}

impl BaseService for CorticalAreaService {
    fn connectome_manager(&self) -> Arc<RwLock<ConnectomeManager>> {
        self.connectome_manager.clone()
    }
    
    fn state_manager(&self) -> Arc<RwLock<RustStateManager>> {
        self.state_manager.clone()
    }
}

impl CorticalAreaService {
    pub fn add_cortical_area(&self, area: CorticalArea) -> Result<(), ServiceError> {
        // Validation
        self.validate_connectome_stable()?;
        self.validate_area_data(&area)?;
        
        // Business logic
        self.connectome_manager.write()
            .add_cortical_area(area)
            .map_err(ServiceError::BduError)?;
        
        // State sync
        self.sync_state_if_needed()?;
        
        Ok(())
    }
    
    pub fn get_cortical_area(&self, cortical_id: &str) -> Result<CorticalArea, ServiceError> {
        self.validate_genome_loaded()?;
        
        self.connectome_manager.read()
            .get_cortical_area(cortical_id)
            .cloned()
            .ok_or_else(|| ServiceError::AreaNotFound(cortical_id.to_string()))
    }
    
    // ... 20 more methods from Python CorticalAreaService
}
```

**Services to migrate:**
1. **SystemService** - Health checks, system state
2. **GenomeService** - Genome loading, validation
3. **CorticalAreaService** - Cortical area management (largest)
4. **ConnectomeService** - Connectome queries
5. **BrainService** - Brain state, neuron queries
6. **AgentsService** - Agent management
7. **NetworkService** - Network configuration
8. **NPUService** - Already exists (minimal changes)

**Deliverables:**
- ✅ All 8 services in Rust
- ✅ Same public interface as Python
- ✅ Unit tests for each service

**Timeline:** 2 weeks

#### 3.3 CoreAPIService Facade (Week 10)

```rust
// feagi-core/crates/feagi-services/src/core_api_service.rs

pub struct CoreAPIService {
    system_service: Arc<SystemService>,
    genome_service: Arc<GenomeService>,
    cortical_area_service: Arc<CorticalAreaService>,
    connectome_service: Arc<ConnectomeService>,
    brain_service: Arc<BrainService>,
    agents_service: Arc<AgentsService>,
    network_service: Arc<NetworkService>,
    npu_service: Arc<NPUService>,
}

impl CoreAPIService {
    pub fn new(
        connectome_manager: Arc<RwLock<ConnectomeManager>>,
        state_manager: Arc<RwLock<RustStateManager>>,
    ) -> Self {
        let system_service = Arc::new(SystemService::new(
            connectome_manager.clone(),
            state_manager.clone(),
        ));
        
        let genome_service = Arc::new(GenomeService::new(
            connectome_manager.clone(),
            state_manager.clone(),
        ));
        
        let cortical_area_service = Arc::new(CorticalAreaService::new(
            connectome_manager.clone(),
            state_manager.clone(),
            genome_service.clone(),
        ));
        
        // ... initialize all services
        
        Self {
            system_service,
            genome_service,
            cortical_area_service,
            connectome_service,
            brain_service,
            agents_service,
            network_service,
            npu_service,
        }
    }
    
    // Delegate to services
    pub fn get_system_health(&self) -> Result<SystemHealth, ServiceError> {
        self.system_service.get_health()
    }
    
    pub fn add_cortical_area(&self, area: CorticalArea) -> Result<(), ServiceError> {
        self.cortical_area_service.add_cortical_area(area)
    }
    
    // ... delegate all methods to appropriate services
}
```

**Deliverables:**
- ✅ CoreAPIService facade
- ✅ All delegation methods
- ✅ Integration tests

**Timeline:** 1 week

**Phase 3 Total: 4 weeks**

---

### Phase 4: API Layer (Weeks 12-15)

**Goal:** Migrate FastAPI to Axum

#### 4.1 API Infrastructure (Week 12)

**Dependencies:**
```toml
[dependencies]
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
utoipa = { version = "4", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "6", features = ["axum"] }
validator = { version = "0.16", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

**API Setup:**
```rust
// feagi-core/crates/feagi-api/src/app.rs

use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::State,
    Json,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        get_cortical_areas,
        add_cortical_area,
        // ... all endpoints
    ),
    components(
        schemas(CorticalArea, BrainRegion, SystemHealth, /* ... */)
    ),
    tags(
        (name = "system", description = "System endpoints"),
        (name = "cortical_area", description = "Cortical area management"),
        // ... all tags
    )
)]
struct ApiDoc;

pub struct AppState {
    core_api_service: Arc<CoreAPIService>,
}

pub fn create_app(core_api_service: Arc<CoreAPIService>) -> Router {
    let state = Arc::new(AppState { core_api_service });
    
    Router::new()
        // System endpoints
        .route("/v1/health", get(health_check))
        .route("/v1/system/status", get(system_status))
        
        // Cortical area endpoints
        .route("/v1/cortical_area", get(get_cortical_areas))
        .route("/v1/cortical_area", post(add_cortical_area))
        .route("/v1/cortical_area/:id", get(get_cortical_area))
        .route("/v1/cortical_area/:id", put(update_cortical_area))
        .route("/v1/cortical_area/:id", delete(delete_cortical_area))
        
        // ... all other endpoints
        
        // OpenAPI docs
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        
        .layer(CorsLayer::permissive())
        .with_state(state)
}
```

**Deliverables:**
- ✅ Axum setup
- ✅ OpenAPI documentation
- ✅ CORS configuration
- ✅ Error handling middleware

**Timeline:** 3 days

#### 4.2 System Endpoints (Week 12)

```rust
// feagi-core/crates/feagi-api/src/endpoints/system.rs

use axum::{
    extract::State,
    Json,
};
use utoipa::ToSchema;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub brain_readiness: bool,
    pub burst_engine: bool,
    pub neuron_count: u64,
    pub synapse_count: u64,
    // ... all fields from Python
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/v1/health",
    responses(
        (status = 200, description = "Health check successful", body = HealthCheckResponse),
        (status = 500, description = "Health check failed")
    ),
    tag = "system"
)]
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthCheckResponse>, ApiError> {
    let health = state.core_api_service
        .get_system_health()
        .await?;
    
    Ok(Json(HealthCheckResponse {
        status: "healthy".to_string(),
        brain_readiness: health.brain_readiness,
        burst_engine: health.burst_engine,
        neuron_count: health.neuron_count,
        synapse_count: health.synapse_count,
    }))
}
```

**Deliverables:**
- ✅ Health check endpoint
- ✅ System status endpoint
- ✅ Metrics endpoint

**Timeline:** 2 days

#### 4.3 Cortical Area Endpoints (Week 13)

```rust
// feagi-core/crates/feagi-api/src/endpoints/cortical_area.rs

use axum::{
    extract::{State, Path},
    Json,
};
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct AddCorticalAreaRequest {
    #[validate(length(min = 1, max = 100))]
    pub cortical_id: String,
    
    #[validate(range(min = 0))]
    pub cortical_idx: u32,
    
    pub cortical_name: String,
    
    #[validate(custom = "validate_dimensions")]
    pub dimensions: (u32, u32, u32),
    
    // ... all fields
}

/// Add new cortical area
#[utoipa::path(
    post,
    path = "/v1/cortical_area",
    request_body = AddCorticalAreaRequest,
    responses(
        (status = 200, description = "Cortical area added successfully"),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Cortical area already exists")
    ),
    tag = "cortical_area"
)]
pub async fn add_cortical_area(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AddCorticalAreaRequest>,
) -> Result<Json<()>, ApiError> {
    // Validate request
    request.validate()?;
    
    // Convert to CorticalArea
    let area = CorticalArea {
        cortical_id: request.cortical_id,
        cortical_idx: request.cortical_idx,
        cortical_name: request.cortical_name,
        dimensions: request.dimensions,
        // ... all fields
    };
    
    // Call service
    state.core_api_service
        .add_cortical_area(area)
        .await?;
    
    Ok(Json(()))
}

/// Get cortical area by ID
#[utoipa::path(
    get,
    path = "/v1/cortical_area/{id}",
    params(
        ("id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Cortical area found", body = CorticalArea),
        (status = 404, description = "Cortical area not found")
    ),
    tag = "cortical_area"
)]
pub async fn get_cortical_area(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<CorticalArea>, ApiError> {
    let area = state.core_api_service
        .get_cortical_area(&id)
        .await?;
    
    Ok(Json(area))
}
```

**Deliverables:**
- ✅ All cortical area endpoints (10+ endpoints)
- ✅ Request validation
- ✅ OpenAPI documentation

**Timeline:** 1 week

#### 4.4 Remaining Endpoints (Week 14-15)

**Endpoint groups to migrate:**
1. **Genome endpoints** - Load, save, validate genome
2. **Connectome endpoints** - Query neurons, synapses
3. **Brain endpoints** - Brain state, firing patterns
4. **Agent endpoints** - Register agents, sensor/motor
5. **Network endpoints** - Configuration
6. **Burst engine endpoints** - Control burst engine

**Total endpoints:** ~50-60 endpoints

**Deliverables:**
- ✅ All REST endpoints migrated
- ✅ Request/response validation
- ✅ OpenAPI documentation
- ✅ Error handling

**Timeline:** 2 weeks

**Phase 4 Total: 4 weeks**

---

### Phase 5: Integration & Testing (Weeks 16-18)

**Goal:** End-to-end testing, performance validation, deployment prep

#### 5.1 Integration Tests (Week 16)

```rust
// feagi-core/tests/integration/api_tests.rs

#[tokio::test]
async fn test_full_genome_loading_workflow() {
    // Setup test app
    let app = create_test_app().await;
    
    // 1. Load genome
    let genome_data = load_test_genome();
    let response = app
        .post("/v1/genome/load")
        .json(&genome_data)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    
    // 2. Verify cortical areas created
    let response = app
        .get("/v1/cortical_area")
        .send()
        .await
        .unwrap();
    let areas: Vec<CorticalArea> = response.json().await.unwrap();
    assert!(areas.len() > 0);
    
    // 3. Query neurons
    let area_id = &areas[0].cortical_id;
    let response = app
        .get(&format!("/v1/brain/neurons/{}", area_id))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_agent_sensor_workflow() {
    // Test agent registration and sensor data injection
}

#[tokio::test]
async fn test_burst_engine_integration() {
    // Test burst engine control
}
```

**Test categories:**
- ✅ Full genome loading workflow
- ✅ Agent sensor/motor workflows
- ✅ Burst engine control
- ✅ State synchronization
- ✅ Error handling

**Timeline:** 1 week

#### 5.2 Performance Testing (Week 17)

```rust
// benches/api_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_cortical_area_query(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(create_test_app());
    
    c.bench_function("get_cortical_area", |b| {
        b.iter(|| {
            rt.block_on(async {
                app.get("/v1/cortical_area/test_area")
                    .send()
                    .await
                    .unwrap()
            })
        })
    });
}

fn bench_genome_loading(c: &mut Criterion) {
    // Benchmark full genome load
}

fn bench_neuron_creation(c: &mut Criterion) {
    // Benchmark batch neuron creation
}

criterion_group!(benches, bench_cortical_area_query, bench_genome_loading, bench_neuron_creation);
criterion_main!(benches);
```

**Performance targets:**
- ✅ API latency < 10ms (p99)
- ✅ Genome load < 1s (5-10x faster than Python)
- ✅ Neuron creation 50x faster
- ✅ Memory usage 3x lower

**Timeline:** 3 days

#### 5.3 Migration Testing (Week 17)

**Test strategy:**
- Run Python test suite against Rust API
- Verify all responses match Python
- Check for regressions
- Validate state consistency

**Timeline:** 2 days

#### 5.4 Documentation (Week 18)

- ✅ API documentation (Swagger/OpenAPI)
- ✅ Migration guide
- ✅ Deployment guide
- ✅ Performance tuning guide
- ✅ Troubleshooting guide

**Timeline:** 1 week

**Phase 5 Total: 3 weeks**

---

### Phase 6: Deployment (Week 19-20)

**Goal:** Production deployment

#### 6.1 Build & Optimization

```bash
# Release build with optimizations
cargo build --release

# Strip binary
strip target/release/feagi-core

# Check size
ls -lh target/release/feagi-core

# Expected: 20-50MB (vs 100-200MB Python + deps)
```

#### 6.2 Deployment Configurations

**Docker:**
```dockerfile
# Multi-stage build for minimal image
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/feagi-core /usr/local/bin/
CMD ["feagi-core"]
```

**Kubernetes:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: feagi-core
spec:
  replicas: 3
  selector:
    matchLabels:
      app: feagi-core
  template:
    metadata:
      labels:
        app: feagi-core
    spec:
      containers:
      - name: feagi-core
        image: feagi/feagi-core:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
```

#### 6.3 Monitoring & Observability

```rust
// Add tracing/metrics
use tracing::{info, warn, error};
use prometheus::{Registry, Counter, Histogram};

// In API handlers
#[instrument]
pub async fn add_cortical_area(...) -> Result<...> {
    info!("Adding cortical area: {}", area.cortical_id);
    // ... implementation
}
```

**Timeline:** 2 weeks

---

## Complete Rust Crate Structure (Modular Subcrates)

```
feagi-core/
├── Cargo.toml                       # Workspace definition
├── src/
│   └── main.rs                      # Binary that composes all subcrates
│
├── crates/                          # 7 SUBCRATES
│   │
│   ├── feagi-api/                   # REST API (Axum) - Full Stack Only
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs               # Axum app setup
│   │       ├── middleware/
│   │       │   ├── auth.rs
│   │       │   ├── cors.rs
│   │       │   └── error_handler.rs
│   │       ├── endpoints/
│   │       │   ├── mod.rs
│   │       │   ├── system.rs         # /v1/health, /v1/system/*
│   │       │   ├── cortical_area.rs  # /v1/cortical_area/*
│   │       │   ├── genome.rs         # /v1/genome/*
│   │       │   ├── connectome.rs     # /v1/connectome/*
│   │       │   ├── brain.rs          # /v1/brain/*
│   │       │   ├── agent.rs          # /v1/agent/*
│   │       │   ├── network.rs        # /v1/network/*
│   │       │   └── burst_engine.rs   # /v1/burst_engine/*
│   │       ├── models/
│   │       │   ├── requests.rs       # Request DTOs
│   │       │   └── responses.rs      # Response DTOs
│   │       └── error.rs
│   │
│   ├── feagi-services/              # Service Layer - Full Stack Only
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── base_service.rs       # BaseService trait
│   │       ├── core_api_service.rs   # Facade
│   │       ├── system_service.rs
│   │       ├── genome_service.rs
│   │       ├── cortical_area_service.rs
│   │       ├── connectome_service.rs
│   │       ├── brain_service.rs
│   │       ├── agents_service.rs
│   │       ├── network_service.rs
│   │       ├── npu_service.rs
│   │       └── error.rs
│   │
│   ├── feagi-bdu/                   # Business Logic - CORE (Reusable)
│   │   ├── Cargo.toml               # Features: std, minimal, full, wasm
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connectome_manager.rs
│   │       ├── embryogenesis/
│   │       │   ├── mod.rs
│   │       │   └── neuroembryogenesis.rs
│   │       ├── models/
│   │       │   ├── cortical_area.rs
│   │       │   ├── brain_region.rs
│   │       │   ├── brain_region_hierarchy.rs
│   │       │   └── neuron.rs
│   │       ├── cortical_mapping.rs
│   │       ├── utils/
│   │       │   ├── metrics.rs
│   │       │   ├── mapping_utils.rs
│   │       │   └── position.rs
│   │       └── error.rs
│   │
│   ├── feagi-npu/                   # Burst Engine - CORE (Reusable)
│   │   ├── Cargo.toml               # Features: std, no_std, gpu, wasm
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── burst_engine.rs
│   │       ├── neuron_pool.rs
│   │       └── synapse_manager.rs
│   │
│   ├── feagi-state/                 # State Manager - CORE (Reusable)
│   │   ├── Cargo.toml               # Features: std, no_std
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state_manager.rs
│   │       └── atomic_state.rs
│   │
│   ├── feagi-pns/                   # I/O Streams - Full Stack Only
│   │   ├── Cargo.toml               # ZMQ, not WASM compatible
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── zmq_streams.rs
│   │       └── sensory_injection.rs
│   │
│   └── feagi-config/                # Config Loader - CORE (Reusable)
│       ├── Cargo.toml               # Features: std, no_std
│       └── src/
│           ├── lib.rs
│           └── toml_loader.rs
│
├── tests/
│   ├── integration/
│   │   ├── api_tests.rs
│   │   ├── service_tests.rs
│   │   └── bdu_tests.rs
│   └── common/
│       └── test_helpers.rs
│
└── benches/
    ├── api_benchmarks.rs
    ├── service_benchmarks.rs
    └── bdu_benchmarks.rs
```

**Key Points:**
- **7 subcrates** enable modularity and selective dependencies
- **Full Stack subcrates (3):** api, services, pns → Server only
- **Core subcrates (4):** bdu, npu, state, config → Reusable by inference-engine, web, py
- Each subcrate has its own `Cargo.toml` with feature flags
- Main binary in `src/main.rs` uses all subcrates

---

## Migration Timeline Summary

| Phase | Duration | Focus | Deliverable |
|-------|----------|-------|-------------|
| **Phase 0** | 1 week | Preparation | Clean codebase, analysis |
| **Phase 1** | 2 weeks | Data Structures | Core BDU models |
| **Phase 2** | 4 weeks | Business Logic | ConnectomeManager (89 methods) |
| **Phase 3** | 4 weeks | Service Layer | 8 domain services |
| **Phase 4** | 4 weeks | API Layer | 50-60 REST endpoints |
| **Phase 5** | 3 weeks | Testing | Integration, performance, docs |
| **Phase 6** | 2 weeks | Deployment | Production ready |
| **TOTAL** | **20 weeks** | **5 months** | **Full Rust stack** |

---

## Success Criteria

### Functional Requirements
- ✅ All Python API endpoints work in Rust
- ✅ All Python tests pass against Rust API
- ✅ Zero functional regressions
- ✅ Same service architecture preserved

### Performance Requirements
- ✅ API latency < 10ms (p99)
- ✅ Genome load 5-10x faster
- ✅ Neuron creation 50x faster
- ✅ Memory usage 3x lower
- ✅ Binary size < 50MB

### Quality Requirements
- ✅ 80%+ code coverage
- ✅ Zero unsafe code (except necessary)
- ✅ All clippy warnings fixed
- ✅ OpenAPI documentation complete
- ✅ RTOS compatible (`no_std` where possible)

---

## Risk Mitigation

### High-Risk Areas

| Area | Risk | Mitigation |
|------|------|------------|
| Genome Loading | Complex Python logic | Extensive testing, keep Python test suite |
| State Sync | Race conditions | Use RwLock correctly, integration tests |
| API Compatibility | Breaking changes | API contract tests, response validation |
| Performance | Regression | Benchmark against Python, profile |

### Rollback Strategy

**Feature flags for gradual rollout:**
```rust
#[cfg(feature = "rust_api")]
fn main() {
    // Start Rust API
}

#[cfg(not(feature = "rust_api"))]
fn main() {
    // Start Python API (fallback)
}
```

---

## Python Elimination Checklist

### What Gets Deleted After Migration

- ✅ `feagi/api/` (entire API layer)
- ✅ `feagi/api/core/services/` (entire service layer)
- ✅ `feagi/bdu/` (entire BDU layer)
- ✅ FastAPI dependencies
- ✅ Pydantic models
- ✅ Python test files (kept as reference)

### What Stays in Python (Temporarily)

- ⏸️ BDU (Brain Development Unit) - until Rust impl complete
- ⏸️ Genome validation - complex rules
- ⏸️ Training/Evolution - research code
- ⏸️ Notebooks - analysis/visualization

**Timeline:** These can be migrated in Phase 2 (6+ months later)

---

## Conclusion

**This is a comprehensive, clean-sweep migration plan:**

✅ **No intermediate states** - Direct Python → Rust  
✅ **No throwaway code** - No PyO3 FFI bridges  
✅ **Same architecture** - 3-tier design preserved  
✅ **Full performance** - Native Rust across all layers  
✅ **RTOS ready** - Embedded-compatible from day 1  
✅ **Type safe** - End-to-end type checking  
✅ **Maintainable** - Single language, clear structure  

**Timeline: 20 weeks (5 months)**

**Next steps:**
1. Approve this plan
2. Start Phase 0 (preparation) immediately
3. Begin Phase 1 (data structures) next week
4. Coordinate team resources for 5-month effort

**This is an ambitious but achievable plan that will transform FEAGI into a modern, high-performance, Rust-based architecture!**

