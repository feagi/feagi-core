# API Response Snapshots

This directory contains JSON snapshots of Python FastAPI responses used for contract testing.

## Purpose

These snapshots ensure that the Rust API maintains 100% backward compatibility with the Python FastAPI implementation. Any breaking changes to request/response structures will be caught by the contract tests.

## Capturing Snapshots

### Prerequisites

1. Start the Python FEAGI API server:
```bash
cd /path/to/feagi-py
python -m feagi.api.main
# Server should be running on http://localhost:8000
```

2. Ensure a genome is loaded and FEAGI is running

### Capture Health Check Snapshot

```bash
curl -s http://localhost:8000/api/v1/health | jq . > health_check.json
```

### Capture Readiness Check Snapshot

```bash
curl -s http://localhost:8000/api/v1/ready | jq . > readiness_check.json
```

### Capture All Endpoints

Create a script to capture all endpoints:

```bash
#!/bin/bash
# capture_snapshots.sh

PYTHON_API="http://localhost:8000"
SNAPSHOT_DIR="tests/snapshots"

# Health endpoints
curl -s "$PYTHON_API/api/v1/health" | jq . > "$SNAPSHOT_DIR/health_check.json"
curl -s "$PYTHON_API/api/v1/ready" | jq . > "$SNAPSHOT_DIR/readiness_check.json"

# Cortical area endpoints
curl -s "$PYTHON_API/api/v1/cortical-areas" | jq . > "$SNAPSHOT_DIR/cortical_areas_list.json"
curl -s "$PYTHON_API/api/v1/cortical-areas/v1" | jq . > "$SNAPSHOT_DIR/cortical_area_get.json"

# Genome endpoints
curl -s "$PYTHON_API/api/v1/genome/info" | jq . > "$SNAPSHOT_DIR/genome_info.json"

# Agent endpoints
curl -s "$PYTHON_API/api/v1/agents" | jq . > "$SNAPSHOT_DIR/agents_list.json"

echo "Snapshots captured successfully!"
```

## Using Snapshots in Tests

### Structure Validation

```rust
#[test]
fn test_response_structure() {
    let snapshot = load_snapshot("health_check.json");
    assert_json_structure_matches(&actual_response, &snapshot, "");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_health_endpoint_matches_python() {
    // Start Rust API server
    let rust_api = "http://localhost:8080";
    
    // Query Rust API
    let rust_response = reqwest::get(format!("{}/api/v1/health", rust_api))
        .await.unwrap()
        .json::<Value>()
        .await.unwrap();
    
    // Load Python snapshot
    let python_snapshot = load_snapshot("health_check.json");
    
    // Compare (ignoring dynamic fields like timestamps)
    assert_json_structure_matches(&rust_response, &python_snapshot, "");
}
```

## Snapshot Format

All snapshots should be:
- Valid JSON
- Pretty-printed (use `jq .`)
- Contain complete responses (headers, status, body)
- Be captured from a clean FEAGI state (known genome loaded)

## Dynamic Fields

Some fields are dynamic and should be ignored in comparisons:
- `timestamp` - Current time
- `genome_timestamp` - Genome modification time
- `connectome_path` - Filesystem path
- `neuron_count` - Varies by loaded genome
- `synapse_count` - Varies by loaded genome
- `cortical_area_count` - Varies by loaded genome

The test utilities in `test_utils.rs` automatically handle these dynamic fields.

## Updating Snapshots

Snapshots should be updated when:
1. The Python API intentionally changes response format
2. New fields are added to responses
3. Field types change

**DO NOT** update snapshots to make failing tests pass without understanding why the format changed!

## Example Snapshot

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "brain_readiness": true,
    "burst_engine": true,
    "neuron_count": 1000,
    "synapse_count": 5000,
    "cortical_area_count": 10,
    "genome_validity": true,
    "influxdb_availability": false,
    "connectome_path": "/path/to/connectome",
    "genome_timestamp": "2025-10-29T12:34:56Z",
    "change_state": "saved",
    "changes_saved_externally": false
  },
  "timestamp": "2025-10-29T12:34:56Z"
}
```

## Running Contract Tests

```bash
# Run all contract tests
cargo test --test contract_tests

# Run specific test
cargo test --test contract_tests test_health_check_response_structure

# Run with output
cargo test --test contract_tests -- --nocapture
```

## CI Integration

Contract tests should run in CI:

```yaml
# .github/workflows/contract_tests.yml
- name: Start Python API
  run: |
    python -m feagi.api.main &
    sleep 5  # Wait for server to start

- name: Run Contract Tests
  run: cargo test --test contract_tests
```

## Status

**Captured Snapshots:**
- [ ] health_check.json
- [ ] readiness_check.json
- [ ] cortical_areas_list.json
- [ ] cortical_area_get.json
- [ ] genome_info.json
- [ ] agents_list.json

**TODO:**
- Capture all endpoint snapshots from Python API
- Add integration tests that compare live responses
- Set up CI pipeline for contract testing
- Document all response format changes


