// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Connectome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::ConnectomeService;
use crate::types::*;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use feagi_data_structures::genomic::brain_regions::{BrainRegion, RegionType, RegionID};
use feagi_data_structures::genomic::cortical_area::{CorticalArea, CorticalAreaDimensions};
use feagi_bdu::models::CorticalAreaExt;
// Note: decode_cortical_id removed - use feagi_data_structures::CorticalID directly
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
    
    /// Convert RegionType enum to string
    fn region_type_to_string(region_type: &RegionType) -> String {
        match region_type {
            RegionType::Undefined => "Undefined".to_string(),
        }
    }
    
    /// Convert string to RegionType enum
    fn string_to_region_type(s: &str) -> Result<RegionType, ServiceError> {
        match s {
            "Undefined" | "Sensory" | "Motor" | "Memory" | "Custom" => Ok(RegionType::Undefined),
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
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(&params.cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        // Get cortical area type from the cortical ID
        let area_type = cortical_id_typed.as_cortical_type()
            .map_err(|e| ServiceError::InvalidInput(format!("Failed to determine cortical area type: {}", e)))?;
        
        // Create CorticalArea
        let mut area = CorticalArea::new(
            cortical_id_typed,
            0,  // Auto-assigned by ConnectomeManager
            params.name.clone(),
            CorticalAreaDimensions::new(params.dimensions.0 as u32, params.dimensions.1 as u32, params.dimensions.2 as u32)?,
            params.position.into(),  // Convert (i32, i32, i32) to GenomeCoordinate3D
            area_type,
        )?;
        
        // Set the cortical type
        // Note: cortical_type_new field removed - type is encoded in CorticalID
        
        // Apply all neural parameters from params
        if let Some(visible) = params.visible {
            area.add_property_mut("visible".to_string(), serde_json::json!(visible));
        }
        if let Some(sub_group) = params.sub_group {
            area.add_property_mut("sub_group".to_string(), serde_json::json!(sub_group));
        }
        if let Some(neurons_per_voxel) = params.neurons_per_voxel {
            area.add_property_mut("neurons_per_voxel".to_string(), serde_json::json!(neurons_per_voxel));
        }
        if let Some(postsynaptic_current) = params.postsynaptic_current {
            area.add_property_mut("postsynaptic_current".to_string(), serde_json::json!(postsynaptic_current));
        }
        if let Some(plasticity_constant) = params.plasticity_constant {
            area.add_property_mut("plasticity_constant".to_string(), serde_json::json!(plasticity_constant));
        }
        if let Some(degeneration) = params.degeneration {
            area.add_property_mut("degeneration".to_string(), serde_json::json!(degeneration));
        }
        if let Some(psp_uniform_distribution) = params.psp_uniform_distribution {
            area.add_property_mut("psp_uniform_distribution".to_string(), serde_json::json!(psp_uniform_distribution));
        }
        if let Some(firing_threshold_increment) = params.firing_threshold_increment {
            area.add_property_mut("firing_threshold_increment".to_string(), serde_json::json!(firing_threshold_increment));
        }
        if let Some(firing_threshold_limit) = params.firing_threshold_limit {
            area.add_property_mut("firing_threshold_limit".to_string(), serde_json::json!(firing_threshold_limit));
        }
        if let Some(consecutive_fire_count) = params.consecutive_fire_count {
            area.add_property_mut("consecutive_fire_limit".to_string(), serde_json::json!(consecutive_fire_count));
        }
        if let Some(snooze_period) = params.snooze_period {
            area.add_property_mut("snooze_period".to_string(), serde_json::json!(snooze_period));
        }
        if let Some(refractory_period) = params.refractory_period {
            area.add_property_mut("refractory_period".to_string(), serde_json::json!(refractory_period));
        }
        if let Some(leak_coefficient) = params.leak_coefficient {
            area.add_property_mut("leak_coefficient".to_string(), serde_json::json!(leak_coefficient));
        }
        if let Some(leak_variability) = params.leak_variability {
            area.add_property_mut("leak_variability".to_string(), serde_json::json!(leak_variability));
        }
        if let Some(burst_engine_active) = params.burst_engine_active {
            area.add_property_mut("burst_engine_active".to_string(), serde_json::json!(burst_engine_active));
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
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        self.connectome
            .write()
            .remove_cortical_area(&cortical_id_typed)
            .map_err(ServiceError::from)?;
        
        Ok(())
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        _params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services","Updating cortical area: {}", cortical_id);
        
        // TODO: This should be routed through GenomeService for proper genome update
        // and change classification (PARAMETER vs STRUCTURAL vs METADATA)
        // Currently this is a stub - needs architecture alignment with Python implementation
        
        Err(ServiceError::NotImplemented(
            "Cortical area updates must go through GenomeService for proper genome synchronization".to_string()
        ))
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        debug!(target: "feagi-services","Getting cortical area: {}", cortical_id);
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        let manager = self.connectome.read();
        
        let area = manager
            .get_cortical_area(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let cortical_idx = manager
            .get_cortical_idx(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let neuron_count = manager.get_neuron_count_in_area(&CorticalID::try_from_base_64(cortical_id).map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?);
        let synapse_count = manager.get_synapse_count_in_area(&CorticalID::try_from_base_64(cortical_id).map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?);
        
        // Get cortical_group from the area (uses cortical_type_new if available)
        let cortical_group = area.get_cortical_group();
        
        // Note: decode_cortical_id removed - IPU/OPU metadata now in CorticalID
        
        Ok(CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_id_s: area.cortical_id.to_string(), // Human-readable ASCII string
            cortical_idx,
            name: area.name.clone(),
            dimensions: (area.dimensions.width as usize, area.dimensions.height as usize, area.dimensions.depth as usize),
            position: area.position.into(),  // Convert GenomeCoordinate3D to (i32, i32, i32)
            area_type: cortical_group.clone().unwrap_or_else(|| "CUSTOM".to_string()),
            cortical_group: cortical_group.unwrap_or_else(|| "CUSTOM".to_string()),
            neuron_count,
            synapse_count,
            // All neural parameters come from the actual CorticalArea struct
            visible: area.visible(),
            sub_group: area.sub_group(),
            neurons_per_voxel: area.neurons_per_voxel(),
            postsynaptic_current: area.postsynaptic_current() as f64,
            plasticity_constant: area.plasticity_constant() as f64,
            degeneration: area.degeneration() as f64,
            psp_uniform_distribution: area.psp_uniform_distribution() != 0.0,
            firing_threshold_increment: area.firing_threshold_increment() as f64,
            firing_threshold_limit: area.firing_threshold_limit() as f64,
            consecutive_fire_count: area.consecutive_fire_count(),
            snooze_period: area.snooze_period() as u32,
            refractory_period: area.refractory_period() as u32,
            leak_coefficient: area.leak_coefficient() as f64,
            leak_variability: area.leak_variability() as f64,
            burst_engine_active: area.burst_engine_active(),
            properties: area.properties.clone(),
            // IPU/OPU-specific decoded fields (only populated for IPU/OPU areas)
            cortical_subtype: None, // Note: decode_cortical_id removed
            encoding_type: None,
            encoding_format: None,
            unit_id: None,
            group_id: None,
            parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
        })
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        debug!(target: "feagi-services","Listing all cortical areas");
        
        let cortical_ids: Vec<String> = {
            let manager = self.connectome.read();
            manager.get_cortical_area_ids().into_iter().map(|id| id.as_base_64()).collect()
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
            ids_refs.into_iter().map(|id| id.as_base_64()).collect()
        }; // Lock dropped here
        info!(target: "feagi-services", "Returning {} cortical area IDs: {:?}", ids.len(), ids.iter().take(10).collect::<Vec<_>>());
        Ok(ids)
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if cortical area exists: {}", cortical_id);
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        Ok(self.connectome.read().has_cortical_area(&cortical_id_typed))
    }

    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>> {
        debug!(target: "feagi-services","Getting cortical area properties: {}", cortical_id);
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        let manager = self.connectome.read();
        manager
            .get_cortical_area_properties(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })
    }

    async fn get_all_cortical_area_properties(
        &self,
    ) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        debug!(target: "feagi-services","Getting all cortical area properties");
        
        let manager = self.connectome.read();
        Ok(manager.get_all_cortical_area_properties())
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
            RegionID::from_string(&params.region_id).map_err(|e| ServiceError::InvalidInput(format!("Invalid region ID: {}", e)))?,
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

    async fn update_brain_region(
        &self,
        region_id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        info!(target: "feagi-services", "Updating brain region: {}", region_id);
        
        self.connectome
            .write()
            .update_brain_region_properties(region_id, properties)
            .map_err(ServiceError::from)?;
        
        // Return updated info
        self.get_brain_region(region_id).await
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
            cortical_areas: region.cortical_areas.iter().map(|id| id.as_base_64()).collect(),  // Use base64 to match cortical area API
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
        
        // Convert String to CorticalID
        use feagi_data_structures::genomic::cortical_area::CorticalID;
        let src_id = CorticalID::try_from_base_64(&src_area_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid source cortical ID: {}", e)))?;
        let dst_id = CorticalID::try_from_base_64(&dst_area_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid destination cortical ID: {}", e)))?;
        
        // Update the cortical_mapping_dst property in ConnectomeManager
        let mut manager = self.connectome.write();
        manager.update_cortical_mapping(&src_id, &dst_id, mapping_data.clone())
            .map_err(|e| ServiceError::Backend(format!("Failed to update mapping: {}", e)))?;
        
        // Regenerate synapses for this mapping
        let synapse_count = manager.regenerate_synapses_for_mapping(&src_id, &dst_id)
            .map_err(|e| ServiceError::Backend(format!("Failed to regenerate synapses: {}", e)))?;
        
        info!(target: "feagi-services", "Cortical mapping updated: {} synapses created", synapse_count);
        Ok(synapse_count)
    }
}

