# feagi-api

REST API layer for FEAGI with HTTP and ZMQ transport adapters.

## Architecture Overview

**feagi-api** is responsible for **business logic** of API endpoints, while **feagi-io** handles **transport infrastructure**.

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
│  │ HTTP (Axum)      │ ZMQ (feagi-io)      ││
│  │ transports/http/ │ transports/zmq/      ││
│  └──────────────────┴──────────────────────┘│
└─────────────────────────────────────────────┘
                     ↓
┌─────────────────────────────────────────────┐
│ feagi-io (Transport Infrastructure)        │
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

**feagi-io:**
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

// ZMQ adapter (feagi-io integration)
async fn zmq_health_handler(state: &ZmqApiState) -> ZmqResponse {
    endpoints::health::health_check(&auth_ctx, state.analytics).await
}
```

## Integration with feagi-io

The ZMQ transport adapter in `feagi-api` **uses** `feagi-io` infrastructure:

```rust
// feagi-io provides the ZMQ transport
use feagi_io::api_control::ApiControlStream;

// feagi-api provides the business logic
use feagi_api::transports::zmq::handle_api_control_request;

// In feagi-io::api_control, when a REST request arrives:
let response = handle_api_control_request(
    method,
    path,
    body,
    &api_state
).await;
```

## Why Keep Them Separate?

| Concern | feagi-io | feagi-api |
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

### ZMQ (via feagi-io)

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
- ✅ ZMQ server (feagi-io integration)
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
feagi-io = { path = "../feagi-io" }            # ZMQ infrastructure

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

### Integrating with feagi-io (ZMQ)

```rust
// In feagi-io::api_control when REST request arrives
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

## RGBD Registration Example

For RGBD sensors, register two sensory units in `device_registrations`:
- `Vision` for RGB frames
- `DepthMap` for quantized depth volumes (`x/y` topology, `z` depth bins)

Use the same `cortical_unit_index` and `bundle_id` so FEAGI and BV can treat both streams as one camera rig.

```json
{
  "input_units_and_encoder_properties": {
    "Vision": [
      [
        {
          "friendly_name": "front_rgb_camera",
          "cortical_unit_index": 0,
          "io_configuration_flags": {
            "frame_change_handling": "Absolute"
          },
          "device_grouping": [
            {
              "friendly_name": "rgb_sensor",
              "device_properties": {
                "bundle_id": "front_rgbd",
                "bundle_type": "rgbd_camera"
              }
            }
          ]
        },
        {
          "CartesianPlane": {
            "image_resolution": { "width": 640, "height": 480 },
            "color_space": "Gamma",
            "color_channel_layout": "RGB"
          }
        }
      ]
    ],
    "DepthMap": [
      [
        {
          "friendly_name": "front_depth_camera",
          "cortical_unit_index": 0,
          "io_configuration_flags": {
            "frame_change_handling": "Absolute"
          },
          "device_grouping": [
            {
              "friendly_name": "depth_sensor",
              "device_properties": {
                "bundle_id": "front_rgbd",
                "bundle_type": "rgbd_camera"
              }
            }
          ]
        },
        {
          "MiscData": {
            "width": 160,
            "height": 120,
            "depth": 64
          }
        }
      ]
    ]
  },
  "output_units_and_decoder_properties": {},
  "feedbacks": {}
}
```

Notes:
- `DepthMap` uses cortical unit reference `dpt` (IPU type key `idpt`).
- `MiscData.width/height/depth` controls the depth cortical dimensions when auto-create is enabled.

## License

Apache-2.0





