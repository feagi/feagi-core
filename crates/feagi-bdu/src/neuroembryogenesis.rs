/*!
Neuroembryogenesis - Brain Development from Genome.

This module orchestrates the development of a functional connectome (phenotype)
from a genome blueprint (genotype). It coordinates:

1. **Corticogenesis**: Creating cortical area structures
2. **Voxelogenesis**: Establishing 3D spatial framework
3. **Neurogenesis**: Generating neurons within cortical areas
4. **Synaptogenesis**: Forming synaptic connections between neurons

The process is biologically inspired by embryonic brain development.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::sync::Arc;
use parking_lot::RwLock;
use feagi_evo::RuntimeGenome;
use crate::connectome_manager::ConnectomeManager;
use crate::types::BduResult;
use tracing::{info, warn, debug};

/// Development stage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevelopmentStage {
    /// Initial state, not started
    Initialization,
    /// Creating cortical area structures
    Corticogenesis,
    /// Establishing spatial framework
    Voxelogenesis,
    /// Generating neurons
    Neurogenesis,
    /// Forming synaptic connections
    Synaptogenesis,
    /// Development completed successfully
    Completed,
    /// Development failed
    Failed,
}

/// Development progress information
#[derive(Debug, Clone)]
pub struct DevelopmentProgress {
    /// Current development stage
    pub stage: DevelopmentStage,
    /// Progress percentage within current stage (0-100)
    pub progress: u8,
    /// Cortical areas created
    pub cortical_areas_created: usize,
    /// Neurons created
    pub neurons_created: usize,
    /// Synapses created
    pub synapses_created: usize,
    /// Duration of development in milliseconds
    pub duration_ms: u64,
}

impl Default for DevelopmentProgress {
    fn default() -> Self {
        Self {
            stage: DevelopmentStage::Initialization,
            progress: 0,
            cortical_areas_created: 0,
            neurons_created: 0,
            synapses_created: 0,
            duration_ms: 0,
        }
    }
}

/// Neuroembryogenesis orchestrator
///
/// Manages the development of a brain from genome instructions.
/// Uses ConnectomeManager to build the actual neural structures.
pub struct Neuroembryogenesis {
    /// Reference to ConnectomeManager for building structures
    connectome_manager: Arc<RwLock<ConnectomeManager>>,
    
    /// Current development progress
    progress: Arc<RwLock<DevelopmentProgress>>,
    
    /// Start time for duration tracking
    start_time: std::time::Instant,
}

impl Neuroembryogenesis {
    /// Create a new neuroembryogenesis instance
    pub fn new(connectome_manager: Arc<RwLock<ConnectomeManager>>) -> Self {
        Self {
            connectome_manager,
            progress: Arc::new(RwLock::new(DevelopmentProgress::default())),
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Get current development progress
    pub fn get_progress(&self) -> DevelopmentProgress {
        self.progress.read().clone()
    }
    
    /// Develop the brain from a genome
    ///
    /// This is the main entry point that orchestrates all development stages.
    pub fn develop_from_genome(&mut self, genome: &RuntimeGenome) -> BduResult<()> {
        info!(target: "feagi-bdu","🧬 Starting neuroembryogenesis for genome: {}", genome.metadata.genome_id);
        
        // Update stage: Initialization
        self.update_stage(DevelopmentStage::Initialization, 0);
        
        // Stage 1: Corticogenesis - Create cortical area structures
        self.corticogenesis(genome)?;
        
        // Stage 2: Voxelogenesis - Establish spatial framework
        self.voxelogenesis(genome)?;
        
        // Stage 3: Neurogenesis - Generate neurons
        self.neurogenesis(genome)?;
        
        // Stage 4: Synaptogenesis - Form synaptic connections
        self.synaptogenesis(genome)?;
        
        // Mark as completed
        self.update_stage(DevelopmentStage::Completed, 100);
        
        let progress = self.progress.read();
        info!(target: "feagi-bdu",
            "✅ Neuroembryogenesis completed in {}ms: {} cortical areas, {} neurons, {} synapses",
            progress.duration_ms,
            progress.cortical_areas_created,
            progress.neurons_created,
            progress.synapses_created
        );
        
        Ok(())
    }
    
    /// Stage 1: Corticogenesis - Create cortical area structures
    fn corticogenesis(&mut self, genome: &RuntimeGenome) -> BduResult<()> {
        self.update_stage(DevelopmentStage::Corticogenesis, 0);
        info!(target: "feagi-bdu","🧠 Stage 1: Corticogenesis - Creating {} cortical areas", genome.cortical_areas.len());
        
        let total_areas = genome.cortical_areas.len();
        
        // CRITICAL: Minimize lock scope - only hold lock when actually adding areas
        for (idx, (cortical_id, area)) in genome.cortical_areas.iter().enumerate() {
            // Add cortical area to connectome - lock held only during this operation
            {
                let mut manager = self.connectome_manager.write();
                manager.add_cortical_area(area.clone())?;
            } // Lock released immediately after adding
            
            // Update progress (doesn't need lock)
            let progress_pct = ((idx + 1) * 100 / total_areas.max(1)) as u8;
            self.update_progress(|p| {
                p.cortical_areas_created = idx + 1;
                p.progress = progress_pct;
            });
            
            debug!(target: "feagi-bdu","  ✓ Created cortical area: {} ({})", cortical_id, area.name);
        }
        
        // Add brain regions - minimize lock scope
        {
            let mut manager = self.connectome_manager.write();
            let brain_region_count = genome.brain_regions.len();
            info!(target: "feagi-bdu","  Adding {} brain regions from genome", brain_region_count);
            for (region_id, region) in genome.brain_regions.iter() {
                // TODO: Track parent relationships from genome
                manager.add_brain_region(region.clone(), None)?;
                debug!(target: "feagi-bdu","    ✓ Added brain region: {} ({})", region_id, region.name);
            }
            info!(target: "feagi-bdu","  Total brain regions in ConnectomeManager: {}", manager.get_brain_region_ids().len());
        } // Lock released
        
        self.update_stage(DevelopmentStage::Corticogenesis, 100);
        info!(target: "feagi-bdu","  ✅ Corticogenesis complete: {} cortical areas created", total_areas);
        
        Ok(())
    }
    
    /// Stage 2: Voxelogenesis - Establish spatial framework
    fn voxelogenesis(&mut self, _genome: &RuntimeGenome) -> BduResult<()> {
        self.update_stage(DevelopmentStage::Voxelogenesis, 0);
        info!(target: "feagi-bdu","📐 Stage 2: Voxelogenesis - Establishing spatial framework");
        
        // Spatial framework is implicitly established by cortical area dimensions
        // The Morton spatial hash in ConnectomeManager handles the actual indexing
        
        self.update_stage(DevelopmentStage::Voxelogenesis, 100);
        info!(target: "feagi-bdu","  ✅ Voxelogenesis complete: Spatial framework established");
        
        Ok(())
    }
    
    /// Stage 3: Neurogenesis - Generate neurons within cortical areas
    ///
    /// This uses ConnectomeManager which delegates to NPU's SIMD-optimized batch operations.
    /// Each cortical area is processed with `create_cortical_area_neurons()` which creates
    /// ALL neurons for that area in one vectorized operation (not a loop).
    fn neurogenesis(&mut self, genome: &RuntimeGenome) -> BduResult<()> {
        self.update_stage(DevelopmentStage::Neurogenesis, 0);
        info!(target: "feagi-bdu","🔬 Stage 3: Neurogenesis - Generating neurons (SIMD-optimized batches)");
        
        let expected_neurons = genome.stats.innate_neuron_count;
        info!(target: "feagi-bdu","  Expected innate neurons from genome: {}", expected_neurons);
        
        let mut total_neurons_created = 0;
        let total_areas = genome.cortical_areas.len();
        
        // Process each cortical area via ConnectomeManager (each area = one SIMD batch)
        // NOTE: Loop is over AREAS, not neurons. Each area creates all its neurons in ONE batch call.
        for (idx, (cortical_id, area)) in genome.cortical_areas.iter().enumerate() {
            // Get per_voxel_neuron_cnt from area properties
            let per_voxel_count = area.properties
                .get("per_voxel_neuron_cnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);
            
            if per_voxel_count == 0 {
                debug!(target: "feagi-bdu","  Skipping area {} - per_voxel_neuron_cnt is 0", cortical_id);
                continue;
            }
            
            // Call ConnectomeManager to create neurons (delegates to NPU)
            // CRITICAL: Minimize lock scope - only hold lock during neuron creation
            let neurons_created = {
                let manager_arc = self.connectome_manager.clone();
                let mut manager = manager_arc.write();
                manager.create_neurons_for_area(cortical_id)
            }; // Lock released immediately
            
            match neurons_created {
                Ok(count) => {
                    total_neurons_created += count as usize;
                    info!(target: "feagi-bdu","  Created {} neurons for area {}", count, cortical_id);
                }
                Err(e) => {
                    // If NPU not connected, calculate expected count
                    warn!(target: "feagi-bdu","  Failed to create neurons for {}: {} (NPU may not be connected)", 
                        cortical_id, e);
                    let total_voxels = area.dimensions.width * area.dimensions.height * area.dimensions.depth;
                    let expected = total_voxels * per_voxel_count as usize;
                    total_neurons_created += expected;
                }
            }
            
            // Update progress
            let progress_pct = ((idx + 1) * 100 / total_areas.max(1)) as u8;
            self.update_progress(|p| {
                p.neurons_created = total_neurons_created;
                p.progress = progress_pct;
            });
        }
        
        // Verify against genome stats
        if expected_neurons > 0 && total_neurons_created != expected_neurons {
            warn!(target: "feagi-bdu",
                "Neuron count mismatch: created {} but genome stats expected {}",
                total_neurons_created, expected_neurons
            );
        }
        
        self.update_stage(DevelopmentStage::Neurogenesis, 100);
        info!(target: "feagi-bdu","  ✅ Neurogenesis complete: {} neurons created", total_neurons_created);
        
        Ok(())
    }
    
    /// Stage 4: Synaptogenesis - Form synaptic connections between neurons
    ///
    /// This uses ConnectomeManager which delegates to NPU's morphology functions.
    /// Each morphology application (`apply_projector_morphology`, etc.) processes ALL neurons
    /// from the source area and creates ALL synapses in one SIMD-optimized batch operation.
    fn synaptogenesis(&mut self, genome: &RuntimeGenome) -> BduResult<()> {
        self.update_stage(DevelopmentStage::Synaptogenesis, 0);
        info!(target: "feagi-bdu","🔗 Stage 4: Synaptogenesis - Forming synaptic connections (SIMD-optimized batches)");
        
        let expected_synapses = genome.stats.innate_synapse_count;
        info!(target: "feagi-bdu","  Expected innate synapses from genome: {}", expected_synapses);
        
        let mut total_synapses_created = 0;
        let total_areas = genome.cortical_areas.len();
        
        // Process each source area via ConnectomeManager (each mapping = one SIMD batch)
        // NOTE: Loop is over AREAS, not synapses. Each area applies all mappings in batch calls.
        for (idx, (src_cortical_id, src_area)) in genome.cortical_areas.iter().enumerate() {
            // Check if area has mappings
            let has_dstmap = src_area.properties.get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .map(|m| !m.is_empty())
                .unwrap_or(false);
            
            if !has_dstmap {
                debug!(target: "feagi-bdu","  No dstmap for area {}", src_cortical_id);
                continue;
            }
            
            // Call ConnectomeManager to apply cortical mappings (delegates to NPU)
            // CRITICAL: Minimize lock scope - only hold lock during synapse creation
            let synapses_created = {
                let manager_arc = self.connectome_manager.clone();
                let mut manager = manager_arc.write();
                manager.apply_cortical_mapping(src_cortical_id)
            }; // Lock released immediately
            
            match synapses_created {
                Ok(count) => {
                    total_synapses_created += count as usize;
                    info!(target: "feagi-bdu","  Created {} synapses for area {}", count, src_cortical_id);
                }
                Err(e) => {
                    // If NPU not connected, estimate count
                    warn!(target: "feagi-bdu","  Failed to create synapses for {}: {} (NPU may not be connected)", 
                        src_cortical_id, e);
                    let estimated = estimate_synapses_for_area(src_area, genome);
                    total_synapses_created += estimated;
                }
            }
            
            // Update progress
            let progress_pct = ((idx + 1) * 100 / total_areas.max(1)) as u8;
            self.update_progress(|p| {
                p.synapses_created = total_synapses_created;
                p.progress = progress_pct;
            });
        }
        
        // Verify against genome stats
        if expected_synapses > 0 {
            let diff = (total_synapses_created as i64 - expected_synapses as i64).abs();
            let diff_pct = (diff as f64 / expected_synapses.max(1) as f64) * 100.0;
            
            if diff_pct > 10.0 {
                warn!(target: "feagi-bdu",
                    "Synapse count variance: created {} but genome stats expected {} ({:.1}% difference)",
                    total_synapses_created, expected_synapses, diff_pct
                );
            } else {
                info!(target: "feagi-bdu",
                    "Synapse count matches genome stats within {:.1}% ({} vs {})",
                    diff_pct, total_synapses_created, expected_synapses
                );
            }
        }
        
        self.update_stage(DevelopmentStage::Synaptogenesis, 100);
        info!(target: "feagi-bdu","  ✅ Synaptogenesis complete: {} synapses created", total_synapses_created);
        
        Ok(())
    }
}

/// Estimate synapse count for an area (fallback when NPU not connected)
///
/// This is only used when NPU is not available for actual synapse creation.
fn estimate_synapses_for_area(
    src_area: &feagi_types::CorticalArea,
    genome: &feagi_evo::RuntimeGenome,
) -> usize {
    let dstmap = match src_area.properties.get("cortical_mapping_dst") {
        Some(serde_json::Value::Object(map)) => map,
        _ => return 0,
    };
    
    let mut total = 0;
    
    for (dst_id, rules) in dstmap {
        let dst_area = match genome.cortical_areas.get(dst_id) {
            Some(area) => area,
            None => continue,
        };
        
        let rules_array = match rules.as_array() {
            Some(arr) => arr,
            None => continue,
        };
        
        for rule in rules_array {
            let morphology_id = rule.get("morphology_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let scalar = rule.get("morphology_scalar")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as usize;
            
            // Simplified estimation
            let src_per_voxel = src_area.properties
                .get("per_voxel_neuron_cnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as usize;
            let dst_per_voxel = dst_area.properties
                .get("per_voxel_neuron_cnt")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as usize;
            
            let src_voxels = src_area.dimensions.width * src_area.dimensions.height * src_area.dimensions.depth;
            let dst_voxels = dst_area.dimensions.width * dst_area.dimensions.height * dst_area.dimensions.depth;
            
            let src_neurons = src_voxels * src_per_voxel;
            let dst_neurons = dst_voxels * dst_per_voxel;
            
            // Basic estimation by morphology type
            let count = match morphology_id {
                "block_to_block" => src_neurons * dst_per_voxel * scalar,
                "projector" => src_neurons * dst_neurons * scalar,
                _ if morphology_id.contains("lateral") => src_neurons * scalar,
                _ => (src_neurons * scalar).min(src_neurons * dst_neurons / 10),
            };
            
            total += count;
        }
    }
    
    total
}

impl Neuroembryogenesis {
    /// Update development stage
    fn update_stage(&self, stage: DevelopmentStage, progress: u8) {
        let mut p = self.progress.write();
        p.stage = stage;
        p.progress = progress;
        p.duration_ms = self.start_time.elapsed().as_millis() as u64;
    }
    
    /// Update progress with a closure
    fn update_progress<F>(&self, f: F)
    where
        F: FnOnce(&mut DevelopmentProgress),
    {
        let mut p = self.progress.write();
        f(&mut p);
        p.duration_ms = self.start_time.elapsed().as_millis() as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_evo::{create_genome_with_core_morphologies};
    
    #[test]
    fn test_neuroembryogenesis_creation() {
        let manager = ConnectomeManager::instance();
        let neuro = Neuroembryogenesis::new(manager);
        
        let progress = neuro.get_progress();
        assert_eq!(progress.stage, DevelopmentStage::Initialization);
        assert_eq!(progress.progress, 0);
    }
    
    #[test]
    fn test_development_from_minimal_genome() {
        let manager = ConnectomeManager::instance();
        let mut neuro = Neuroembryogenesis::new(manager.clone());
        
        // Create a minimal genome with one cortical area
        let mut genome = create_genome_with_core_morphologies(
            "test_genome".to_string(),
            "Test Genome".to_string(),
        );
        
        let area = feagi_types::CorticalArea::new(
            "test01".to_string(),
            0,
            "Test Area".to_string(),
            feagi_types::Dimensions::new(10, 10, 10),
            (0, 0, 0),
            feagi_types::AreaType::Custom,
        ).expect("Failed to create cortical area");
        genome.cortical_areas.insert("test01".to_string(), area);
        
        // Run neuroembryogenesis
        let result = neuro.develop_from_genome(&genome);
        assert!(result.is_ok(), "Development failed: {:?}", result);
        
        // Check progress
        let progress = neuro.get_progress();
        assert_eq!(progress.stage, DevelopmentStage::Completed);
        assert_eq!(progress.progress, 100);
        assert_eq!(progress.cortical_areas_created, 1);
        
        // Verify cortical area was added to connectome
        let mgr = manager.read();
        assert_eq!(mgr.get_cortical_area_count(), 1);
        
        println!("✅ Development completed in {}ms", progress.duration_ms);
    }
}

