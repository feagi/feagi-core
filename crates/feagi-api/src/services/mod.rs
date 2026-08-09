// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome-backed service implementations.
//!
//! These implement the `feagi-services` traits by reading a [`feagi_evolutionary::RuntimeGenome`].
//! They are transport-agnostic: the same implementations serve the HTTP router and any other
//! adapter, so endpoint behaviour does not vary by transport.
//!
//! They cover the genome-derived surface (cortical areas, brain regions, morphologies, genome
//! metadata). Operations needing live neural state are answered by the runtime services instead.

use feagi_evolutionary::RuntimeGenome;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use parking_lot::RwLock;
use std::sync::Arc;

/// The genome the server is currently running, shared between the loader and the API services.
///
/// A shared handle rather than an owned snapshot: `/v1/genome/*` can replace the genome while the
/// server is running, and every service must observe the replacement immediately. `None` means no
/// genome has been loaded yet.
pub type SharedGenome = Arc<RwLock<Option<RuntimeGenome>>>;

/// Creates an empty genome handle for a server that has not loaded a genome yet.
pub fn empty_shared_genome() -> SharedGenome {
    Arc::new(RwLock::new(None))
}

/// Reads the current genome, or reports that none is loaded.
///
/// The closure runs under a read guard, so it must not block or await. Callers clone or project
/// what they need and return it.
pub fn with_genome<T>(
    genome: &SharedGenome,
    f: impl FnOnce(&RuntimeGenome) -> T,
) -> ServiceResult<T> {
    match genome.read().as_ref() {
        Some(runtime_genome) => Ok(f(runtime_genome)),
        None => Err(ServiceError::NotFound {
            resource: "genome".to_string(),
            id: "current".to_string(),
        }),
    }
}

/// The NPU handle the services drive, when the application has supplied one.
///
/// `None` means the API is running without an engine (genome inspection only), in which case
/// engine-dependent operations report that the NPU is unavailable.
pub type OptionalNpu = Option<Arc<dyn npu_access::NpuAccess>>;

/// Reports that no NPU was injected, for services asked to perform an engine operation.
pub fn npu_unavailable(operation: &str) -> ServiceError {
    ServiceError::NotImplemented(format!(
        "{} requires a running NPU, which this server was started without",
        operation
    ))
}

pub mod analytics;
pub mod connectome;
pub mod npu_access;
pub mod genome;
pub mod neuron;
pub mod runtime;
pub mod state;
pub mod system;

pub use npu_access::{NpuAccess, NpuCorticalArea};

pub use analytics::GenomeAnalyticsService;
pub use connectome::GenomeConnectomeService;
pub use genome::GenomeGenomeService;
pub use neuron::GenomeNeuronService;
pub use runtime::GenomeRuntimeService;
pub use system::GenomeSystemService;
pub use state::create_api_state_from_genome;
