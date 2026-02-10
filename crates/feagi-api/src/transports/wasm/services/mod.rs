// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM-specific service implementations
//!
//! These services extract data from RuntimeGenome and implement the same
//! service traits used by HTTP/ZMQ adapters, ensuring endpoint compatibility.

pub mod analytics;
pub mod connectome;
pub mod genome;
pub mod neuron;
pub mod runtime;
// pub mod system; // TODO: File not yet implemented

pub use analytics::WasmAnalyticsService;
pub use connectome::WasmConnectomeService;
pub use genome::WasmGenomeService;
pub use neuron::WasmNeuronService;
pub use runtime::WasmRuntimeService;
pub use system::WasmSystemService;
