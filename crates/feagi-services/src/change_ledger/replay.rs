// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Apply a change recorded on another instance.
//!
//! The change runs through the recording services under a replay context, so it is
//! recorded with its original change id, group, timestamp and agent, and marked as
//! replayed (which lets relays skip it and avoid echoing it back).

use super::context::{with_change_context, ChangeContext, ReplaySource};
use super::ledger::ChangeLedger;
use super::types::{ApplyGenomeChangeOutcome, ApplyGenomeChangeRequest, GenomeChangeOperation};
use crate::traits::{ConnectomeService, GenomeService};
use crate::types::{ServiceError, ServiceResult};

/// Apply `request` once. `genome` and `connectome` must be the recording services
/// that write to `ledger`; otherwise the change is applied but not recorded.
pub async fn apply_change(
    request: ApplyGenomeChangeRequest,
    ledger: &ChangeLedger,
    genome: &dyn GenomeService,
    connectome: &dyn ConnectomeService,
) -> ServiceResult<ApplyGenomeChangeOutcome> {
    if request.change_id.trim().is_empty() {
        return Err(ServiceError::InvalidInput(
            "change_id must not be empty".to_string(),
        ));
    }
    if request.group_id.trim().is_empty() {
        return Err(ServiceError::InvalidInput(
            "group_id must not be empty".to_string(),
        ));
    }
    if !request.operation.is_replayable() {
        return Err(ServiceError::InvalidInput(
            "operation is a whole-brain marker and cannot be replayed".to_string(),
        ));
    }
    let agent_id = request
        .agent_id
        .as_deref()
        .map(|id| ledger.validate_agent_id(id))
        .transpose()?;

    let _guard = ledger.lock_apply().await;
    if ledger.contains_change_id(&request.change_id) {
        return Ok(ApplyGenomeChangeOutcome::AlreadyApplied);
    }

    let context = ChangeContext {
        agent_id,
        group_id: None,
        replay: Some(ReplaySource {
            change_id: request.change_id.clone(),
            group_id: request.group_id.clone(),
            timestamp_ms: request.timestamp_ms,
        }),
    };
    with_change_context(context, dispatch(request.operation, genome, connectome)).await?;

    let recorded = ledger.find(&request.change_id).ok_or_else(|| {
        ServiceError::Internal(format!(
            "change {} applied but not recorded; services are not wired to this ledger",
            request.change_id
        ))
    })?;
    Ok(ApplyGenomeChangeOutcome::Applied {
        sequence: recorded.sequence,
    })
}

/// Call the service method that originally produced `operation`.
async fn dispatch(
    operation: GenomeChangeOperation,
    genome: &dyn GenomeService,
    connectome: &dyn ConnectomeService,
) -> ServiceResult<()> {
    use GenomeChangeOperation as Op;
    match operation {
        Op::CreateCorticalAreas { params } => genome.create_cortical_areas(params).await.map(drop),
        Op::UpdateCorticalArea {
            cortical_id,
            changes,
        } => genome
            .update_cortical_area(&cortical_id, changes)
            .await
            .map(drop),
        Op::DeleteCorticalArea { cortical_id } => {
            connectome.delete_cortical_area(&cortical_id).await
        }
        Op::CreateBrainRegion { params } => connectome.create_brain_region(*params).await.map(drop),
        Op::UpdateBrainRegion {
            region_id,
            properties,
        } => connectome
            .update_brain_region(&region_id, properties)
            .await
            .map(drop),
        Op::DeleteBrainRegion { region_id } => connectome.delete_brain_region(&region_id).await,
        Op::CreateMorphology {
            morphology_id,
            morphology,
        } => {
            connectome
                .create_morphology(morphology_id, *morphology)
                .await
        }
        Op::UpdateMorphology {
            morphology_id,
            morphology,
        } => {
            connectome
                .update_morphology(morphology_id, *morphology)
                .await
        }
        Op::DeleteMorphology { morphology_id } => {
            connectome.delete_morphology(&morphology_id).await
        }
        Op::RenameMorphology { old_id, new_id } => {
            connectome.rename_morphology(&old_id, &new_id).await
        }
        Op::UpdateCorticalMapping {
            src_area_id,
            dst_area_id,
            mapping_data,
        } => connectome
            .update_cortical_mapping(src_area_id, dst_area_id, mapping_data)
            .await
            .map(drop),
        Op::UpsertClassifier { classifier } => connectome.upsert_classifier(*classifier).await,
        Op::DeleteClassifier { classifier_id } => {
            connectome.delete_classifier(&classifier_id).await
        }
        Op::RekeyMemoryTwinSource {
            memory_area_id,
            old_src_area_id,
            new_src_area_id,
        } => {
            connectome
                .rekey_memory_twin_source(&memory_area_id, &old_src_area_id, &new_src_area_id)
                .await
        }
        Op::GenomeLoaded { .. } | Op::ConnectomeReset | Op::ConnectomeImported => {
            Err(ServiceError::InvalidInput(
                "operation is a whole-brain marker and cannot be replayed".to_string(),
            ))
        }
    }
}
