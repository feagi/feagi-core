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
        
        // Delegate to ConnectomeManager
        self.connectome
            .write()
            .load_genome_from_json(&params.json_str)
            .map_err(ServiceError::from)?;
        
        // Return genome info
        self.get_genome_info().await
    }

    async fn save_genome(&self, params: SaveGenomeParams) -> ServiceResult<String> {
        log::info!("Saving genome to JSON");
        
        // Delegate to ConnectomeManager
        let json_str = self.connectome
            .read()
            .save_genome_to_json(params.genome_id, params.genome_title)
            .map_err(ServiceError::from)?;
        
        Ok(json_str)
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        log::debug!("Getting genome info");
        
        let manager = self.connectome.read();
        let cortical_area_count = manager.get_cortical_area_count();
        let brain_region_count = manager.get_brain_region_ids().len();
        
        Ok(GenomeInfo {
            genome_id: "current".to_string(),  // TODO: Track genome_id
            genome_title: "Current Genome".to_string(),
            version: "2.1".to_string(),
            cortical_area_count,
            brain_region_count,
        })
    }

    async fn validate_genome(&self, json_str: String) -> ServiceResult<bool> {
        log::debug!("Validating genome JSON");
        
        // Try to parse genome without loading it
        match feagi_evo::GenomeParser::parse(&json_str) {
            Ok(_) => Ok(true),
            Err(e) => Err(ServiceError::InvalidInput(format!(
                "Invalid genome: {}",
                e
            ))),
        }
    }

    async fn reset_connectome(&self) -> ServiceResult<()> {
        log::info!("Resetting connectome");
        
        // TODO: Implement proper connectome reset
        // For now, remove all cortical areas and brain regions manually
        let cortical_ids: Vec<String> = self.connectome.read().get_cortical_area_ids().into_iter().cloned().collect();
        for cortical_id in cortical_ids {
            let _ = self.connectome.write().remove_cortical_area(&cortical_id);
        }
        
        let region_ids: Vec<String> = self.connectome.read().get_brain_region_ids()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        for region_id in region_ids {
            let _ = self.connectome.write().remove_brain_region(&region_id);
        }
        
        Ok(())
    }
}

