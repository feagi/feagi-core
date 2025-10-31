# feagi-api

REST API layer for FEAGI with HTTP and ZMQ transport adapters.

## Architecture Overview

**feagi-api** is responsible for **business logic** of API endpoints, while **feagi-pns** handles **transport infrastructure**.

```
┌─────────────────────────────────────────────┐
│ feagi-api (Business Logic Layer)            │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │ endpoints/ (Transport-agnostic)        │ │
│  │  • health.rs                           │ │
│  │  • cortical_areas.rs                   │ │
│  │  • genome.rs                           │ │
│  └────────────────────────────────────────┘ │
│              ↓ Called by ↓                   │
│  ┌──────────────────┬──────────────────────┐│
│  │ HTTP (Axum)      │ ZMQ (feagi-pns)      ││
│  │ transports/http/ │ transports/zmq/      ││
│  └──────────────────┴──────────────────────┘│
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│ feagi-pns (Transport Infrastructure)        │
│  • api_control.rs (ZMQ ROUTER/DEALER)      │
│  • sensory.rs (PUSH/PULL)                   │
│  • motor.rs (PUB/SUB)                       │
│  • visualization.rs (PUB/SUB)               │
└─────────────────────────────────────────────┘
```

## Design Principles

### Clear Separation of Concerns

**feagi-api:**
- ✅ Defines API endpoints (what operations are available)
- ✅ Implements business logic (transport-agnostic)
- ✅ Provides thin transport adapters (HTTP + ZMQ)
- ❌ Does NOT implement ZMQ infrastructure

**feagi-pns:**
- ✅ Owns ALL ZMQ code (data + control plane)
- ✅ Provides `api_control` for REST-over-ZMQ
- ✅ Handles real-time streaming (sensory, motor, viz)
- ❌ Does NOT implement business logic

### Unified Endpoint Layer

All endpoints are **transport-agnostic**:

```rust
// src/endpoints/health.rs

pub async fn health_check(
    auth_ctx: &AuthContext,
    analytics_service: Arc<dyn AnalyticsService>,
) -> ApiResult<HealthCheckResponseV1> {
    // Business logic here - NO HTTP or ZMQ code!
}
```

This endpoint is called by **both** HTTP and ZMQ adapters:

```rust
// HTTP adapter (Axum)
async fn http_health_handler(State(state): State<ApiState>) -> Json<ApiResponse<...>> {
    endpoints::health::health_check(&auth_ctx, state.analytics).await
}

// ZMQ adapter (feagi-pns integration)
async fn zmq_health_handler(state: &ZmqApiState) -> ZmqResponse {
    endpoints::health::health_check(&auth_ctx, state.analytics).await
}
```

## Integration with feagi-pns

The ZMQ transport adapter in `feagi-api` **uses** `feagi-pns` infrastructure:

```rust
// feagi-pns provides the ZMQ transport
use feagi_pns::api_control::ApiControlStream;

// feagi-api provides the business logic
use feagi_api::transports::zmq::handle_api_control_request;

// In feagi-pns::api_control, when a REST request arrives:
let response = handle_api_control_request(
    method,
    path,
    body,
    &api_state
).await;
```

## Why Keep Them Separate?

| Concern | feagi-pns | feagi-api |
|---------|-----------|-----------|
| **Responsibility** | How to move data | What operations are available |
| **Deployment** | Main FEAGI process (hot path) | Could be separate process |
| **Evolution** | Add transports (WebSocket, QUIC) | Add endpoints (features, versioning) |
| **Consumers** | Agents, BV, connectors | Web UI, CLI, mgmt scripts |
| **Compilation** | Must compile for all I/O | Can compile with only API deps |

## API Versioning

Supports multiple API versions:

```
/api/v1/health      → Version 1 (stable)
/api/v2/health      → Version 2 (future)
/health             → Version-agnostic (always available)
```

## Transport Support

### HTTP (Axum)

```
GET  /api/v1/health          → Health check
GET  /api/v1/cortical-areas  → List cortical areas
POST /api/v1/cortical-areas  → Create cortical area
```

**Features:**
- OpenAPI 3.0 documentation (utoipa)
- Custom Swagger UI styling
- CORS support
- Request/response logging

### ZMQ (via feagi-pns)

Same endpoints available over ZMQ ROUTER/DEALER:

```json
{
  "method": "GET",
  "path": "/api/v1/health",
  "body": null
}
```

Response:
```json
{
  "status": 200,
  "body": {
    "success": true,
    "data": { ... },
    "timestamp": "..."
  }
}
```

## Contract Testing

All endpoints tested for **100% compatibility** with Python FastAPI:

```rust
#[test]
fn test_health_check_response_format() {
    let rust_response = rust_api.get("/v1/health").await;
    let python_snapshot = load_snapshot("health_check.json");
    
    assert_json_match!(rust_response, python_snapshot);
}
```

## Security (Stub Architecture)

Security stubs are in place for future implementation:

```rust
// Authentication (stub - always anonymous for now)
let auth_ctx = AuthContext::anonymous();

// Authorization (stub - always allowed for now)
Authorizer::authorize(&auth_ctx, Permission::CorticalAreaRead)?;
```

## Status

**Current (Phase 1 - Infrastructure):**
- ✅ Crate structure
- ✅ Common types (ApiRequest, ApiResponse, ApiError)
- ✅ Security stubs (AuthContext, Permission)
- ✅ HTTP server (Axum) with basic routing
- ✅ ZMQ server (feagi-pns integration)
- ✅ Middleware (CORS, logging)
- ✅ Health endpoint (working for HTTP + ZMQ)

**Next (Phase 2 - Endpoints):**
- 🔄 OpenAPI/Swagger UI integration
- 🔄 Cortical area endpoints (CRUD)
- 🔄 Brain region endpoints
- 🔄 Genome endpoints
- 🔄 Analytics endpoints

**Future (Phase 3 - Testing):**
- ⏳ Contract tests (Python compatibility)
- ⏳ Integration tests
- ⏳ Performance benchmarks

## Dependencies

```toml
feagi-services = { path = "../feagi-services" }  # Service layer
feagi-types = { path = "../feagi-types" }        # Core types
feagi-pns = { path = "../feagi-pns" }            # ZMQ infrastructure

axum = "0.7"              # HTTP server
utoipa = "4.0"            # OpenAPI generation
tower-http = "0.5"        # Middleware (CORS, logging)
```

## Example Usage

### Starting the HTTP API

```rust
use feagi_api::transports::http;

let state = http::ApiState {
    analytics_service: Arc::new(analytics),
};

let app = http::create_app(state);

axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
```

### Integrating with feagi-pns (ZMQ)

```rust
// In feagi-pns::api_control when REST request arrives
use feagi_api::transports::zmq;

let api_state = zmq::ZmqApiState {
    analytics_service: Arc::new(analytics),
};

let response = zmq::handle_api_control_request(
    method,
    path,
    body,
    &api_state
).await;

// Send response back over ZMQ
```

## License

Apache-2.0




