# FEAGI REST API Design Architecture

**Date:** 2025-10-29  
**Status:** Design Phase  
**Version:** 1.0

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Core Design Principles](#2-core-design-principles)
3. [Architecture Overview](#3-architecture-overview)
4. [Multi-Version Strategy](#4-multi-version-strategy)
5. [Unified Endpoint Layer](#5-unified-endpoint-layer)
6. [Transport Adapters](#6-transport-adapters)
7. [Security Architecture](#7-security-architecture)
8. [API Specification](#8-api-specification)
9. [Implementation Plan](#9-implementation-plan)
10. [Migration Path](#10-migration-path)

---

## 1. Executive Summary

### 1.1 Design Goals

**Primary Objectives:**
- ✅ **Transport-Agnostic**: Single endpoint layer serves HTTP REST and ZMQ
- ✅ **Version-Safe**: Multi-version support (v1, v2) without breaking clients
- ✅ **Security-Ready**: Stub architecture for authentication/authorization/encryption
- ✅ **License-Compatible**: Pure Rust, Apache-2.0/MIT dependencies only
- ✅ **Service-Oriented**: Clean separation between API, service, and business logic layers
- ✅ **100% API Compatibility**: Exact request/response structure matching with Python FEAGI
- ✅ **OpenAPI 3.0**: Full utoipa integration with compile-time validation
- ✅ **Custom Swagger UI**: Preserved HTML/CSS styling from Python implementation

**Non-Goals (Out of Scope):**
- ❌ Implementing actual security (stubs only)
- ❌ GraphQL or gRPC (REST and ZMQ only)
- ❌ WebSocket API (handled by existing Brain Visualizer protocol)

### 1.2 Addressing Key Requirements

**Requirement 1: OpenAPI Implementation (utoipa)**
- ✅ **Full OpenAPI 3.0 support** - Same spec version as Python FastAPI
- ✅ **Compile-time validation** - API spec guaranteed to match code
- ✅ **Zero runtime overhead** - All code generation at compile time
- ✅ **Auto-sync** - No manual spec maintenance required
- ✅ **See Section 8.4** for detailed utoipa implementation

**Requirement 2: 100% API Compatibility**
- ✅ **Contract testing** - Automated tests verify Rust matches Python exactly
- ✅ **Snapshot testing** - Python API responses captured as test fixtures
- ✅ **Field-by-field mapping** - Every field name, type, and structure preserved
- ✅ **Type mapping rules** - Python types → Rust types documented
- ✅ **BV compatibility** - Brain Visualizer will work unchanged
- ✅ **See Section 8.5** for compatibility strategy

**Requirement 3: Custom Swagger UI Styling**
- ✅ **CSS/JS preservation** - Existing custom styles migrated to Rust
- ✅ **HTML override support** - Full control over Swagger UI presentation
- ✅ **Static asset embedding** - CSS/JS compiled into binary
- ✅ **utoipa-swagger-ui** - Supports all customization features
- ✅ **See Section 8.6** for Swagger UI customization

### 1.3 Key Innovations

1. **Unified Endpoint Layer** - Write endpoints once, use with HTTP and ZMQ
2. **Version Routing** - `/api/v1/*` and `/api/v2/*` coexist peacefully
3. **License-Safe Crypto** - Application-level encryption (ChaCha20-Poly1305) instead of CurveZMQ
4. **Hexagonal Architecture** - Service layer is the stable boundary

---

## 2. Core Design Principles

### 2.1 Separation of Concerns

```
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Transport Parsing (HTTP/ZMQ-specific)        │
│  - Parse HTTP requests / ZMQ messages                   │
│  - Handle transport-specific auth headers              │
│  - Convert to generic ApiRequest                        │
└──────────────────┬──────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 2: Unified Endpoints (Transport-agnostic)       │
│  - Business logic & validation                          │
│  - Call service layer                                   │
│  - Return generic ApiResponse                           │
└──────────────────┬──────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Service Layer (Already implemented!)         │
│  - NeuronService, GenomeService, etc.                   │
│  - Transport-agnostic business logic                    │
└──────────────────┬──────────────────────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Domain Logic (BDU, NPU, State)               │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Zero Duplication

**Anti-Pattern (What We're Avoiding):**
```rust
// ❌ BAD: Duplicated endpoint logic
mod http {
    fn get_cortical_area(...) {
        // 100 lines of business logic
    }
}

mod zmq {
    fn get_cortical_area(...) {
        // 100 lines of SAME logic (duplication!)
    }
}
```

**Pattern (What We're Implementing):**
```rust
// ✅ GOOD: Shared endpoint logic
mod endpoints {
    fn get_cortical_area(...) {
        // 100 lines of business logic (ONCE!)
    }
}

mod http {
    fn handler(...) { endpoints::get_cortical_area(...) } // 1 line
}

mod zmq {
    fn handler(...) { endpoints::get_cortical_area(...) } // 1 line
}
```

### 2.3 Backward Compatibility

**Promise to Users:**
- ✅ Existing v1 clients continue working indefinitely
- ✅ Deprecation warnings give 12+ months notice
- ✅ New features in v2 don't break v1

---

## 3. Architecture Overview

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    External Clients                         │
│  HTTP (curl, web UI) │ ZMQ (Python agents, services)        │
└────────────┬─────────────────────┬──────────────────────────┘
             │                     │
      ┌──────▼────────┐     ┌──────▼───────────┐
      │ TLS (rustls)  │     │ App-Level Crypto │
      │ (OPTIONAL)    │     │ ChaCha20-Poly1305│
      │               │     │ (OPTIONAL)       │
      └──────┬────────┘     └──────┬───────────┘
             │                     │
      ┌──────▼────────┐     ┌──────▼───────────┐
      │ HTTP Adapter  │     │  ZMQ Adapter     │
      │  (Axum)       │     │  (tmq/raw)       │
      └──────┬────────┘     └──────┬───────────┘
             │                     │
             │ Parse to ApiRequest │
             ├─────────────────────┤
             │                     │
      ┌──────▼─────────────────────▼──────────┐
      │    Authentication Middleware          │  ◄── STUB
      │   (JWT, API Key - future)             │
      └──────┬────────────────────────────────┘
             │
      ┌──────▼────────────────────────────────┐
      │    Authorization Middleware           │  ◄── STUB
      │   (RBAC - future)                     │
      └──────┬────────────────────────────────┘
             │
      ┌──────▼────────────────────────────────┐
      │    Version Router                     │
      │   /api/v1/* → v1 endpoints            │
      │   /api/v2/* → v2 endpoints            │
      └──────┬────────────────────────────────┘
             │
      ┌──────▼────────────────────────────────┐
      │    Unified Endpoint Layer             │
      │   • Transport-agnostic                │
      │   • Business logic here               │
      │   • Call service layer                │
      └──────┬────────────────────────────────┘
             │
      ┌──────▼────────────────────────────────┐
      │    Service Layer                      │
      │   • NeuronService                     │
      │   • GenomeService                     │
      │   • ConnectomeService                 │
      │   • AnalyticsService                  │
      │   (Already implemented!)              │
      └───────────────────────────────────────┘
```

### 3.2 Directory Structure

```
feagi-core/crates/feagi-api/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── server.rs                 # Multi-version router
│   │
│   ├── endpoints/                # ◄── Unified (transport-agnostic)
│   │   ├── mod.rs
│   │   ├── health.rs
│   │   ├── neurons.rs
│   │   ├── cortical_areas.rs
│   │   ├── brain_regions.rs
│   │   ├── genome.rs
│   │   └── analytics.rs
│   │
│   ├── v1/                       # Version 1 DTOs & mappings
│   │   ├── mod.rs
│   │   ├── dtos.rs               # V1-specific request/response types
│   │   └── mapping.rs            # Service DTO ↔ V1 DTO
│   │
│   ├── v2/                       # Version 2 (future)
│   │   └── mod.rs                # Placeholder
│   │
│   ├── transports/               # Transport adapters
│   │   ├── mod.rs
│   │   ├── http/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs         # Axum server
│   │   │   ├── router.rs         # HTTP → Endpoint mapping
│   │   │   └── responses.rs      # Endpoint result → HTTP
│   │   │
│   │   └── zmq/
│   │       ├── mod.rs
│   │       ├── server.rs         # ZMQ ROUTER/DEALER
│   │       ├── router.rs         # ZMQ message → Endpoint
│   │       └── responses.rs      # Endpoint result → ZMQ
│   │
│   ├── security/                 # ◄── STUBS for future
│   │   ├── mod.rs
│   │   ├── auth/
│   │   │   ├── mod.rs            # Authentication (stub)
│   │   │   ├── context.rs        # AuthContext type
│   │   │   └── anonymous.rs      # Anonymous auth (default)
│   │   │
│   │   ├── authz/
│   │   │   ├── mod.rs            # Authorization (stub)
│   │   │   └── permissions.rs    # Permission types
│   │   │
│   │   └── encryption/
│   │       ├── mod.rs
│   │       └── message.rs        # ChaCha20-Poly1305 (stub)
│   │
│   ├── common/
│   │   ├── mod.rs
│   │   ├── request.rs            # Generic ApiRequest
│   │   ├── response.rs           # Generic ApiResponse
│   │   └── error.rs              # ApiError types
│   │
│   └── middleware/
│       ├── mod.rs
│       ├── cors.rs               # CORS (implemented)
│       ├── logging.rs            # Request logging (implemented)
│       └── auth.rs               # Auth middleware (stub)
```

---

## 4. Multi-Version Strategy

### 4.1 URL-Based Versioning

**Routing:**
```
/api/v1/*  → Version 1 endpoints (current, stable)
/api/v2/*  → Version 2 endpoints (future)
/api/*     → Default to latest stable (v1 for now)
/health    → Version-agnostic health check
/ready     → Version-agnostic readiness check
```

### 4.2 Version Router

```rust
// src/server.rs

pub fn create_app(state: ApiState) -> Router {
    Router::new()
        // Version-agnostic (always available)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // V1 API (stable)
        .nest("/api/v1", v1::create_v1_router())
        
        // V2 API (future, stub for now)
        .nest("/api/v2", v2::create_v2_router())
        
        // Default /api/* routes to latest stable (v1)
        .nest("/api", v1::create_v1_router())
        
        // OpenAPI docs (multi-version)
        .route("/docs/v1", get(swagger_ui_v1))
        .route("/docs/v2", get(swagger_ui_v2))
        
        // Middleware
        .layer(middleware::cors_layer())
        .layer(middleware::logging_layer())
        .layer(middleware::auth_layer())  // ◄── STUB
        
        .with_state(state)
}
```

### 4.3 Deprecation Strategy

**Phase 1: Introduce V2**
```rust
// Add deprecation header to v1 responses
response.headers_mut().insert(
    "X-API-Deprecation",
    "This endpoint is deprecated. Use /api/v2/... instead."
);
response.headers_mut().insert(
    "X-API-Sunset",
    "2027-12-31"  // 12+ months notice
);
```

**Phase 2: Sunset V1**
```rust
// After sunset date
if is_past_sunset_date() {
    return Err(ApiError::Gone {
        message: "API v1 has been sunset. Please use v2.".into(),
        replacement: "/api/v2/cortical-areas".into(),
    });
}
```

### 4.4 Version-Specific DTOs

**Purpose:** Allow breaking changes between versions

```rust
// v1/dtos.rs
#[derive(Serialize)]
pub struct CorticalAreaV1 {
    pub cortical_id: String,
    pub dimensions: (usize, usize, usize),  // Tuple (v1 format)
}

// v2/dtos.rs (future)
#[derive(Serialize)]
pub struct CorticalAreaV2 {
    pub cortical_id: String,
    pub dimensions: DimensionsV2,  // Object (breaking change)
}
```

**Conversion:**
```rust
// Both versions call the same service layer
let service_result = service.get_cortical_area(id).await?;

// Convert to version-specific DTO
let v1_dto = CorticalAreaV1::from_service(service_result);
let v2_dto = CorticalAreaV2::from_service(service_result);
```

---

## 5. Unified Endpoint Layer

### 5.1 Core Concept

**Endpoints are pure functions** that:
- Accept generic `ApiRequest` (transport-agnostic)
- Return `Result<T, ApiError>` (transport-agnostic)
- Contain ALL business logic (validation, service calls)
- Are called by BOTH HTTP and ZMQ adapters

### 5.2 Example Endpoint

```rust
// src/endpoints/cortical_areas.rs

use crate::common::{ApiRequest, ApiError};
use crate::v1::dtos::{CorticalAreaV1, CreateCorticalAreaRequest};
use crate::security::AuthContext;
use feagi_services::ConnectomeService;
use std::sync::Arc;

/// Get cortical area by ID (transport-agnostic)
pub async fn get_cortical_area(
    cortical_id: String,
    auth_ctx: &AuthContext,  // ◄── Stub (always anonymous for now)
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
) -> Result<CorticalAreaV1, ApiError> {
    // Future: Authorization check
    // auth_ctx.require_permission(Permission::CorticalAreaRead)?;
    
    // Call service layer
    let service_result = connectome_service
        .get_cortical_area(&cortical_id)
        .await
        .map_err(ApiError::from_service_error)?;
    
    // Convert service DTO → API DTO
    let api_dto = CorticalAreaV1::from_service(service_result);
    
    Ok(api_dto)
}

/// Create cortical area
pub async fn create_cortical_area(
    request: CreateCorticalAreaRequest,
    auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
) -> Result<CorticalAreaV1, ApiError> {
    // Validate request
    request.validate()?;
    
    // Future: Authorization check
    // auth_ctx.require_permission(Permission::CorticalAreaCreate)?;
    
    // Convert API DTO → Service DTO
    let service_params = request.to_service_params();
    
    // Call service
    let service_result = connectome_service
        .add_cortical_area(service_params)
        .await
        .map_err(ApiError::from_service_error)?;
    
    Ok(CorticalAreaV1::from_service(service_result))
}

/// List all cortical areas
pub async fn list_cortical_areas(
    auth_ctx: &AuthContext,
    connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
) -> Result<Vec<CorticalAreaV1>, ApiError> {
    let service_results = connectome_service
        .list_cortical_areas()
        .await
        .map_err(ApiError::from_service_error)?;
    
    let api_dtos = service_results
        .into_iter()
        .map(CorticalAreaV1::from_service)
        .collect();
    
    Ok(api_dtos)
}
```

**Key Points:**
- ✅ No HTTP or ZMQ code here
- ✅ All business logic in one place
- ✅ Security hooks (stubbed)
- ✅ Clean service layer calls

---

## 6. Transport Adapters

### 6.1 HTTP Adapter (Axum)

```rust
// src/transports/http/router.rs

use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use crate::endpoints;
use crate::common::{ApiState, ApiResponse};
use crate::security::AuthContext;

pub fn create_http_router_v1(state: ApiState) -> Router {
    Router::new()
        .route("/cortical-areas/:id", get(http_get_cortical_area))
        .route("/cortical-areas", get(http_list_cortical_areas)
                                  .post(http_create_cortical_area))
        .route("/cortical-areas/:id", delete(http_delete_cortical_area))
        // ... more routes
        .with_state(state)
}

/// HTTP adapter - thin wrapper around unified endpoint
async fn http_get_cortical_area(
    Path(cortical_id): Path<String>,
    State(state): State<ApiState>,
    // Future: Extract(auth_ctx): Extract<AuthContext>
) -> Result<Json<ApiResponse<CorticalAreaV1>>, HttpError> {
    // For now, use anonymous auth context
    let auth_ctx = AuthContext::anonymous();
    
    // Call unified endpoint (SAME AS ZMQ!)
    let result = endpoints::get_cortical_area(
        cortical_id,
        &auth_ctx,
        state.connectome_service.clone()
    ).await?;
    
    // Wrap in HTTP response format
    Ok(Json(ApiResponse::success(result)))
}

async fn http_create_cortical_area(
    State(state): State<ApiState>,
    Json(request): Json<CreateCorticalAreaRequest>,
) -> Result<Json<ApiResponse<CorticalAreaV1>>, HttpError> {
    let auth_ctx = AuthContext::anonymous();
    
    let result = endpoints::create_cortical_area(
        request,
        &auth_ctx,
        state.connectome_service.clone()
    ).await?;
    
    Ok(Json(ApiResponse::success(result)))
}
```

### 6.2 ZMQ Adapter

```rust
// src/transports/zmq/router.rs

use crate::endpoints;
use crate::common::{ZmqRequest, ZmqResponse, ApiState};
use crate::security::AuthContext;

pub async fn route_zmq_request(
    request: ZmqRequest,
    state: &ApiState,
) -> ZmqResponse {
    // For now, use anonymous auth context
    let auth_ctx = AuthContext::anonymous();
    
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", path) if path.starts_with("/api/v1/cortical-areas/") => {
            let cortical_id = extract_id_from_path(path);
            
            // Call unified endpoint (SAME AS HTTP!)
            let result = endpoints::get_cortical_area(
                cortical_id,
                &auth_ctx,
                state.connectome_service.clone()
            ).await;
            
            ZmqResponse::from_result(result)
        }
        
        ("GET", "/api/v1/cortical-areas") => {
            let result = endpoints::list_cortical_areas(
                &auth_ctx,
                state.connectome_service.clone()
            ).await;
            
            ZmqResponse::from_result(result)
        }
        
        ("POST", "/api/v1/cortical-areas") => {
            let request: CreateCorticalAreaRequest = 
                serde_json::from_str(&request.body)?;
            
            let result = endpoints::create_cortical_area(
                request,
                &auth_ctx,
                state.connectome_service.clone()
            ).await;
            
            ZmqResponse::from_result(result)
        }
        
        _ => ZmqResponse::not_found()
    }
}
```

**Key Observation:** Steps 4-7 are **IDENTICAL** between HTTP and ZMQ!

---

## 7. Security Architecture

### 7.1 License-Safe Crypto Stack

**Decision:** Use application-level encryption instead of CurveZMQ (MPL-2.0)

```toml
# All MIT/Apache-2.0 compatible!
[dependencies]
# Encryption
chacha20poly1305 = "0.10"  # Apache-2.0 or MIT
x25519-dalek = "2.0"        # BSD-3-Clause (compatible)
ed25519-dalek = "2.0"       # BSD-3-Clause (for signatures)
rand = "0.8"                # MIT or Apache-2.0

# Authentication
jsonwebtoken = "9.0"        # MIT
argon2 = "0.5"              # Apache-2.0 or MIT

# TLS
rustls = "0.21"             # Apache-2.0 / ISC / MIT
tokio-rustls = "0.24"       # MIT
```

### 7.2 Security Layers (All Stubs for Now)

```rust
// src/security/auth/context.rs

/// Authentication context (stub)
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub principal_id: String,
    pub auth_method: AuthMethod,
    pub roles: Vec<String>,
    pub is_authenticated: bool,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Anonymous,       // ◄── Default for now
    ApiKey,          // Future
    Jwt,             // Future
    MutualTls,       // Future
}

impl AuthContext {
    /// Create anonymous context (default)
    pub fn anonymous() -> Self {
        Self {
            principal_id: "anonymous".to_string(),
            auth_method: AuthMethod::Anonymous,
            roles: vec!["viewer".to_string()],
            is_authenticated: false,
        }
    }
    
    /// Future: Check if user has role
    pub fn has_role(&self, _role: &str) -> bool {
        true  // Stub: always allow
    }
    
    /// Future: Require authentication
    pub fn require_auth(&self) -> Result<(), AuthError> {
        Ok(())  // Stub: always allow
    }
}
```

### 7.3 Authorization Stubs

```rust
// src/security/authz/permissions.rs

/// Future permissions (stub)
#[derive(Debug, Clone)]
pub enum Permission {
    NeuronRead,
    NeuronCreate,
    CorticalAreaRead,
    CorticalAreaCreate,
    GenomeLoad,
    SystemAdmin,
    // ... more
}

/// Future authorizer (stub)
pub struct Authorizer;

impl Authorizer {
    pub fn authorize(_ctx: &AuthContext, _perm: Permission) -> Result<(), AuthzError> {
        Ok(())  // Stub: always allow
    }
}
```

### 7.4 Encryption Stubs

```rust
// src/security/encryption/message.rs

/// Application-level message encryption (stub)
pub struct MessageEncryptor {
    secret_key: [u8; 32],
    public_key: [u8; 32],
}

impl MessageEncryptor {
    pub fn new() -> Self {
        // Stub: Generate keys but don't use them yet
        todo!("Implement ChaCha20-Poly1305 encryption")
    }
    
    pub fn encrypt(&self, _plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Stub: Return plaintext for now
        todo!("Implement encryption")
    }
    
    pub fn decrypt(&self, _ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Stub: Return ciphertext for now
        todo!("Implement decryption")
    }
}
```

### 7.5 Security Configuration

```toml
# feagi_configuration.toml

[api.security]
# Authentication (stub - disabled for now)
auth_enabled = false
auth_required = false

# Authorization (stub - disabled for now)
rbac_enabled = false
default_role = "admin"  # Everyone is admin for now

# Encryption (stub - disabled for now)
[api.security.tls]
enabled = false

[api.security.zmq]
encryption_enabled = false
```

---

## 8. API Specification

### 8.1 Endpoint Catalog

**Base URL:** `/api/v1`

#### System Endpoints
```
GET  /health                     → SystemHealth
GET  /ready                      → ReadinessCheck
GET  /analytics/system           → Detailed system stats
```

#### Neuron Endpoints
```
POST   /neurons                  → Create neuron
DELETE /neurons/{id}             → Delete neuron
GET    /neurons/{id}             → Get neuron info
GET    /cortical-areas/{id}/neurons?limit=100  → List neurons
```

#### Cortical Area Endpoints
```
POST   /cortical-areas           → Create cortical area
DELETE /cortical-areas/{id}      → Delete cortical area
GET    /cortical-areas/{id}      → Get cortical area info
GET    /cortical-areas           → List all areas
GET    /cortical-areas/{id}/stats → Get area statistics
```

#### Brain Region Endpoints
```
POST   /brain-regions            → Create brain region
DELETE /brain-regions/{id}       → Delete brain region
GET    /brain-regions/{id}       → Get brain region info
GET    /brain-regions            → List all regions
GET    /brain-regions/hierarchy  → Get hierarchy tree
```

#### Genome Endpoints
```
POST   /genome/load              → Load genome from JSON
POST   /genome/save              → Save genome to JSON
GET    /genome/info              → Get genome metadata
POST   /genome/validate          → Validate genome
POST   /genome/reset             → Reset connectome
```

#### Analytics Endpoints
```
GET    /analytics/neurons/count         → Total neuron count
GET    /analytics/synapses/count        → Total synapse count
GET    /analytics/areas/populated       → List populated areas
GET    /analytics/areas                 → All area stats
GET    /analytics/connectivity?source={}&target={}  → Stats
```

### 8.2 Response Format

**Success Response:**
```json
{
  "success": true,
  "data": { /* ... */ },
  "timestamp": "2025-10-29T12:34:56Z"
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Cortical area 'vis_l0' not found",
    "details": { "resource": "CorticalArea", "id": "vis_l0" }
  },
  "timestamp": "2025-10-29T12:34:56Z"
}
```

### 8.3 HTTP Status Codes

```
200 OK               → Success
201 Created          → Resource created
204 No Content       → Delete success
400 Bad Request      → InvalidInput
401 Unauthorized     → Authentication required (future)
403 Forbidden        → Authorization failed (future)
404 Not Found        → NotFound
409 Conflict         → AlreadyExists
410 Gone             → API version sunset
500 Internal Error   → Internal
501 Not Implemented  → NotImplemented
```

### 8.4 OpenAPI Documentation (utoipa)

**Why utoipa?**
- ✅ **Compile-time validation** - OpenAPI spec is guaranteed to match actual code
- ✅ **Zero runtime overhead** - All code generation at compile time
- ✅ **Full OpenAPI 3.0 support** - Same as Python FastAPI
- ✅ **Custom Swagger UI** - Supports HTML/CSS customization
- ✅ **Auto-sync** - No manual spec maintenance

**Example Annotation:**
```rust
// src/transports/http/endpoints/cortical_areas.rs

#[utoipa::path(
    get,
    path = "/api/v1/cortical-areas/{cortical_id}",
    params(
        ("cortical_id" = String, Path, description = "Cortical area ID")
    ),
    responses(
        (status = 200, description = "Cortical area found", body = CorticalAreaV1),
        (status = 404, description = "Cortical area not found", body = ApiError)
    ),
    tag = "cortical_areas",
    security(
        (),  // Anonymous for now
        ("api_key" = [])  // Future
    )
)]
pub async fn get_cortical_area(...) { /* ... */ }
```

**OpenAPI Schema Definition:**
```rust
// src/lib.rs

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FEAGI REST API",
        version = "1.0.0",
        description = "FEAGI (Framework for Evolutionary Artificial General Intelligence) REST API",
        contact(
            name = "FEAGI Team",
            url = "https://feagi.org",
            email = "info@feagi.org"
        ),
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    ),
    paths(
        // System
        crate::transports::http::endpoints::health::health_check,
        crate::transports::http::endpoints::health::readiness_check,
        
        // Cortical Areas
        crate::transports::http::endpoints::cortical_areas::get_cortical_area,
        crate::transports::http::endpoints::cortical_areas::list_cortical_areas,
        crate::transports::http::endpoints::cortical_areas::create_cortical_area,
        crate::transports::http::endpoints::cortical_areas::update_cortical_area,
        crate::transports::http::endpoints::cortical_areas::delete_cortical_area,
        
        // ... all other endpoints
    ),
    components(
        schemas(
            // Request/Response DTOs
            CorticalAreaV1,
            CreateCorticalAreaRequest,
            UpdateCorticalAreaRequest,
            BrainRegionV1,
            GenomeInfo,
            SystemHealth,
            ApiResponse<CorticalAreaV1>,
            ApiError,
            // ... all other types
        )
    ),
    tags(
        (name = "system", description = "System health and status endpoints"),
        (name = "cortical_areas", description = "Cortical area management"),
        (name = "brain_regions", description = "Brain region management"),
        (name = "genome", description = "Genome operations"),
        (name = "analytics", description = "Analytics and statistics"),
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development"),
        (url = "https://api.feagi.org", description = "Production")
    )
)]
pub struct ApiDoc;
```

**Access Points:**
- Swagger UI: `/docs/v1` (interactive)
- OpenAPI JSON: `/api-docs/openapi.json` (machine-readable)
- ReDoc: `/redoc` (alternative UI)

---

## 8.5 API Contract Compatibility (Python → Rust)

**CRITICAL REQUIREMENT:** Rust API must be 100% backward compatible with Python API.

### 8.5.1 Compatibility Strategy

**1. Contract Testing**
```rust
// tests/contract_tests.rs

/// Verify Rust response matches Python response exactly
#[tokio::test]
async fn test_health_check_response_format() {
    let rust_response = rust_api_client.get("/v1/health").await;
    let python_response = load_python_response_snapshot("health_check.json");
    
    // Compare JSON structure (ignore dynamic fields like timestamp)
    assert_json_match!(rust_response, python_response, ignore = ["timestamp"]);
}

#[tokio::test]
async fn test_cortical_area_response_format() {
    let rust_response = rust_api_client.get("/v1/cortical-areas/vis_l0").await;
    let python_response = load_python_response_snapshot("cortical_area.json");
    
    assert_json_match!(rust_response, python_response);
}
```

**2. Response Snapshot Testing**
```bash
# Generate snapshots from running Python FEAGI
python scripts/generate_api_snapshots.py \
    --output tests/snapshots/ \
    --endpoints all

# Run contract tests against snapshots
cargo test --test contract_tests
```

**3. Field-by-Field Mapping**

**Example: Health Check Endpoint**

**Python Response (feagi-py):**
```json
{
  "status": "healthy",
  "brain_readiness": true,
  "burst_engine": true,
  "neuron_count": 1000,
  "synapse_count": 5000,
  "cortical_area_count": 10,
  "genome_validity": true,
  "influxdb_availability": false,
  "connectome_path": "/path/to/connectome",
  "genome_timestamp": "2025-10-29T12:34:56",
  "change_state": "modified",
  "changes_saved_externally": false
}
```

**Rust DTO (Must Match Exactly):**
```rust
// src/v1/dtos.rs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResponseV1 {
    /// Overall system status
    pub status: String,
    
    /// Brain is ready to process
    pub brain_readiness: bool,
    
    /// Burst engine is running
    pub burst_engine: bool,
    
    /// Total neuron count
    pub neuron_count: u64,
    
    /// Total synapse count
    pub synapse_count: u64,
    
    /// Number of cortical areas
    pub cortical_area_count: usize,
    
    /// Genome is valid
    pub genome_validity: bool,
    
    /// InfluxDB is available
    pub influxdb_availability: bool,
    
    /// Path to connectome
    pub connectome_path: String,
    
    /// Genome timestamp (ISO 8601)
    pub genome_timestamp: String,
    
    /// Change state
    pub change_state: String,
    
    /// Changes saved externally
    pub changes_saved_externally: bool,
}
```

**Key Rules:**
- ✅ **Exact field names** (snake_case, not camelCase)
- ✅ **Exact field order** (for consistency, not required by JSON)
- ✅ **Exact types** (string, bool, number)
- ✅ **Same optional/required** (use `Option<T>` for optional)
- ✅ **Same defaults** (use `#[serde(default)]`)

**4. Type Mapping Rules**

| Python Type | Rust Type | Notes |
|-------------|-----------|-------|
| `str` | `String` | Always owned |
| `int` | `i64` or `u64` | Match sign and range |
| `float` | `f64` | Always 64-bit |
| `bool` | `bool` | Direct mapping |
| `list` | `Vec<T>` | Generic over element type |
| `dict` | `HashMap<K, V>` or custom struct | Prefer struct for known fields |
| `Optional[T]` | `Option<T>` | Direct mapping |
| `datetime` | `String` (ISO 8601) | Use `chrono` for parsing |
| `None` | `null` | Use `#[serde(skip_serializing_if = "Option::is_none")]` |

**5. Special Cases**

**Tuple Dimensions (Python):**
```python
# Python FastAPI
class CorticalArea(BaseModel):
    dimensions: Tuple[int, int, int]  # (width, height, depth)
```

**Tuple Dimensions (Rust):**
```rust
// Rust - Must use tuple, NOT array!
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CorticalAreaV1 {
    #[schema(example = json!([10, 10, 10]))]
    pub dimensions: (usize, usize, usize),  // ✅ Tuple
    // NOT: pub dimensions: [usize; 3],     // ❌ Array (different JSON)
}
```

**Why?**
- Python tuple `(10, 10, 10)` → JSON `[10, 10, 10]`
- Rust tuple `(10, 10, 10)` → JSON `[10, 10, 10]` ✅
- Rust array `[10; 3]` → JSON `[10, 10, 10]` (same, but semantics differ)

**6. Validation Compatibility**

**Python Validation (Pydantic):**
```python
class CreateCorticalAreaRequest(BaseModel):
    cortical_id: str = Field(..., min_length=1, max_length=100)
    dimensions: Tuple[int, int, int] = Field(..., gt=0)
```

**Rust Validation (validator crate):**
```rust
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateCorticalAreaRequest {
    #[validate(length(min = 1, max = 100))]
    pub cortical_id: String,
    
    #[validate(custom = "validate_dimensions")]
    pub dimensions: (usize, usize, usize),
}

fn validate_dimensions(dims: &(usize, usize, usize)) -> Result<(), ValidationError> {
    if dims.0 == 0 || dims.1 == 0 || dims.2 == 0 {
        return Err(ValidationError::new("dimensions must be greater than 0"));
    }
    Ok(())
}
```

**7. Error Response Compatibility**

**Python Error Format:**
```json
{
  "detail": "Cortical area 'vis_l0' not found"
}
```

**Rust Error Format (Must Match):**
```rust
#[derive(Serialize, ToSchema)]
pub struct ApiError {
    pub detail: String,  // ✅ Same field name as Python
}

// Usage
Err(ApiError { detail: "Cortical area 'vis_l0' not found".to_string() })
```

### 8.5.2 Migration Validation Checklist

**Before deploying Rust API:**

- [ ] All Python endpoints have Rust equivalents
- [ ] Contract tests pass (100% snapshot match)
- [ ] Brain Visualizer connects successfully
- [ ] All existing REST clients work unchanged
- [ ] Error responses match Python format
- [ ] OpenAPI spec validates
- [ ] Performance is same or better

---

## 8.6 Custom Swagger UI Styling

**Requirement:** Preserve Python FEAGI's custom HTML/CSS for Swagger UI.

### 8.6.1 Current Python Customizations

**Python Implementation (feagi-py):**
```python
# feagi/api/rest/app.py

from fastapi.openapi.docs import get_swagger_ui_html

@app.get("/docs", include_in_schema=False)
async def custom_swagger_ui_html():
    return get_swagger_ui_html(
        openapi_url=app.openapi_url,
        title=app.title + " - Swagger UI",
        swagger_css_url="/static/swagger-ui-custom.css",
        swagger_js_url="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js",
    )
```

**Custom CSS (static/swagger-ui-custom.css):**
```css
/* FEAGI branding */
.swagger-ui .topbar {
    background-color: #1a1a2e;
}

.swagger-ui .info .title {
    color: #16c79a;
    font-family: 'Roboto', sans-serif;
}

/* Custom button styling */
.swagger-ui .btn.execute {
    background-color: #16c79a;
    border-color: #16c79a;
}

.swagger-ui .btn.execute:hover {
    background-color: #119a7a;
}

/* ... more custom styles */
```

### 8.6.2 Rust Implementation (utoipa-swagger-ui)

**Step 1: Embed Custom CSS/JS**

```rust
// src/transports/http/swagger.rs

use axum::response::Html;
use utoipa_swagger_ui::SwaggerUi;

// Embed custom CSS at compile time
const CUSTOM_CSS: &str = include_str!("../../../static/swagger-ui-custom.css");
const CUSTOM_JS: &str = include_str!("../../../static/swagger-ui-custom.js");

pub fn create_custom_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/docs/v1")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .config(
            utoipa_swagger_ui::Config::default()
                .try_it_out_enabled(true)
                .filter(true)
                .persist_authorization(true)
        )
}

// Serve custom CSS
pub async fn swagger_custom_css() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    ([(axum::http::header::CONTENT_TYPE, "text/css")], CUSTOM_CSS)
}

// Serve custom JS
pub async fn swagger_custom_js() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], CUSTOM_JS)
}
```

**Step 2: Register Custom Assets**

```rust
// src/server.rs

pub fn create_app(state: ApiState) -> Router {
    Router::new()
        // API routes
        .nest("/api/v1", v1::create_v1_router())
        
        // Custom Swagger UI with injected CSS/JS
        .merge(swagger::create_custom_swagger_ui())
        
        // Serve custom CSS/JS
        .route("/static/swagger-ui-custom.css", get(swagger::swagger_custom_css))
        .route("/static/swagger-ui-custom.js", get(swagger::swagger_custom_js))
        
        .with_state(state)
}
```

**Step 3: HTML Override (Advanced)**

If you need full HTML control:

```rust
// src/transports/http/swagger.rs

pub async fn custom_swagger_html() -> Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>FEAGI API Documentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css" />
    <link rel="stylesheet" href="/static/swagger-ui-custom.css" />
    <link rel="icon" href="/static/favicon.ico" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="/static/swagger-ui-custom.js"></script>
    <script>
        window.onload = function() {{
            window.ui = SwaggerUIBundle({{
                url: "/api-docs/openapi.json",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            }});
        }};
    </script>
</body>
</html>
    "#);
    
    Html(html)
}
```

### 8.6.3 Migration Steps

**1. Extract Python Assets**
```bash
# From feagi-py/
cp feagi/api/static/swagger-ui-custom.css \
   ../feagi-core/crates/feagi-api/static/

cp feagi/api/static/swagger-ui-custom.js \
   ../feagi-core/crates/feagi-api/static/
```

**2. Verify Styling**
```bash
# Start Rust API
cargo run

# Open browser
open http://localhost:8080/docs/v1

# Compare side-by-side with Python
open http://localhost:8080/docs/v1  # Rust
open http://localhost:8000/docs     # Python
```

**3. Adjust CSS for utoipa**

Some selectors may differ:
```css
/* Python FastAPI uses these classes */
.swagger-ui .topbar { /* ... */ }

/* utoipa may use slightly different structure */
.swagger-ui .swagger-ui-wrap .topbar { /* ... */ }
```

Use browser DevTools to inspect and adjust.

### 8.6.4 Directory Structure

```
feagi-api/
├── static/
│   ├── swagger-ui-custom.css    ← Migrated from Python
│   ├── swagger-ui-custom.js     ← Migrated from Python
│   └── favicon.ico
├── src/
│   ├── transports/
│   │   └── http/
│   │       └── swagger.rs       ← Custom Swagger setup
│   └── server.rs
```

---

## 9. Implementation Plan

### 9.1 Phase 1: Infrastructure (Week 1)

**Deliverables:**
- ✅ Crate structure (`feagi-api/`)
- ✅ Common types (`ApiRequest`, `ApiResponse`, `ApiError`)
- ✅ Security stubs (`AuthContext`, `Permission`)
- ✅ Axum setup with basic routing
- ✅ ZMQ setup with basic message handling
- ✅ CORS middleware
- ✅ Logging middleware
- ✅ Python API snapshot generator
- ✅ Contract test infrastructure

**Files Created:**
```
feagi-api/
├── Cargo.toml
├── static/
│   ├── swagger-ui-custom.css    ← Copied from Python
│   └── swagger-ui-custom.js     ← Copied from Python
├── src/
│   ├── lib.rs
│   ├── server.rs
│   ├── common/
│   │   ├── request.rs
│   │   ├── response.rs
│   │   └── error.rs
│   ├── security/
│   │   ├── auth/context.rs
│   │   └── authz/permissions.rs
│   ├── middleware/
│   │   ├── cors.rs
│   │   └── logging.rs
│   └── transports/
│       ├── http/
│       │   ├── server.rs
│       │   └── swagger.rs       ← Custom Swagger UI
│       └── zmq/server.rs
└── tests/
    ├── contract_tests.rs
    └── snapshots/               ← Python API response snapshots
        ├── health_check.json
        └── ...
```

**Python Snapshot Generator:**
```python
# feagi-py/scripts/generate_api_snapshots.py

import json
import httpx
from pathlib import Path

ENDPOINTS = [
    ("GET", "/v1/health"),
    ("GET", "/v1/cortical-areas"),
    ("GET", "/v1/brain-regions"),
    # ... all endpoints
]

def generate_snapshots(output_dir: Path):
    client = httpx.Client(base_url="http://localhost:8080")
    
    for method, path in ENDPOINTS:
        response = client.request(method, path)
        
        snapshot = {
            "request": {"method": method, "path": path},
            "status": response.status_code,
            "body": response.json(),
        }
        
        filename = f"{method}_{path.replace('/', '_')}.json"
        (output_dir / filename).write_text(json.dumps(snapshot, indent=2))
```

### 9.2 Phase 2: Unified Endpoints (Week 2)

**Deliverables:**
- ✅ Health endpoint (working for HTTP + ZMQ)
- ✅ Cortical area endpoints (CRUD)
- ✅ OpenAPI documentation

**Files Created:**
```
feagi-api/src/
├── endpoints/
│   ├── health.rs
│   └── cortical_areas.rs
├── v1/
│   ├── dtos.rs
│   └── mapping.rs
└── transports/
    ├── http/router.rs
    └── zmq/router.rs
```

### 9.3 Phase 3: Complete Endpoint Set (Week 3)

**Deliverables:**
- ✅ All 50-60 endpoints
- ✅ Request validation
- ✅ Error handling
- ✅ Integration tests

**Files Created:**
```
feagi-api/src/endpoints/
├── neurons.rs
├── brain_regions.rs
├── genome.rs
└── analytics.rs
```

### 9.4 Phase 4: Testing & Documentation (Week 4)

**Deliverables:**
- ✅ Integration tests (HTTP + ZMQ)
- ✅ OpenAPI spec verification
- ✅ Migration guide
- ✅ API documentation

---

## 10. Migration Path

### 10.1 Current State (Python)

```
feagi-py/feagi/api/v1/
├── system.py         → SystemAPIRouter (FastAPI)
├── genome.py         → GenomeAPIRouter
├── cortical_area.py  → CorticalAreaAPIRouter
└── ...
```

### 10.2 Target State (Rust)

```
feagi-core/crates/feagi-api/
├── endpoints/        → Unified (HTTP + ZMQ)
├── transports/
│   ├── http/        → Axum
│   └── zmq/         → tmq
└── v1/              → V1 DTOs
```

### 10.3 Coexistence Strategy

**Option 1: Feature Flag (Gradual Rollout)**
```rust
#[cfg(feature = "rust_api")]
fn start_api() {
    // Start Rust API
    axum::Server::bind(&addr).serve(app).await
}

#[cfg(not(feature = "rust_api"))]
fn start_api() {
    // Start Python API (fallback)
    python_api::start()
}
```

**Option 2: Port-Based (Side-by-Side)**
```toml
[api]
python_port = 8080  # Old API
rust_port = 8081    # New API (testing)
```

**Option 3: Clean Sweep (Recommended)**
- Complete Rust API implementation
- Run full test suite against Rust API
- Delete Python API entirely
- Single deployment, no hybrid state

---

## Appendix A: Dependencies

```toml
# feagi-api/Cargo.toml

[dependencies]
# Core FEAGI
feagi-services = { path = "../feagi-services" }
feagi-types = { path = "../feagi-types" }

# HTTP/REST
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
hyper = "1.0"

# ZMQ
tmq = "0.4"  # Pure Rust, MIT license

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }

# OpenAPI (utoipa - compile-time, full OpenAPI 3.0 support)
utoipa = { version = "4.0", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Security (stubs for now, all MIT/Apache-2.0 compatible)
chacha20poly1305 = { version = "0.10", optional = true }  # MIT/Apache-2.0
x25519-dalek = { version = "2.0", optional = true }       # BSD-3-Clause (compatible)
jsonwebtoken = { version = "9.0", optional = true }       # MIT
argon2 = { version = "0.5", optional = true }             # MIT/Apache-2.0

# TLS
rustls = { version = "0.21", optional = true }            # Apache-2.0/ISC/MIT
tokio-rustls = { version = "0.24", optional = true }      # MIT

# Error handling
thiserror = "1.0"

# HTTP client (for testing against Python API)
reqwest = { version = "0.11", features = ["json"] }

[dev-dependencies]
# Contract testing
insta = "1.34"           # Snapshot testing
wiremock = "0.6"         # HTTP mocking
assert-json-diff = "2.0" # JSON comparison
tokio-test = "0.4"       # Async test utilities

[features]
default = []
security = ["chacha20poly1305", "x25519-dalek", "jsonwebtoken", "argon2"]
tls = ["rustls", "tokio-rustls"]
```

### Why utoipa?

**Comparison with Alternatives:**

| Feature | utoipa | paperclip | okapi | Manual OpenAPI |
|---------|--------|-----------|-------|----------------|
| **Compile-time generation** | ✅ | ❌ | ❌ | ❌ |
| **OpenAPI 3.0 support** | ✅ | Partial | ✅ | ✅ |
| **Axum integration** | ✅ | ❌ | ❌ | Manual |
| **Custom Swagger UI** | ✅ | ✅ | ✅ | ✅ |
| **Auto-sync with code** | ✅ | Partial | Partial | ❌ |
| **Type safety** | ✅ | Partial | Partial | ❌ |
| **Active maintenance** | ✅ | ❌ | ⚠️ | N/A |
| **License** | MIT/Apache-2.0 | MIT | Apache-2.0 | N/A |

**Decision:** utoipa provides the best combination of compile-time safety, OpenAPI 3.0 support, and Axum integration. It's actively maintained and matches Python FastAPI's capabilities.

**Python FastAPI Equivalence:**

| Python (FastAPI) | Rust (utoipa) | Feature |
|------------------|---------------|---------|
| `@app.get("/path")` | `#[utoipa::path(get, path = "/path")]` | Route definition |
| `response_model=Model` | `body = Model` | Response schema |
| `Depends()` | `State<AppState>` | Dependency injection |
| Pydantic `BaseModel` | `#[derive(ToSchema)]` struct | Schema generation |
| `Field(description="...")` | `#[schema(description = "...")]` | Field documentation |
| `responses={404: {...}}` | `responses((status = 404, ...))` | Error responses |

---

## Appendix B: Key Design Decisions

| Decision | Rationale | Alternative Considered |
|----------|-----------|------------------------|
| **Unified Endpoint Layer** | Zero duplication, single source of truth | Separate HTTP/ZMQ endpoints (rejected: duplication) |
| **URL Versioning** | Clear, explicit, cache-friendly | Header versioning (rejected: harder for clients) |
| **Application-Level Crypto** | License-safe (MIT/Apache-2.0) | CurveZMQ (rejected: MPL-2.0 incompatible) |
| **Axum over Actix** | Better async, tower ecosystem, production-ready | Actix-web (rejected: more opinionated) |
| **tmq over libzmq** | Pure Rust, MIT license, no C deps | libzmq (rejected: MPL-2.0, C dependency) |
| **Security Stubs** | Ship faster, add security later | Full security now (rejected: scope creep) |
| **Service Layer Boundary** | Clean separation, testability | API calls BDU directly (rejected: coupling) |

---

## Appendix C: Future Enhancements

**Not in initial implementation, but designed for:**

1. **Authentication**
   - JWT validation
   - API key management
   - mTLS client certificates

2. **Authorization**
   - Role-Based Access Control (RBAC)
   - Fine-grained permissions
   - Policy enforcement

3. **Encryption**
   - ChaCha20-Poly1305 message encryption
   - TLS/HTTPS
   - Key rotation

4. **Rate Limiting**
   - Per-user/per-IP limits
   - Burst allowances

5. **Caching**
   - Response caching
   - ETag support

6. **GraphQL**
   - Alternative to REST
   - Single endpoint, flexible queries

7. **WebSocket API**
   - Real-time updates
   - Bi-directional communication

---

## Conclusion

This API design provides:
- ✅ **Zero Duplication** - Unified endpoint layer
- ✅ **Future-Proof** - Multi-version support
- ✅ **License-Safe** - Pure Rust, MIT/Apache-2.0
- ✅ **Security-Ready** - Stub architecture for future
- ✅ **Transport-Agnostic** - HTTP + ZMQ from one codebase
- ✅ **Service-Oriented** - Clean separation of concerns

**Implementation Timeline:** 4 weeks  
**Total Endpoints:** 50-60  
**Lines of Code (estimated):** ~8,000 LOC  

**Ready to proceed with implementation!** 🚀

