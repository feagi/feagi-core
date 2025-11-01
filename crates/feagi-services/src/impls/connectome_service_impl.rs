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
use std::collections::HashMap;
use tracing::{info, debug, warn};

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
        info!(target: "feagi-services","Creating cortical area: {}", params.cortical_id);
        
        // Convert string to AreaType
        let area_type = Self::string_to_area_type(&params.area_type)?;
        
        // Create CorticalArea (cortical_idx=0 will be auto-assigned by ConnectomeManager)
        let mut area = CorticalArea::new(
            params.cortical_id.clone(),
            0,  // Auto-assigned by ConnectomeManager
            params.name.clone(),
            Dimensions::new(params.dimensions.0, params.dimensions.1, params.dimensions.2),
            params.position,
            area_type,
        ).map_err(ServiceError::from)?;
        
        // Apply all neural parameters from params (direct field assignment)
        if let Some(visible) = params.visible {
            area.visible = visible;
        }
        if let Some(sub_group) = params.sub_group {
            area.sub_group = Some(sub_group);
        }
        if let Some(neurons_per_voxel) = params.neurons_per_voxel {
            area.neurons_per_voxel = neurons_per_voxel;
        }
        if let Some(postsynaptic_current) = params.postsynaptic_current {
            area.postsynaptic_current = postsynaptic_current;
        }
        if let Some(plasticity_constant) = params.plasticity_constant {
            area.plasticity_constant = plasticity_constant;
        }
        if let Some(degeneration) = params.degeneration {
            area.degeneration = degeneration;
        }
        if let Some(psp_uniform_distribution) = params.psp_uniform_distribution {
            area.psp_uniform_distribution = psp_uniform_distribution;
        }
        if let Some(firing_threshold_increment) = params.firing_threshold_increment {
            area.firing_threshold_increment = firing_threshold_increment;
        }
        if let Some(firing_threshold_limit) = params.firing_threshold_limit {
            area.firing_threshold_limit = firing_threshold_limit;
        }
        if let Some(consecutive_fire_count) = params.consecutive_fire_count {
            area.consecutive_fire_count = consecutive_fire_count;
        }
        if let Some(snooze_period) = params.snooze_period {
            area.snooze_period = snooze_period;
        }
        if let Some(refractory_period) = params.refractory_period {
            area.refractory_period = refractory_period;
        }
        if let Some(leak_coefficient) = params.leak_coefficient {
            area.leak_coefficient = leak_coefficient;
        }
        if let Some(leak_variability) = params.leak_variability {
            area.leak_variability = leak_variability;
        }
        if let Some(burst_engine_active) = params.burst_engine_active {
            area.burst_engine_active = burst_engine_active;
        }
        if let Some(properties) = params.properties {
            area.properties = properties;
        }
        
        // Add to connectome
        self.connectome
            .write()
            .add_cortical_area(area)
            .map_err(ServiceError::from)?;
        
        // Return info
        self.get_cortical_area(&params.cortical_id).await
    }

    async fn delete_cortical_area(&self, cortical_id: &str) -> ServiceResult<()> {
        info!(target: "feagi-services","Deleting cortical area: {}", cortical_id);
        
        self.connectome
            .write()
            .remove_cortical_area(cortical_id)
            .map_err(ServiceError::from)?;
        
        Ok(())
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services","Updating cortical area: {}", cortical_id);
        
        // Get mutable access to the cortical area
        {
            let mut manager = self.connectome.write();
            let area = manager
                .get_cortical_area_mut(cortical_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;
            
            // Update only the fields that are provided
            if let Some(name) = params.name {
                area.name = name;
            }
            if let Some(position) = params.position {
                area.position = position;
            }
            if let Some(dimensions) = params.dimensions {
                area.dimensions = Dimensions::new(dimensions.0, dimensions.1, dimensions.2);
            }
            if let Some(area_type_str) = params.area_type {
                area.area_type = Self::string_to_area_type(&area_type_str)?;
            }
            if let Some(visible) = params.visible {
                area.visible = visible;
            }
            if let Some(postsynaptic_current) = params.postsynaptic_current {
                area.postsynaptic_current = postsynaptic_current;
            }
            if let Some(plasticity_constant) = params.plasticity_constant {
                area.plasticity_constant = plasticity_constant;
            }
            if let Some(degeneration) = params.degeneration {
                area.degeneration = degeneration;
            }
            if let Some(psp_uniform_distribution) = params.psp_uniform_distribution {
                area.psp_uniform_distribution = psp_uniform_distribution;
            }
            if let Some(firing_threshold_increment) = params.firing_threshold_increment {
                area.firing_threshold_increment = firing_threshold_increment;
            }
            if let Some(firing_threshold_limit) = params.firing_threshold_limit {
                area.firing_threshold_limit = firing_threshold_limit;
            }
            if let Some(consecutive_fire_count) = params.consecutive_fire_count {
                area.consecutive_fire_count = consecutive_fire_count;
            }
            if let Some(snooze_period) = params.snooze_period {
                area.snooze_period = snooze_period;
            }
            if let Some(refractory_period) = params.refractory_period {
                area.refractory_period = refractory_period;
            }
            if let Some(leak_coefficient) = params.leak_coefficient {
                area.leak_coefficient = leak_coefficient;
            }
            if let Some(leak_variability) = params.leak_variability {
                area.leak_variability = leak_variability;
            }
            if let Some(burst_engine_active) = params.burst_engine_active {
                area.burst_engine_active = burst_engine_active;
            }
        } // Release write lock
        
        // Return updated area info
        self.get_cortical_area(cortical_id).await
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        debug!(target: "feagi-services","Getting cortical area: {}", cortical_id);
        
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
        let synapse_count = manager.get_synapse_count_in_area(cortical_id);
        
        // Derive cortical_group from area_type (matching Python logic)
        // Python logic: cortical_group = str(raw_group).upper() if raw_group else "CUSTOM"
        // Maps: Sensory → IPU, Motor → OPU, Memory → MEMORY, Custom → CUSTOM
        let cortical_group = match area.area_type {
            feagi_types::AreaType::Sensory => "IPU",
            feagi_types::AreaType::Motor => "OPU",
            feagi_types::AreaType::Memory => "MEMORY",
            feagi_types::AreaType::Custom => "CUSTOM",
        }.to_string();
        
        Ok(CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_idx,
            name: area.name.clone(),
            dimensions: area.dimensions.to_tuple(),
            position: area.position,
            area_type: Self::area_type_to_string(&area.area_type),
            cortical_group,
            neuron_count,
            synapse_count,
            // All neural parameters come from the actual CorticalArea struct
            visible: area.visible,
            sub_group: area.sub_group.clone(),
            neurons_per_voxel: area.neurons_per_voxel,
            postsynaptic_current: area.postsynaptic_current,
            plasticity_constant: area.plasticity_constant,
            degeneration: area.degeneration,
            psp_uniform_distribution: area.psp_uniform_distribution,
            firing_threshold_increment: area.firing_threshold_increment,
            firing_threshold_limit: area.firing_threshold_limit,
            consecutive_fire_count: area.consecutive_fire_count,
            snooze_period: area.snooze_period,
            refractory_period: area.refractory_period,
            leak_coefficient: area.leak_coefficient,
            leak_variability: area.leak_variability,
            burst_engine_active: area.burst_engine_active,
            properties: area.properties.clone(),
        })
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        debug!(target: "feagi-services","Listing all cortical areas");
        
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
        debug!(target: "feagi-services","Getting cortical area IDs");
        
        // CRITICAL: Use try_read() instead of read() to avoid blocking forever
        // If write lock is held (e.g., during genome loading), return error instead of hanging
        let ids: Vec<String> = {
            let manager = match self.connectome.try_read() {
                Some(guard) => guard,
                None => {
                    warn!(target: "feagi-services", "⚠️ ConnectomeManager write lock is held - cannot read cortical area IDs");
                    return Err(ServiceError::Backend("ConnectomeManager is currently being modified (e.g., genome loading in progress). Please try again in a moment.".to_string()));
                }
            };
            
            let area_count = manager.get_cortical_area_count();
            let ids_refs = manager.get_cortical_area_ids();
            info!(target: "feagi-services", "Found {} cortical areas in ConnectomeManager", area_count);
            info!(target: "feagi-services", "Cortical area IDs (references): {:?}", ids_refs.iter().take(10).collect::<Vec<_>>());
            ids_refs.into_iter().cloned().collect()
        }; // Lock dropped here
        info!(target: "feagi-services", "Returning {} cortical area IDs: {:?}", ids.len(), ids.iter().take(10).collect::<Vec<_>>());
        Ok(ids)
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if cortical area exists: {}", cortical_id);
        Ok(self.connectome.read().has_cortical_area(cortical_id))
    }

    // ========================================================================
    // BRAIN REGION OPERATIONS
    // ========================================================================

    async fn create_brain_region(
        &self,
        params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        info!(target: "feagi-services","Creating brain region: {}", params.region_id);
        
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
        info!(target: "feagi-services","Deleting brain region: {}", region_id);
        
        self.connectome
            .write()
            .remove_brain_region(region_id)
            .map_err(ServiceError::from)?;
        
        Ok(())
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        debug!(target: "feagi-services","Getting brain region: {}", region_id);
        
        let manager = self.connectome.read();
        
        let region = manager
            .get_brain_region(region_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "BrainRegion".to_string(),
                id: region_id.to_string(),
            })?;
        
        let hierarchy = manager.get_brain_region_hierarchy();
        let parent_id = hierarchy.get_parent(region_id).map(|s| s.to_string());
        let child_regions: Vec<String> = hierarchy
            .get_children(region_id)
            .into_iter()
            .cloned()
            .collect();
        
        Ok(BrainRegionInfo {
            region_id: region_id.to_string(),
            name: region.name.clone(),
            region_type: Self::region_type_to_string(&region.region_type),
            parent_id,
            cortical_areas: region.cortical_areas.iter().cloned().collect(),
            child_regions,
            properties: region.properties.clone(),
        })
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        debug!(target: "feagi-services","Listing all brain regions");
        
        let region_ids: Vec<String> = {
            let manager = self.connectome.read();
            let ids = manager.get_brain_region_ids();
            debug!(target: "feagi-services","  Found {} brain region IDs from ConnectomeManager", ids.len());
            ids.into_iter().map(|s| s.to_string()).collect()
        };
        
        debug!(target: "feagi-services","  Processing {} regions...", region_ids.len());
        let mut regions = Vec::new();
        for region_id in region_ids {
            debug!(target: "feagi-services","    Getting region: {}", region_id);
            match self.get_brain_region(&region_id).await {
                Ok(region_info) => {
                    debug!(target: "feagi-services","      ✓ Got region: {} with {} areas", region_info.name, region_info.cortical_areas.len());
                    regions.push(region_info);
                }
                Err(e) => {
                    warn!(target: "feagi-services","      ✗ Failed to get region {}: {}", region_id, e);
                }
            }
        }
        
        debug!(target: "feagi-services","  Returning {} brain regions", regions.len());
        Ok(regions)
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        debug!(target: "feagi-services","Getting brain region IDs");
        Ok(self.connectome.read().get_brain_region_ids().into_iter().map(|s| s.to_string()).collect())
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if brain region exists: {}", region_id);
        Ok(self.connectome.read().get_brain_region(region_id).is_some())
    }
    
    async fn get_morphologies(&self) -> ServiceResult<HashMap<String, MorphologyInfo>> {
        let manager = self.connectome.read();
        let registry = manager.get_morphologies();
        
        let mut result = HashMap::new();
        for (id, morphology) in registry.iter() {
            result.insert(
                id.clone(),
                MorphologyInfo {
                    morphology_type: format!("{:?}", morphology.morphology_type).to_lowercase(),
                    class: morphology.class.clone(),
                    parameters: serde_json::to_value(&morphology.parameters)
                        .unwrap_or(serde_json::json!({})),
                }
            );
        }
        
        debug!(target: "feagi-services", "Retrieved {} morphologies", result.len());
        Ok(result)
    }
    
    async fn update_cortical_mapping(
        &self,
        src_area_id: String,
        dst_area_id: String,
        mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize> {
        info!(target: "feagi-services", "Updating cortical mapping: {} -> {} with {} connections",
              src_area_id, dst_area_id, mapping_data.len());
        
        // Update the cortical_mapping_dst property in ConnectomeManager
        let mut manager = self.connectome.write();
        manager.update_cortical_mapping(&src_area_id, &dst_area_id, mapping_data.clone())
            .map_err(|e| ServiceError::Backend(format!("Failed to update mapping: {}", e)))?;
        
        // Regenerate synapses for this mapping
        let synapse_count = manager.regenerate_synapses_for_mapping(&src_area_id, &dst_area_id)
            .map_err(|e| ServiceError::Backend(format!("Failed to regenerate synapses: {}", e)))?;
        
        info!(target: "feagi-services", "Cortical mapping updated: {} synapses created", synapse_count);
        Ok(synapse_count)
    }
}

