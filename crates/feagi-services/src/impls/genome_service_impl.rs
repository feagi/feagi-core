/*!
Genome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::GenomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use parking_lot::RwLock;
use std::sync::Arc;

/// Default implementation of GenomeService
pub struct GenomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
}

impl GenomeServiceImpl {
    pub fn new(connectome: Arc<RwLock<ConnectomeManager>>) -> Self {
        Self { connectome }
    }
}

#[async_trait]
impl GenomeService for GenomeServiceImpl {
    async fn load_genome(&self, params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        log::info!("Loading genome from JSON");
        
        // Parse genome using feagi-evo (this is CPU-bound, but relatively fast)
        let genome = feagi_evo::load_genome_from_json(&params.json_str)
            .map_err(|e| ServiceError::InvalidInput(format!("Failed to parse genome: {}", e)))?;
        
        // Load into connectome via ConnectomeManager
        // This involves synaptogenesis which can be CPU-intensive, so run it on a blocking thread
        // CRITICAL: Add timeout to prevent hanging during shutdown
        // Note: spawn_blocking tasks cannot be cancelled, but timeout ensures we don't wait forever
        let connectome_clone = self.connectome.clone();
        let blocking_handle = tokio::task::spawn_blocking(move || {
            connectome_clone
                .write()
                .load_from_genome(genome)
                .map_err(ServiceError::from)
        });
        
        // Wait with timeout - if timeout expires, abort the blocking task
        let progress = match tokio::time::timeout(
            tokio::time::Duration::from_secs(300), // 5 minute timeout
            blocking_handle
        ).await {
            Ok(Ok(result)) => result?,
            Ok(Err(e)) => return Err(ServiceError::Backend(format!("Blocking task panicked: {}", e))),
            Err(_) => {
                // Timeout expired - abort the task (though it may continue running)
                log::warn!("Genome loading timed out after 5 minutes - aborting");
                return Err(ServiceError::Backend("Genome loading timed out after 5 minutes".to_string()));
            }
        };
        
        log::info!(
            "Genome loaded: {} cortical areas, {} neurons, {} synapses created",
            progress.cortical_areas_created,
            progress.neurons_created,
            progress.synapses_created
        );
        
        // Return genome info
        self.get_genome_info().await
    }

    async fn save_genome(&self, params: SaveGenomeParams) -> ServiceResult<String> {
        log::info!("Saving genome to JSON");
        
        // Delegate to ConnectomeManager
        let json_str = self.connectome
            .read()
            .save_genome_to_json(
                params.genome_id,
                params.genome_title,
            )
            .map_err(ServiceError::from)?;
        
        Ok(json_str)
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        log::debug!("Getting genome info");
        
        let manager = self.connectome.read();
        let cortical_area_count = manager.get_cortical_area_count();
        let brain_region_count = manager.get_brain_region_ids().len();
        
        Ok(GenomeInfo {
            genome_id: "current".to_string(),  // TODO: Track genome_id in ConnectomeManager
            genome_title: "Current Genome".to_string(),
            version: "2.1".to_string(),
            cortical_area_count,
            brain_region_count,
        })
    }

    async fn validate_genome(&self, json_str: String) -> ServiceResult<bool> {
        log::debug!("Validating genome JSON");
        
        // Parse genome
        let genome = feagi_evo::load_genome_from_json(&json_str)
            .map_err(|e| ServiceError::InvalidInput(format!("Failed to parse genome: {}", e)))?;
        
        // Validate genome structure
        let validation = feagi_evo::validate_genome(&genome);
        
        if !validation.errors.is_empty() {
            return Err(ServiceError::InvalidInput(format!(
                "Genome validation failed: {} errors, {} warnings. First error: {}",
                validation.errors.len(),
                validation.warnings.len(),
                validation.errors.first().unwrap_or(&"Unknown error".to_string())
            )));
        }
        
        Ok(true)
    }

    async fn reset_connectome(&self) -> ServiceResult<()> {
        log::info!("Resetting connectome");
        
        // Use ConnectomeManager's prepare_for_new_genome method
        self.connectome
            .write()
            .prepare_for_new_genome()
            .map_err(ServiceError::from)?;
        
        log::info!("Connectome reset complete");
        Ok(())
    }
}

