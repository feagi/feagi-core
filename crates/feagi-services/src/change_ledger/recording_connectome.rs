// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! [`ConnectomeService`] decorator that records successful structural mutations in
//! the change ledger.
//!
//! `create_cortical_area` and `update_cortical_area` on this trait are internal or
//! blocked paths (genome-level changes go through `GenomeService`), so they pass
//! through unrecorded.

use super::context::current_change_context;
use super::ledger::ChangeLedger;
use super::snapshot::{
    area_with_incoming_snapshot, classifier_snapshot, mapping_rules, morphology_snapshot,
    region_snapshot,
};
use super::types::GenomeChangeOperation;
use crate::traits::ConnectomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_structures::genomic::classifiers::Classifier;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Wraps a connectome service; reads and failed calls pass through unrecorded.
pub struct RecordingConnectomeService {
    inner: Arc<dyn ConnectomeService + Send + Sync>,
    ledger: Arc<ChangeLedger>,
}

impl RecordingConnectomeService {
    pub fn new(inner: Arc<dyn ConnectomeService + Send + Sync>, ledger: Arc<ChangeLedger>) -> Self {
        Self { inner, ledger }
    }

    fn record(
        &self,
        operation: GenomeChangeOperation,
        before: Option<Value>,
        after: Option<Value>,
    ) {
        self.ledger
            .record(operation, before, after, &current_change_context());
    }
}

#[async_trait]
impl ConnectomeService for RecordingConnectomeService {
    async fn create_cortical_area(
        &self,
        params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        self.inner.create_cortical_area(params).await
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        self.inner.update_cortical_area(cortical_id, params).await
    }

    async fn delete_cortical_area(&self, cortical_id: &str) -> ServiceResult<()> {
        let before = area_with_incoming_snapshot(self.inner.as_ref(), cortical_id).await;
        self.inner.delete_cortical_area(cortical_id).await?;
        self.record(
            GenomeChangeOperation::DeleteCorticalArea {
                cortical_id: cortical_id.to_string(),
            },
            before,
            None,
        );
        Ok(())
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        self.inner.get_cortical_area(cortical_id).await
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        self.inner.list_cortical_areas().await
    }

    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>> {
        self.inner.get_cortical_area_ids().await
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        self.inner.cortical_area_exists(cortical_id).await
    }

    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<HashMap<String, Value>> {
        self.inner.get_cortical_area_properties(cortical_id).await
    }

    async fn get_all_cortical_area_properties(&self) -> ServiceResult<Vec<HashMap<String, Value>>> {
        self.inner.get_all_cortical_area_properties().await
    }

    async fn get_neuron_properties(&self, neuron_id: u64) -> ServiceResult<HashMap<String, Value>> {
        self.inner.get_neuron_properties(neuron_id).await
    }

    async fn create_brain_region(
        &self,
        params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        let info = self.inner.create_brain_region(params.clone()).await?;
        let after = region_snapshot(self.inner.as_ref(), &info.region_id).await;
        self.record(
            GenomeChangeOperation::CreateBrainRegion {
                params: Box::new(params),
            },
            None,
            after,
        );
        Ok(info)
    }

    async fn delete_brain_region(&self, region_id: &str) -> ServiceResult<()> {
        let before = region_snapshot(self.inner.as_ref(), region_id).await;
        self.inner.delete_brain_region(region_id).await?;
        self.record(
            GenomeChangeOperation::DeleteBrainRegion {
                region_id: region_id.to_string(),
            },
            before,
            None,
        );
        Ok(())
    }

    async fn update_brain_region(
        &self,
        region_id: &str,
        properties: HashMap<String, Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        let before = region_snapshot(self.inner.as_ref(), region_id).await;
        let info = self
            .inner
            .update_brain_region(region_id, properties.clone())
            .await?;
        let after = region_snapshot(self.inner.as_ref(), &info.region_id).await;
        self.record(
            GenomeChangeOperation::UpdateBrainRegion {
                region_id: region_id.to_string(),
                properties,
            },
            before,
            after,
        );
        Ok(info)
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        self.inner.get_brain_region(region_id).await
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        self.inner.list_brain_regions().await
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        self.inner.get_brain_region_ids().await
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        self.inner.brain_region_exists(region_id).await
    }

    async fn get_root_region_id(&self) -> ServiceResult<Option<String>> {
        self.inner.get_root_region_id().await
    }

    async fn get_morphologies(&self) -> ServiceResult<HashMap<String, MorphologyInfo>> {
        self.inner.get_morphologies().await
    }

    async fn create_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        self.inner
            .create_morphology(morphology_id.clone(), morphology.clone())
            .await?;
        let after = morphology_snapshot(self.inner.as_ref(), &morphology_id).await;
        self.record(
            GenomeChangeOperation::CreateMorphology {
                morphology_id,
                morphology: Box::new(morphology),
            },
            None,
            after,
        );
        Ok(())
    }

    async fn update_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        let before = morphology_snapshot(self.inner.as_ref(), &morphology_id).await;
        self.inner
            .update_morphology(morphology_id.clone(), morphology.clone())
            .await?;
        let after = morphology_snapshot(self.inner.as_ref(), &morphology_id).await;
        self.record(
            GenomeChangeOperation::UpdateMorphology {
                morphology_id,
                morphology: Box::new(morphology),
            },
            before,
            after,
        );
        Ok(())
    }

    async fn delete_morphology(&self, morphology_id: &str) -> ServiceResult<()> {
        let before = morphology_snapshot(self.inner.as_ref(), morphology_id).await;
        self.inner.delete_morphology(morphology_id).await?;
        self.record(
            GenomeChangeOperation::DeleteMorphology {
                morphology_id: morphology_id.to_string(),
            },
            before,
            None,
        );
        Ok(())
    }

    async fn rename_morphology(&self, old_id: &str, new_id: &str) -> ServiceResult<()> {
        self.inner.rename_morphology(old_id, new_id).await?;
        self.record(
            GenomeChangeOperation::RenameMorphology {
                old_id: old_id.to_string(),
                new_id: new_id.to_string(),
            },
            None,
            None,
        );
        Ok(())
    }

    async fn update_cortical_mapping(
        &self,
        src_area_id: String,
        dst_area_id: String,
        mapping_data: Vec<Value>,
    ) -> ServiceResult<usize> {
        let before = mapping_rules(self.inner.as_ref(), &src_area_id, &dst_area_id).await;
        let synapses = self
            .inner
            .update_cortical_mapping(
                src_area_id.clone(),
                dst_area_id.clone(),
                mapping_data.clone(),
            )
            .await?;
        let after = mapping_rules(self.inner.as_ref(), &src_area_id, &dst_area_id).await;
        self.record(
            GenomeChangeOperation::UpdateCorticalMapping {
                src_area_id,
                dst_area_id,
                mapping_data,
            },
            before,
            after,
        );
        Ok(synapses)
    }

    #[cfg(feature = "connectome-io")]
    async fn export_connectome(
        &self,
        mode: feagi_npu_neural::types::connectome::ConnectomePersistMode,
    ) -> ServiceResult<feagi_npu_neural::types::connectome::ConnectomeSnapshot> {
        self.inner.export_connectome(mode).await
    }

    #[cfg(feature = "connectome-io")]
    async fn import_connectome(
        &self,
        snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot,
    ) -> ServiceResult<()> {
        self.inner.import_connectome(snapshot).await?;
        self.record(GenomeChangeOperation::ConnectomeImported, None, None);
        Ok(())
    }

    async fn upsert_classifier(&self, classifier: Classifier) -> ServiceResult<()> {
        let classifier_id = classifier.classifier_id.clone();
        let before = classifier_snapshot(self.inner.as_ref(), &classifier_id).await;
        self.inner.upsert_classifier(classifier.clone()).await?;
        let after = classifier_snapshot(self.inner.as_ref(), &classifier_id).await;
        self.record(
            GenomeChangeOperation::UpsertClassifier {
                classifier: Box::new(classifier),
            },
            before,
            after,
        );
        Ok(())
    }

    async fn list_classifiers(&self) -> ServiceResult<Vec<ClassifierInfo>> {
        self.inner.list_classifiers().await
    }

    async fn get_classifier(&self, classifier_id: &str) -> ServiceResult<ClassifierInfo> {
        self.inner.get_classifier(classifier_id).await
    }

    async fn rekey_memory_twin_source(
        &self,
        memory_area_id: &str,
        old_src_area_id: &str,
        new_src_area_id: &str,
    ) -> ServiceResult<()> {
        self.inner
            .rekey_memory_twin_source(memory_area_id, old_src_area_id, new_src_area_id)
            .await?;
        self.record(
            GenomeChangeOperation::RekeyMemoryTwinSource {
                memory_area_id: memory_area_id.to_string(),
                old_src_area_id: old_src_area_id.to_string(),
                new_src_area_id: new_src_area_id.to_string(),
            },
            None,
            None,
        );
        Ok(())
    }

    async fn delete_classifier(&self, classifier_id: &str) -> ServiceResult<()> {
        let before = classifier_snapshot(self.inner.as_ref(), classifier_id).await;
        self.inner.delete_classifier(classifier_id).await?;
        self.record(
            GenomeChangeOperation::DeleteClassifier {
                classifier_id: classifier_id.to_string(),
            },
            before,
            None,
        );
        Ok(())
    }
}
