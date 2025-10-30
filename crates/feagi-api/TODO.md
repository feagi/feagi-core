# feagi-api TODO List

## Phase 1: Infrastructure ✅ COMPLETE

- [x] Create feagi-api crate structure and Cargo.toml
- [x] Implement common types (ApiRequest, ApiResponse, ApiError)
- [x] Create security stubs (AuthContext, Permission)
- [x] Set up Axum server with basic routing
- [x] Set up ZMQ server with basic message handling
- [x] Create middleware (CORS, logging)
- [x] Create Swagger UI integration with custom styling support
- [x] Set up contract testing infrastructure
- [x] Create health endpoint (first working endpoint)
- [ ] Test health endpoint via HTTP and ZMQ (requires running server)

**Status:** 9/10 complete (90%)  
**Remaining:** Manual integration testing

---

## Phase 2: Endpoint Implementation (NEXT)

### Cortical Area Endpoints

- [ ] `GET /api/v1/cortical-areas` - List all cortical areas
- [ ] `POST /api/v1/cortical-areas` - Create new cortical area
- [ ] `GET /api/v1/cortical-areas/:id` - Get cortical area by ID
- [ ] `PUT /api/v1/cortical-areas/:id` - Update cortical area
- [ ] `DELETE /api/v1/cortical-areas/:id` - Delete cortical area
- [ ] Add contract tests for all cortical area endpoints

### Brain Region Endpoints

- [ ] `GET /api/v1/brain-regions` - List all brain regions
- [ ] `GET /api/v1/brain-regions/:id` - Get brain region by ID
- [ ] Add contract tests for brain region endpoints

### Genome Endpoints

- [ ] `GET /api/v1/genome/info` - Get genome metadata
- [ ] `POST /api/v1/genome/load` - Load genome from file
- [ ] `POST /api/v1/genome/save` - Save genome to file
- [ ] `POST /api/v1/genome/validate` - Validate genome structure
- [ ] Add contract tests for genome endpoints

### Analytics Endpoints

- [ ] `GET /api/v1/analytics/stats` - System statistics
- [ ] `GET /api/v1/analytics/metrics` - Performance metrics
- [ ] Add contract tests for analytics endpoints

### Agent Endpoints

- [ ] `GET /api/v1/agents` - List registered agents
- [ ] `POST /api/v1/agents/:id/heartbeat` - Agent heartbeat
- [ ] Add contract tests for agent endpoints

---

## Phase 3: Testing & Integration

### Integration Tests

- [ ] Set up test HTTP server
- [ ] Test all endpoints via HTTP
- [ ] Test all endpoints via ZMQ
- [ ] Capture Python API snapshots for all endpoints
- [ ] Implement snapshot comparison tests

### Performance Tests

- [ ] Benchmark HTTP endpoint latency
- [ ] Benchmark ZMQ endpoint latency
- [ ] Compare with Python API performance
- [ ] Optimize hot paths if needed

### CI/CD

- [ ] Set up GitHub Actions for contract tests
- [ ] Set up automated OpenAPI spec validation
- [ ] Set up performance regression tests
- [ ] Document deployment process

---

## Phase 4: Security Implementation

### Authentication

- [ ] Implement JWT validation
- [ ] Implement API key validation
- [ ] Implement mTLS validation
- [ ] Add authentication middleware

### Authorization

- [ ] Implement RBAC (Role-Based Access Control)
- [ ] Add permission checks to all endpoints
- [ ] Document permission requirements

### Encryption

- [ ] Implement ChaCha20-Poly1305 for ZMQ
- [ ] Add TLS/HTTPS support for HTTP
- [ ] Document security configuration

---

## Phase 5: Production Readiness

### Configuration

- [ ] Externalize all configuration (ports, hosts, etc.)
- [ ] Add environment variable support
- [ ] Add TOML config file support
- [ ] Document all configuration options

### Observability

- [ ] Add structured logging
- [ ] Add metrics collection (Prometheus)
- [ ] Add distributed tracing
- [ ] Add health check endpoints for all dependencies

### Documentation

- [ ] Complete API reference documentation
- [ ] Add deployment guide
- [ ] Add troubleshooting guide
- [ ] Add security best practices guide

### Migration

- [ ] Document migration from Python to Rust API
- [ ] Create migration scripts/tools
- [ ] Update Brain Visualizer to use Rust API
- [ ] Deprecate Python API endpoints

---

## Notes

**Last Updated:** 2025-10-29  
**Current Phase:** Phase 1 Complete, Phase 2 Next  
**Blockers:** None

