// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
# FEAGI Service Layer

The stable application boundary for FEAGI - defines transport-agnostic
service interfaces that can be used by any adapter (REST API, ZMQ, embedded).

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                    TRANSPORT ADAPTERS                            │
│  Axum/REST, ZMQ Control, Embedded I2C, etc.                     │
└────────────────────────────┬────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────────┐
│              SERVICE LAYER (This Crate)                          │
│  • NeuronService      - Neuron CRUD operations                  │
│  • GenomeService      - Genome load/save                        │
│  • ConnectomeService  - Cortical area & brain region management │
│  • AnalyticsService   - Statistics & system health              │
└────────────────────────────┬────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────────┐
│                   DOMAIN LAYER                                   │
│  feagi-bdu, feagi-evo, feagi-burst-engine, feagi-types          │
└─────────────────────────────────────────────────────────────────┘
```

## Design Principles

1. **Transport-Agnostic**: Services know nothing about HTTP, ZMQ, or I2C
2. **Stable Contracts**: Trait interfaces don't change when backend changes
3. **Async by Default**: All services are async (can be compiled out for embedded)
4. **Error Translation**: Backend errors are translated to transport-agnostic `ServiceError`
5. **DTO-Based**: All parameters and returns use transport-agnostic DTOs

## Usage

### For Adapter Implementers

Adapters depend on service traits, not implementations:

```rust
use feagi_services::{NeuronService, ServiceResult, CreateNeuronParams};

async fn handle_http_request(
    service: &dyn NeuronService,
    req: HttpRequest
) -> HttpResponse {
    // 1. Parse HTTP request to DTO
    let params = CreateNeuronParams { ... };
    
    // 2. Call service (transport-agnostic)
    let result = service.create_neuron(params).await?;
    
    // 3. Convert DTO to HTTP response
    HttpResponse::ok(result)
}
```

### For Backend Implementers

Implementations use domain logic (BDU, NPU, Evo):

```rust
use feagi_services::{NeuronService, ServiceResult};
use feagi_bdu::ConnectomeManager;

struct NeuronServiceImpl {
    connectome: Arc<ConnectomeManager>,
}

#[async_trait]
impl NeuronService for NeuronServiceImpl {
    async fn create_neuron(&self, params: CreateNeuronParams) -> ServiceResult<NeuronInfo> {
        // Delegate to domain logic
        self.connectome.create_neuron(...)?;
        Ok(NeuronInfo { ... })
    }
}
```

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod impls;
pub mod traits;
pub mod types;
pub mod genome;

// Re-export main API
pub use traits::{
    AnalyticsService, ConnectomeService, GenomeService, NeuronService, RuntimeService,
    SnapshotService, SnapshotMetadata, SnapshotCreateOptions,
};

pub use types::{
    // DTOs
    BrainRegionInfo, ConnectivityStats, CorticalAreaInfo, CorticalAreaStats,
    CreateBrainRegionParams, CreateCorticalAreaParams, UpdateCorticalAreaParams,
    CreateNeuronParams, CreateSynapseParams, GenomeInfo, LoadGenomeParams, NeuronInfo,
    SaveGenomeParams, SynapseInfo, SystemHealth, RuntimeStatus,
    // Registration DTOs
    registration::{AreaStatus, CorticalAreaAvailability, CorticalAreaStatus, RegistrationRequest, RegistrationResponse, TransportConfig},
    // Agent registry types
    agent_registry::{AgentType, AgentInfo, AgentCapabilities, AgentTransport, VisionCapability, MotorCapability, VisualizationCapability, SensoryCapability, AgentRegistry},
    // Errors
    ServiceError, ServiceResult,
};

// Re-export implementations (optional - adapters can use their own)
pub use impls::{
    AnalyticsServiceImpl, ConnectomeServiceImpl, GenomeServiceImpl, NeuronServiceImpl,
    RuntimeServiceImpl, SnapshotServiceImpl,
};

