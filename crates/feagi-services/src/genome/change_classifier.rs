/*!
Classification system for cortical area changes to enable intelligent update routing.

This module determines whether cortical area changes require:
- Neuron array updates only (parameter changes)
- Metadata updates only (name changes)
- Synapse rebuild (structural changes like dimensions/neuron density)

Based on Python implementation at: feagi-py/feagi/api/core/services/genome/change_classifier.py

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::{HashMap, HashSet};
use serde_json::Value;

/// Types of cortical area changes requiring different update strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// Direct neuron array updates (NO synapse rebuild)
    /// Examples: firing_threshold, leak_coefficient, refractory_period
    /// Performance: ~2-5ms
    Parameter,
    
    /// Simple property updates (NO neuron/synapse changes)
    /// Examples: cortical_name
    /// Performance: ~1ms
    Metadata,
    
    /// Requires synapse rebuild (localized to affected area)
    /// Examples: cortical_dimensions, neurons_per_voxel, coordinates_3d
    /// Performance: ~100-200ms
    Structural,
    
    /// Multiple types mixed - requires intelligent routing
    Hybrid,
}

/// Classifies cortical area changes to route them to optimal update mechanisms
pub struct CorticalChangeClassifier;

impl CorticalChangeClassifier {
    /// Properties requiring synapse rebuild (affect neuron topology/count/connections)
    /// 
    /// CRITICAL: These changes require deleting and rebuilding synapses TO and FROM
    /// the affected cortical area via localized neuroembryogenesis
    pub fn structural_changes() -> HashSet<&'static str> {
        [
            // Dimension changes → neuron count changes → synapse rebuild required
            "cortical_dimensions",
            "dimensions",
            
            // Neuron density changes → neuron count changes → synapse rebuild required
            "per_voxel_neuron_cnt",
            "cortical_neuron_per_vox_count",
            "neuron_density",
            "neurons_per_voxel",
            
            // Position changes → may affect connection patterns
            "coordinates_3d",
            "coordinates",
            "position",
            
            // Type/role changes
            "cortical_type",
            "area_type",
            
            // Topology changes
            "cortical_mapping_dst",
            
            // Classification changes
            "group_id",
            "sub_group_id",
            "region_id",
            "brain_region_id",
            "parent_region_id",
        ]
        .iter()
        .copied()
        .collect()
    }
    
    /// Simple metadata that can be updated without affecting neurons/synapses
    pub fn metadata_changes() -> HashSet<&'static str> {
        ["cortical_name", "name", "visible"]
            .iter()
            .copied()
            .collect()
    }
    
    /// Parameters mappable to direct neuron array updates (NO synapse rebuild)
    /// 
    /// CRITICAL: These changes ONLY update neuron array values in batch.
    /// They do NOT affect neuron count, topology, or connections.
    pub fn parameter_changes() -> HashSet<&'static str> {
        [
            // Firing threshold parameters
            "firing_threshold",
            "neuron_fire_threshold",
            "firing_threshold_limit",
            "neuron_firing_threshold_limit",
            
            // Refractory period
            "refractory_period",
            "neuron_refractory_period",
            "refrac",
            
            // Leak parameters
            "leak_coefficient",
            "neuron_leak_coefficient",
            "leak",
            
            // Consecutive fire parameters
            "consecutive_fire_cnt_max",
            "neuron_consecutive_fire_count",
            "consecutive_fire_count",
            
            // Snooze period
            "snooze_length",
            "neuron_snooze_period",
            "snooze_period",
            
            // Excitability
            "neuron_excitability",
            
            // Degeneration
            "degeneration",
            "neuron_degeneracy_coefficient",
            
            // Postsynaptic current
            "postsynaptic_current",
            "postsynaptic_current_max",
            
            // Memory parameters
            "longterm_mem_threshold",
            "neuron_longterm_mem_threshold",
            "lifespan_growth_rate",
            "neuron_lifespan_growth_rate",
            "init_lifespan",
            "neuron_init_lifespan",
            "temporal_depth",
            
            // Membrane potential
            "mp_charge_accumulation",
            "neuron_mp_charge_accumulation",
            "mp_driven_psp",
            "neuron_mp_driven_psp",
            
            // Plasticity
            "plasticity_constant",
            
            // Burst engine
            "burst_engine_active",
        ]
        .iter()
        .copied()
        .collect()
    }
    
    /// Parameters that need special handling (may require rebuild)
    pub fn special_parameters() -> HashSet<&'static str> {
        [
            "firing_threshold_increment",
            "neuron_fire_threshold_increment",
            "firing_threshold_increment_x",
            "firing_threshold_increment_y",
            "firing_threshold_increment_z",
            "leak_variability",
            "neuron_leak_variability",
            "psp_uniform_distribution",
            "neuron_psp_uniform_distribution",
            "is_mem_type",
            "dev_count",
            "synapse_attractivity",
            "visualization",
            "location_generation_type",
        ]
        .iter()
        .copied()
        .collect()
    }
    
    /// Classify if changes are structural, parameter, metadata, or hybrid
    pub fn classify_changes(changes: &HashMap<String, Value>) -> ChangeType {
        let structural = Self::structural_changes();
        let parameters = Self::parameter_changes();
        let metadata = Self::metadata_changes();
        let special = Self::special_parameters();
        
        let has_structural = changes.keys().any(|k| structural.contains(k.as_str()));
        let has_parameters = changes.keys().any(|k| parameters.contains(k.as_str()));
        let has_metadata = changes.keys().any(|k| metadata.contains(k.as_str()));
        let has_special = changes.keys().any(|k| special.contains(k.as_str()));
        
        // Count change types
        let change_count = [has_structural, has_parameters, has_metadata, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        
        if change_count > 1 {
            ChangeType::Hybrid
        } else if has_structural || has_special {
            // Special params need rebuild for now
            ChangeType::Structural
        } else if has_parameters {
            ChangeType::Parameter
        } else if has_metadata {
            ChangeType::Metadata
        } else {
            // Unknown changes - be safe and rebuild
            tracing::warn!("Unknown change types detected: {:?}", changes.keys());
            ChangeType::Structural
        }
    }
    
    /// Separate changes into buckets by type for hybrid processing
    pub fn separate_changes_by_type(
        changes: &HashMap<String, Value>,
    ) -> HashMap<ChangeType, HashMap<String, Value>> {
        let structural = Self::structural_changes();
        let parameters = Self::parameter_changes();
        let metadata = Self::metadata_changes();
        let special = Self::special_parameters();
        
        let mut separated = HashMap::new();
        separated.insert(ChangeType::Structural, HashMap::new());
        separated.insert(ChangeType::Parameter, HashMap::new());
        separated.insert(ChangeType::Metadata, HashMap::new());
        
        for (key, value) in changes {
            if structural.contains(key.as_str()) || special.contains(key.as_str()) {
                separated
                    .get_mut(&ChangeType::Structural)
                    .unwrap()
                    .insert(key.clone(), value.clone());
            } else if parameters.contains(key.as_str()) {
                separated
                    .get_mut(&ChangeType::Parameter)
                    .unwrap()
                    .insert(key.clone(), value.clone());
            } else if metadata.contains(key.as_str()) {
                separated
                    .get_mut(&ChangeType::Metadata)
                    .unwrap()
                    .insert(key.clone(), value.clone());
            } else {
                // Unknown - treat as structural to be safe
                separated
                    .get_mut(&ChangeType::Structural)
                    .unwrap()
                    .insert(key.clone(), value.clone());
            }
        }
        
        separated
    }
    
    /// Log the classification result for debugging and monitoring
    pub fn log_classification_result(changes: &HashMap<String, Value>, change_type: ChangeType) {
        let change_summary: Vec<String> = changes
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        
        tracing::info!(
            "[CHANGE-CLASSIFIER] Type: {:?} | Changes: {}",
            change_type,
            change_summary.join(", ")
        );
        
        match change_type {
            ChangeType::Parameter => {
                tracing::info!("[OPTIMIZATION] Fast parameter update path selected - avoiding synapse rebuild");
            }
            ChangeType::Metadata => {
                tracing::info!("[OPTIMIZATION] Metadata-only update - minimal processing required");
            }
            ChangeType::Structural => {
                tracing::info!("[STRUCTURAL] Synapse rebuild required for this change");
            }
            ChangeType::Hybrid => {
                tracing::info!("[HYBRID] Mixed changes - using optimized combination of update paths");
            }
        }
    }
}

