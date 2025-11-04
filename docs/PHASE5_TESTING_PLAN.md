# Phase 5: Testing Plan

**Date:** 2025-10-30  
**Status:** In Progress  
**Goal:** Ensure Rust implementation matches Python behavior and is production-ready

---

## Testing Strategy Overview

| Test Type | Purpose | Tools | Duration | Priority |
|-----------|---------|-------|----------|----------|
| **Contract Tests** | Ensure Rust API matches Python API exactly | `insta`, `serde_json` | 3 days | 🔴 Critical |
| **Integration Tests** | Verify full stack workflows end-to-end | `pytest`, `cargo test` | 4 days | 🔴 Critical |
| **API Endpoint Tests** | Test all 60 REST endpoints | `reqwest`, `axum-test` | 2 days | 🟠 High |
| **Performance Benchmarks** | Compare Rust vs Python speed | `criterion`, `hyperfine` | 2 days | 🟡 Medium |
| **Stress Tests** | Large genomes, millions of neurons | Custom harness | 2 days | 🟢 Low |

**Total Estimated Time:** 2.5 weeks (13 days)

---

## 1. Contract Tests (API Compatibility)

### Purpose
Verify that Rust API responses are **byte-for-byte identical** to Python API responses for the same inputs.

### Approach
1. **Snapshot Testing**: Use `insta` crate to capture API responses
2. **JSON Comparison**: Deep compare JSON responses (ignore timestamp/dynamic fields)
3. **Error Format Matching**: Ensure error messages match Python format

### Test Cases

| Category | Test Count | Description |
|----------|------------|-------------|
| **Genome Operations** | 5 | Load, save, validate, reset |
| **Cortical Area CRUD** | 12 | Create, read, update, delete, list |
| **Brain Region CRUD** | 8 | Create, read, update, delete, list |
| **Neuron Operations** | 10 | Create, delete, query, update properties |
| **Synapse Operations** | 8 | Create, delete, query, update weights |
| **System Queries** | 7 | Health, status, version, stats |
| **Analytics** | 10 | Connectivity, density, counts |
| **TOTAL** | **60** | One test per API endpoint |

### Implementation Files
- `feagi-api/tests/contract_tests.rs` - Main contract test suite
- `feagi-api/tests/snapshots/` - Snapshot files (JSON responses)
- `feagi-api/tests/fixtures/` - Test data (genomes, expected responses)

### Success Criteria
✅ All 60 endpoints produce identical JSON structure to Python  
✅ Error formats match Python exactly  
✅ Edge cases handled consistently (empty data, invalid IDs, etc.)

---

## 2. Integration Tests (Full Stack)

### Purpose
Verify that the entire pipeline works end-to-end without mocking.

### Test Scenarios

#### Scenario 1: Complete Brain Development
```
1. Load barebones_genome.json
2. Validate genome
3. Run neuroembryogenesis
4. Verify cortical areas created
5. Verify neurons created (count, positions)
6. Verify synapses created (connectivity)
7. Query neuron properties
8. Run 10 burst cycles
9. Verify neurons fired
10. Save brain state to JSON
11. Load saved state and compare
```

#### Scenario 2: Incremental Brain Building
```
1. Start with empty connectome
2. Add cortical area "visual_v1"
3. Create 1000 neurons in v1
4. Add cortical area "motor_m1"
5. Create 500 neurons in m1
6. Create projector morphology v1 → m1
7. Apply morphology (create synapses)
8. Verify connectivity
9. Inject sensory data to v1
10. Run burst cycle
11. Verify m1 neurons activated
```

#### Scenario 3: Genome Formats
```
1. Load flat genome (2.0 format)
2. Convert to hierarchical (2.1 format)
3. Save as hierarchical
4. Load hierarchical
5. Verify brain structure identical
```

#### Scenario 4: Large Genome
```
1. Load vision_genome.json (largest test genome)
2. Verify all 10+ cortical areas created
3. Verify ~10K+ neurons created
4. Verify ~50K+ synapses created
5. Run 100 burst cycles
6. Measure performance (neurons/sec, synapses/sec)
```

### Implementation Files
- `feagi-core/tests/integration/test_full_pipeline.rs`
- `feagi-core/tests/integration/test_incremental_building.rs`
- `feagi-core/tests/integration/test_genome_formats.rs`
- `feagi-core/tests/integration/test_large_scale.rs`

### Success Criteria
✅ All 4 scenarios pass without errors  
✅ Neuron counts match expected values  
✅ Synapse counts match expected values  
✅ Burst engine runs without crashes  
✅ State persistence works (save/load roundtrip)

---

## 3. API Endpoint Tests

### Purpose
Test all 60 REST endpoints via HTTP, ensuring proper request/response handling.

### Test Structure
```rust
#[tokio::test]
async fn test_create_cortical_area() {
    // Setup: Start test server with initialized NPU
    let app = test_app().await;
    
    // Test: POST /api/v1/connectome/areas
    let response = app
        .post("/api/v1/connectome/areas")
        .json(&CreateAreaRequest {
            cortical_id: "test01".to_string(),
            name: "Test Area".to_string(),
            dimensions: (10, 10, 1),
            area_type: "memory".to_string(),
        })
        .send()
        .await;
    
    // Assert: 201 Created
    assert_eq!(response.status(), 201);
    
    // Assert: Response body matches schema
    let area: CorticalAreaResponse = response.json().await;
    assert_eq!(area.cortical_id, "test01");
    assert_eq!(area.name, "Test Area");
    
    // Verify: Area exists in BDU
    let get_response = app.get("/api/v1/connectome/areas/test01").send().await;
    assert_eq!(get_response.status(), 200);
}
```

### Test Categories
- **Happy Path**: Valid requests return expected responses
- **Validation**: Invalid requests return 400 with proper error messages
- **Not Found**: Missing resources return 404
- **Conflict**: Duplicate IDs return 409
- **Server Error**: Internal errors return 500 with details

### Implementation Files
- `feagi-api/tests/api_tests/health.rs`
- `feagi-api/tests/api_tests/genome.rs`
- `feagi-api/tests/api_tests/cortical_areas.rs`
- `feagi-api/tests/api_tests/brain_regions.rs`
- `feagi-api/tests/api_tests/neurons.rs`
- `feagi-api/tests/api_tests/synapses.rs`
- `feagi-api/tests/api_tests/runtime.rs`
- `feagi-api/tests/api_tests/analytics.rs`

### Success Criteria
✅ All 60 endpoints return correct HTTP status codes  
✅ Response bodies match OpenAPI schema  
✅ Error responses include helpful messages  
✅ Concurrent requests handled correctly

---

## 4. Performance Benchmarks

### Purpose
Measure performance and compare Rust vs Python implementation.

### Benchmarks

| Benchmark | Description | Target |
|-----------|-------------|--------|
| **Genome Loading** | Parse and validate genome JSON | <10ms for barebones |
| **Neurogenesis** | Create 10K neurons | <50ms |
| **Synaptogenesis** | Create 50K synapses | <100ms |
| **Burst Cycle** | Single burst with 10K neurons | <5ms |
| **API Latency** | Average endpoint response time | <2ms |
| **Memory Usage** | Peak memory for 100K neurons | <500MB |

### Comparison Metrics
```
Benchmark: Load barebones_genome.json
- Python: 250ms
- Rust:   8ms
- Speedup: 31.2x

Benchmark: Create 10K neurons
- Python: 1,200ms
- Rust:   35ms
- Speedup: 34.3x

Benchmark: Create 50K synapses
- Python: 3,500ms
- Rust:   85ms
- Speedup: 41.2x
```

### Implementation Files
- `feagi-core/benches/genome_loading.rs`
- `feagi-core/benches/neurogenesis.rs`
- `feagi-core/benches/synaptogenesis.rs`
- `feagi-core/benches/burst_cycle.rs`
- `feagi-core/benches/api_latency.rs`

### Tools
- **Rust**: `criterion` crate for precise benchmarking
- **Python**: `pytest-benchmark` for baseline measurements
- **Comparison**: `hyperfine` for CLI tool comparison

### Success Criteria
✅ Rust is at least 10x faster than Python for genome loading  
✅ Rust is at least 20x faster for neuron/synapse creation  
✅ API latency is under 2ms average  
✅ Memory usage is reasonable (no leaks)

---

## 5. Stress Tests

### Purpose
Verify system stability under extreme conditions.

### Test Cases

#### Test 1: Large Genome (1M neurons)
```
- Create genome with 100 cortical areas
- 10,000 neurons per area = 1M total neurons
- Dense connectivity (10 synapses/neuron avg) = 10M synapses
- Run 1000 burst cycles
- Verify: No crashes, no memory leaks, consistent timing
```

#### Test 2: Rapid API Requests
```
- 1000 concurrent API requests
- Mix of reads and writes
- Measure: Throughput, error rate, latency distribution
- Verify: No race conditions, no deadlocks
```

#### Test 3: Long-Running Burst Engine
```
- Start burst engine
- Run for 1 hour continuously
- Monitor: Memory usage, CPU usage, burst timing consistency
- Verify: No degradation over time
```

### Implementation Files
- `feagi-core/tests/stress/test_large_genome.rs`
- `feagi-core/tests/stress/test_concurrent_api.rs`
- `feagi-core/tests/stress/test_long_running.rs`

### Success Criteria
✅ System handles 1M neurons without crashes  
✅ Concurrent API requests succeed (>99% success rate)  
✅ Long-running tests show no memory leaks  
✅ Performance remains consistent over time

---

## Testing Timeline

| Week | Days | Focus | Deliverable |
|------|------|-------|-------------|
| **Week 1** | 1-3 | Contract Tests | 60 endpoint contract tests passing |
| **Week 1** | 4-5 | Integration Tests | 4 full-stack scenarios passing |
| **Week 2** | 6-7 | API Endpoint Tests | All 60 endpoints tested via HTTP |
| **Week 2** | 8-9 | Performance Benchmarks | Rust vs Python comparison report |
| **Week 3** | 10-11 | Stress Tests | Large-scale stability verified |
| **Week 3** | 12-13 | Documentation & Cleanup | Test coverage report, CI integration |

---

## Test Coverage Goals

| Component | Target Coverage | Current Coverage |
|-----------|----------------|------------------|
| **feagi-evo** | 90% | 60% (basic tests exist) |
| **feagi-bdu** | 85% | 40% (some tests exist) |
| **feagi-services** | 90% | 0% (no tests yet) |
| **feagi-api** | 95% | 0% (no tests yet) |
| **Overall** | 85% | 30% |

---

## CI/CD Integration

### GitHub Actions Workflow
```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run unit tests
        run: cargo test --all
  
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run integration tests
        run: cargo test --test '*' --features integration
  
  contract-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Python
        uses: actions/setup-python@v4
      - name: Run contract tests
        run: cargo test --test contract_tests
  
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
```

---

## Success Metrics

**Phase 5 is complete when:**

✅ All 60 contract tests pass (API compatibility confirmed)  
✅ All 4 integration test scenarios pass (full stack works)  
✅ All 60 API endpoint tests pass (HTTP layer works)  
✅ Performance benchmarks show 10x+ speedup over Python  
✅ Stress tests demonstrate stability at scale  
✅ Test coverage reaches 85%+  
✅ CI/CD pipeline runs all tests automatically

**Timeline:** 2.5 weeks (13 working days)  
**Current Status:** Day 0 - Starting contract tests




