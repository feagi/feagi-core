/*!
Connectome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::ConnectomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use feagi_types::{AreaType, BrainRegion, CorticalArea, Dimensions, RegionType};
use parking_lot::RwLock;
use std::sync::Arc;

/// Default implementation of ConnectomeService
pub struct ConnectomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
}

impl ConnectomeServiceImpl {
    pub fn new(connectome: Arc<RwLock<ConnectomeManager>>) -> Self {
        Self { connectome }
    }
    
    /// Convert AreaType enum to string
    fn area_type_to_string(area_type: &AreaType) -> String {
        match area_type {
            AreaType::Sensory => "Sensory".to_string(),
            AreaType::Motor => "Motor".to_string(),
            AreaType::Memory => "Memory".to_string(),
            AreaType::Custom => "Custom".to_string(),
        }
    }
    
    /// Convert string to AreaType enum
    fn string_to_area_type(s: &str) -> Result<AreaType, ServiceError> {
        match s {
            "Sensory" => Ok(AreaType::Sensory),
            "Motor" => Ok(AreaType::Motor),
            "Memory" => Ok(AreaType::Memory),
            "Custom" => Ok(AreaType::Custom),
            _ => Err(ServiceError::InvalidInput(format!(
                "Invalid area type: {}",
                s
            ))),
        }
    }
    
    /// Convert RegionType enum to string
    fn region_type_to_string(region_type: &RegionType) -> String {
        match region_type {
            RegionType::Sensory => "Sensory".to_string(),
            RegionType::Motor => "Motor".to_string(),
            RegionType::Memory => "Memory".to_string(),
            RegionType::Custom => "Custom".to_string(),
        }
    }
    
    /// Convert string to RegionType enum
    fn string_to_region_type(s: &str) -> Result<RegionType, ServiceError> {
        match s {
            "Sensory" => Ok(RegionType::Sensory),
            "Motor" => Ok(RegionType::Motor),
            "Memory" => Ok(RegionType::Memory),
            "Custom" => Ok(RegionType::Custom),
            _ => Err(ServiceError::InvalidInput(format!(
                "Invalid region type: {}",
                s
            ))),
        }
    }
}

#[async_trait]
impl ConnectomeService for ConnectomeServiceImpl {
    // ========================================================================
    // CORTICAL AREA OPERATIONS
    // ========================================================================

    async fn create_cortical_area(
        &self,
        params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        log::info!("Creating cortical area: {}", params.cortical_id);
        
        // Convert string to AreaType
        let area_type = Self::string_to_area_type(&params.area_type)?;
        
        // Create CorticalArea (cortical_idx=0 will be auto-assigned by ConnectomeManager)
        let area = CorticalArea::new(
            params.cortical_id.clone(),
            0,  // Auto-assigned by ConnectomeManager
            params.name.clone(),
            Dimensions::new(params.dimensions.0, params.dimensions.1, params.dimensions.2),
            params.position,
            area_type,
        ).map_err(ServiceError::from)?;
        
        // Add to connectome
        self.connectome
            .write()
            .add_cortical_area(area)
            .map_err(ServiceError::from)?;
        
        // Return info
        self.get_cortical_area(&params.cortical_id).await
    }

    async fn delete_cortical_area(&self, cortical_id: &str) -> ServiceResult<()> {
        log::info!("Deleting cortical area: {}", cortical_id);
        
        self.connectome
            .write()
            .remove_cortical_area(cortical_id)
            .map_err(ServiceError::from)?;
        
        Ok(())
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        log::debug!("Getting cortical area: {}", cortical_id);
        
        let manager = self.connectome.read();
        
        let area = manager
            .get_cortical_area(cortical_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let cortical_idx = manager
            .get_cortical_idx(cortical_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let neuron_count = manager.get_neuron_count_in_area(cortical_id);
        
        Ok(CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_idx,
            name: area.name.clone(),
            dimensions: area.dimensions.to_tuple(),
            position: area.position,
            area_type: Self::area_type_to_string(&area.area_type),
            neuron_count,
            properties: area.properties.clone(),
        })
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        log::debug!("Listing all cortical areas");
        
        let cortical_ids: Vec<String> = {
            let manager = self.connectome.read();
            manager.get_cortical_area_ids().into_iter().cloned().collect()
        };
        
        let mut areas = Vec::new();
        for cortical_id in cortical_ids {
            if let Ok(area_info) = self.get_cortical_area(&cortical_id).await {
                areas.push(area_info);
            }
        }
        
        Ok(areas)
    }

    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>> {
        log::debug!("Getting cortical area IDs");
        Ok(self.connectome.read().get_cortical_area_ids().into_iter().cloned().collect())
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        log::debug!("Checking if cortical area exists: {}", cortical_id);
        Ok(self.connectome.read().has_cortical_area(cortical_id))
    }

    // ========================================================================
    // BRAIN REGION OPERATIONS
    // ========================================================================

    async fn create_brain_region(
        &self,
        params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        log::info!("Creating brain region: {}", params.region_id);
        
        // Convert string to RegionType
        let region_type = Self::string_to_region_type(&params.region_type)?;
        
        // Create BrainRegion
        let region = BrainRegion::new(
            params.region_id.clone(),
            params.name.clone(),
            region_type,
        ).map_err(ServiceError::from)?;
        
        // Add to connectome
        self.connectome
            .write()
            .add_brain_region(region, params.parent_id.clone())
            .map_err(ServiceError::from)?;
        
        // Return info
        self.get_brain_region(&params.region_id).await
    }

    async fn delete_brain_region(&self, region_id: &str) -> ServiceResult<()> {
        log::info!("Deleting brain region: {}", region_id);
        
        self.connectome
            .write()
            .remove_brain_region(region_id)
            .map_err(ServiceError::from)?;
        
        Ok(())
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        log::debug!("Getting brain region: {}", region_id);
        
        let manager = self.connectome.read();
        
        let region = manager
            .get_brain_region(region_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "BrainRegion".to_string(),
                id: region_id.to_string(),
            })?;
        
        let hierarchy = manager.get_brain_region_hierarchy();
        let parent_id = hierarchy.get_parent(region_id).map(|s| s.to_string());
        
        Ok(BrainRegionInfo {
            region_id: region_id.to_string(),
            name: region.name.clone(),
            region_type: Self::region_type_to_string(&region.region_type),
            parent_id,
            cortical_areas: region.cortical_areas.iter().cloned().collect(),
            properties: region.properties.clone(),
        })
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        log::debug!("Listing all brain regions");
        
        let region_ids: Vec<String> = {
            let manager = self.connectome.read();
            manager.get_brain_region_ids().into_iter().map(|s| s.to_string()).collect()
        };
        
        let mut regions = Vec::new();
        for region_id in region_ids {
            if let Ok(region_info) = self.get_brain_region(&region_id).await {
                regions.push(region_info);
            }
        }
        
        Ok(regions)
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        log::debug!("Getting brain region IDs");
        Ok(self.connectome.read().get_brain_region_ids().into_iter().map(|s| s.to_string()).collect())
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        log::debug!("Checking if brain region exists: {}", region_id);
        Ok(self.connectome.read().get_brain_region(region_id).is_some())
    }
}

