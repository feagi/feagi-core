// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Contract tests for FEAGI API.

These tests ensure 100% compatibility with the Python FastAPI implementation
by comparing API responses.

## Approach

1. **Snapshot Testing**: Use `insta` to capture and compare JSON responses
2. **JSON Diff**: Use `assert-json-diff` for detailed comparison
3. **Test Server**: Spin up a test HTTP server for each test suite
4. **Fixtures**: Use real test data (genomes, etc.) for realistic scenarios

## Test Organization

Each endpoint group has its own test module:
- `cortical_areas_tests.rs` - Cortical area CRUD
- `brain_regions_tests.rs` - Brain region operations
- `genome_tests.rs` - Genome load/save/validate
- `neurons_tests.rs` - Neuron operations
- `runtime_tests.rs` - Runtime control
- `analytics_tests.rs` - Statistics and monitoring

## Running Contract Tests

```bash
# Run all contract tests
cargo test --test contract

# Run specific test module
cargo test --test contract cortical_areas

# Update snapshots (after verifying changes are correct)
cargo test --test contract -- --review
```

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod test_utils;

// Contract test modules
pub mod cortical_areas_tests;
pub mod health_tests;

// TODO: Add more test modules as endpoints are verified
// pub mod brain_regions_tests;
// pub mod genome_tests;
// pub mod neurons_tests;
// pub mod runtime_tests;
// pub mod analytics_tests;
