// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! [`GenomeService`] decorator that records successful mutations in the change ledger.

use super::context::current_change_context;
use super::ledger::ChangeLedger;
use super::snapshot::{area_snapshot, areas_snapshot};
use super::types::GenomeChangeOperation;
use crate::traits::{ConnectomeService, GenomeService};
use crate::types::*;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Wraps a genome service; reads and failed calls pass through unrecorded.
pub struct RecordingGenomeService {
    inner: Arc<dyn GenomeService + Send + Sync>,
    /// Unwrapped connectome service, used only to snapshot state around a change.
    reads: Arc<dyn ConnectomeService + Send + Sync>,
    ledger: Arc<ChangeLedger>,
}

impl RecordingGenomeService {
    pub fn new(
        inner: Arc<dyn GenomeService + Send + Sync>,
        reads: Arc<dyn ConnectomeService + Send + Sync>,
        ledger: Arc<ChangeLedger>,
    ) -> Self {
        Self {
            inner,
            reads,
            ledger,
        }
    }
}

#[async_trait]
impl GenomeService for RecordingGenomeService {
    async fn load_genome(&self, params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        let info = self.inner.load_genome(params).await?;
        self.ledger.record(
            GenomeChangeOperation::GenomeLoaded {
                genome_id: info.genome_id.clone(),
                genome_title: info.genome_title.clone(),
            },
            None,
            None,
            &current_change_context(),
        );
        Ok(info)
    }

    async fn save_genome(&self, params: SaveGenomeParams) -> ServiceResult<String> {
        self.inner.save_genome(params).await
    }

    async fn export_region_genome(&self, region_id: String) -> ServiceResult<String> {
        self.inner.export_region_genome(region_id).await
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        self.inner.get_genome_info().await
    }

    async fn validate_genome(&self, json_str: String) -> ServiceResult<bool> {
        self.inner.validate_genome(json_str).await
    }

    async fn reset_connectome(&self) -> ServiceResult<()> {
        self.inner.reset_connectome().await?;
        self.ledger.record(
            GenomeChangeOperation::ConnectomeReset,
            None,
            None,
            &current_change_context(),
        );
        Ok(())
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        let before = area_snapshot(self.reads.as_ref(), cortical_id).await;
        let info = self
            .inner
            .update_cortical_area(cortical_id, changes.clone())
            .await?;
        let after = area_snapshot(self.reads.as_ref(), &info.cortical_id).await;
        self.ledger.record(
            GenomeChangeOperation::UpdateCorticalArea {
                cortical_id: cortical_id.to_string(),
                changes,
            },
            before,
            after,
            &current_change_context(),
        );
        Ok(info)
    }

    async fn create_cortical_areas(
        &self,
        params: Vec<CreateCorticalAreaParams>,
    ) -> ServiceResult<Vec<CorticalAreaInfo>> {
        let created = self.inner.create_cortical_areas(params.clone()).await?;
        let ids: Vec<&str> = created.iter().map(|a| a.cortical_id.as_str()).collect();
        let after = areas_snapshot(self.reads.as_ref(), &ids).await;
        self.ledger.record(
            GenomeChangeOperation::CreateCorticalAreas { params },
            None,
            after,
            &current_change_context(),
        );
        Ok(created)
    }

    async fn list_modulator_types(&self) -> ServiceResult<Vec<String>> {
        self.inner.list_modulator_types().await
    }

    async fn list_modulators(&self) -> ServiceResult<serde_json::Value> {
        self.inner.list_modulators().await
    }

    async fn get_modulator(&self, id: &str) -> ServiceResult<serde_json::Value> {
        self.inner.get_modulator(id).await
    }

    async fn create_modulator(
        &self,
        id: String,
        write: feagi_evolutionary::ModulatorWrite,
    ) -> ServiceResult<serde_json::Value> {
        self.inner.create_modulator(id, write).await
    }

    async fn update_modulator(
        &self,
        id: &str,
        write: feagi_evolutionary::ModulatorWrite,
    ) -> ServiceResult<serde_json::Value> {
        self.inner.update_modulator(id, write).await
    }

    async fn rename_modulator(&self, old_id: &str, new_id: String) -> ServiceResult<()> {
        self.inner.rename_modulator(old_id, new_id).await
    }

    async fn modulator_usage(&self, id: &str) -> ServiceResult<serde_json::Value> {
        self.inner.modulator_usage(id).await
    }

    async fn delete_modulator(&self, id: &str, force: bool) -> ServiceResult<()> {
        self.inner.delete_modulator(id, force).await
    }
}
