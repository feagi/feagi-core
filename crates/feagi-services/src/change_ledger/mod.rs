// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome change ledger: records who changed what in the live genome, and when.
//!
//! See `README.md` in this folder for the design.

pub mod context;
pub mod ledger;
pub mod recording_connectome;
pub mod recording_genome;
pub mod replay;
mod snapshot;
pub mod types;

pub use context::{current_change_context, with_change_context, ChangeContext, ReplaySource};
pub use ledger::{ChangeLedger, ChangeLedgerConfig};
pub use recording_connectome::RecordingConnectomeService;
pub use recording_genome::RecordingGenomeService;
pub use replay::apply_change;
pub use types::{
    ApplyGenomeChangeOutcome, ApplyGenomeChangeRequest, ChangeOrigin, ChangeTarget,
    ChangeTargetKind, ChangesPage, GenomeChange, GenomeChangeOperation,
};

use crate::traits::{ConnectomeService, GenomeService};
use crate::types::ServiceResult;
use feagi_evolutionary::RuntimeGenome;
use parking_lot::RwLock;
use std::sync::Arc;

/// A ledger and the recording services that write to it.
pub struct RecordedServices {
    pub ledger: Arc<ChangeLedger>,
    pub genome: Arc<dyn GenomeService + Send + Sync>,
    pub connectome: Arc<dyn ConnectomeService + Send + Sync>,
}

/// Wrap the service implementations so every structural change is recorded.
///
/// `current_genome` must be the genome service's live genome (the one
/// `connectome` also mutates); recorded history is persisted into it.
pub fn record_changes(
    genome: Arc<dyn GenomeService + Send + Sync>,
    connectome: Arc<dyn ConnectomeService + Send + Sync>,
    current_genome: Arc<RwLock<Option<RuntimeGenome>>>,
    config: ChangeLedgerConfig,
) -> ServiceResult<RecordedServices> {
    let ledger = Arc::new(ChangeLedger::new(config, current_genome)?);
    Ok(RecordedServices {
        genome: Arc::new(RecordingGenomeService::new(
            genome,
            Arc::clone(&connectome),
            Arc::clone(&ledger),
        )),
        connectome: Arc::new(RecordingConnectomeService::new(
            connectome,
            Arc::clone(&ledger),
        )),
        ledger,
    })
}
