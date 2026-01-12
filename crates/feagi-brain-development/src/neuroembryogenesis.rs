// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

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

use crate::connectome_manager::ConnectomeManager;
use crate::models::{CorticalArea, CorticalID};
use crate::types::{BduError, BduResult};
use feagi_evolutionary::RuntimeGenome;
use feagi_npu_neural::types::{Precision, QuantizationSpec};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

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
///
/// # Type Parameters
/// - `T: NeuralValue`: The numeric precision for the connectome (f32, INT8Value, f16)
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

    /// Incrementally add cortical areas to an existing connectome
    ///
    /// This is for adding new cortical areas after the initial genome has been loaded.
    /// Unlike `develop_from_genome()`, this only processes the new areas.
    ///
    /// # Arguments
    /// * `areas` - The cortical areas to add
    /// * `genome` - The full runtime genome (needed for synaptogenesis context)
    ///
    /// # Returns
    /// * Number of neurons created and synapses created
    pub fn add_cortical_areas(
        &mut self,
        areas: Vec<CorticalArea>,
        genome: &RuntimeGenome,
    ) -> BduResult<(usize, usize)> {
        info!(target: "feagi-bdu", "🧬 Incrementally adding {} cortical areas", areas.len());

        let mut total_neurons = 0;
        let mut total_synapses = 0;

        // Stage 1: Add cortical area structures (Corticogenesis)
        for area in &areas {
            let mut manager = self.connectome_manager.write();
            manager.add_cortical_area(area.clone())?;
            info!(target: "feagi-bdu", "  ✓ Added cortical area structure: {}", area.cortical_id.as_base_64());
        }

        // Stage 2: Create neurons for each area (Neurogenesis)
        // CRITICAL: Create core area neurons FIRST to ensure deterministic IDs
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        let death_id = CoreCorticalType::Death.to_cortical_id();
        let power_id = CoreCorticalType::Power.to_cortical_id();
        let fatigue_id = CoreCorticalType::Fatigue.to_cortical_id();

        let mut core_areas = Vec::new();
        let mut other_areas = Vec::new();

        // Separate core areas from other areas
        for area in &areas {
            if area.cortical_id == death_id {
                core_areas.push((0, area)); // Area 0 = _death
            } else if area.cortical_id == power_id {
                core_areas.push((1, area)); // Area 1 = _power
            } else if area.cortical_id == fatigue_id {
                core_areas.push((2, area)); // Area 2 = _fatigue
            } else {
                other_areas.push(area);
            }
        }

        // Sort core areas by their deterministic index (0, 1, 2)
        core_areas.sort_by_key(|(idx, _)| *idx);

        // STEP 1: Create core area neurons FIRST
        if !core_areas.is_empty() {
            info!(target: "feagi-bdu", "  🎯 Creating core area neurons FIRST ({} areas) for deterministic IDs", core_areas.len());
            for (core_idx, area) in &core_areas {
                let neurons_created = {
                    let mut manager = self.connectome_manager.write();
                    manager.create_neurons_for_area(&area.cortical_id)
                };

                match neurons_created {
                    Ok(count) => {
                        total_neurons += count as usize;
                        info!(target: "feagi-bdu", "  ✅ Created {} neurons for core area {} (deterministic ID: neuron {})",
                            count, area.cortical_id.as_base_64(), core_idx);
                    }
                    Err(e) => {
                        error!(target: "feagi-bdu", "  ❌ FATAL: Failed to create neurons for core area {}: {}", area.cortical_id.as_base_64(), e);
                        return Err(e);
                    }
                }
            }
        }

        // STEP 2: Create neurons for other areas
        for area in &other_areas {
            let neurons_created = {
                let mut manager = self.connectome_manager.write();
                manager.create_neurons_for_area(&area.cortical_id)
            };

            match neurons_created {
                Ok(count) => {
                    total_neurons += count as usize;
                    trace!(
                        target: "feagi-bdu",
                        "Created {} neurons for area {}",
                        count,
                        area.cortical_id.as_base_64()
                    );
                }
                Err(e) => {
                    error!(target: "feagi-bdu", "  ❌ FATAL: Failed to create neurons for {}: {}", area.cortical_id.as_base_64(), e);
                    // CRITICAL: NPU capacity errors must propagate to UI
                    return Err(e);
                }
            }
        }

        // Stage 3: Create synapses for each area (Synaptogenesis)
        for area in &areas {
            // Check if area has mappings
            let has_dstmap = area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .map(|m| !m.is_empty())
                .unwrap_or(false);

            if !has_dstmap {
                debug!(target: "feagi-bdu", "  No mappings for area {}", area.cortical_id.as_base_64());
                continue;
            }

            let synapses_created = {
                let mut manager = self.connectome_manager.write();
                manager.apply_cortical_mapping(&area.cortical_id)
            };

            match synapses_created {
                Ok(count) => {
                    total_synapses += count as usize;
                    trace!(
                        target: "feagi-bdu",
                        "Created {} synapses for area {}",
                        count,
                        area.cortical_id
                    );
                }
                Err(e) => {
                    warn!(target: "feagi-bdu", "  ⚠️ Failed to create synapses for {}: {}", area.cortical_id, e);
                    let estimated = estimate_synapses_for_area(area, genome);
                    total_synapses += estimated;
                }
            }
        }

        info!(target: "feagi-bdu", "✅ Incremental add complete: {} areas, {} neurons, {} synapses",
              areas.len(), total_neurons, total_synapses);

        Ok((total_neurons, total_synapses))
    }

    /// Develop the brain from a genome
    ///
    /// This is the main entry point that orchestrates all development stages.
    pub fn develop_from_genome(&mut self, genome: &RuntimeGenome) -> BduResult<()> {
        info!(target: "feagi-bdu","🧬 Starting neuroembryogenesis for genome: {}", genome.metadata.genome_id);

        // Phase 5: Parse quantization precision and dispatch to type-specific builder
        let _quantization_precision = &genome.physiology.quantization_precision;
        // Precision parsing handled in genome loader
        let quant_spec = QuantizationSpec::default();

        info!(target: "feagi-bdu",
            "   Quantization precision: {:?} (range: [{}, {}] for membrane potential)",
            quant_spec.precision,
            quant_spec.membrane_potential_min,
            quant_spec.membrane_potential_max
        );

        // Phase 6: Type dispatch - Neuroembryogenesis is now fully generic!
        // The precision is determined by the type T of this Neuroembryogenesis instance.
        // All stages (corticogenesis, neurogenesis, synaptogenesis) automatically use the correct type.
        match quant_spec.precision {
            Precision::FP32 => {
                info!(target: "feagi-bdu", "   ✓ Using FP32 (32-bit floating-point) - highest precision");
                info!(target: "feagi-bdu", "   Memory usage: Baseline (4 bytes/neuron for membrane potential)");
            }
            Precision::INT8 => {
                info!(target: "feagi-bdu", "   ✓ Using INT8 (8-bit integer) - memory efficient");
                info!(target: "feagi-bdu", "   Memory reduction: 42% (1 byte/neuron for membrane potential)");
                info!(target: "feagi-bdu", "   Quantization range: [{}, {}]",
                    quant_spec.membrane_potential_min,
                    quant_spec.membrane_potential_max);
                // Note: If this Neuroembryogenesis was created with <f32>, this will warn below
                // The caller must create Neuroembryogenesis::<INT8Value> to use INT8
            }
            Precision::FP16 => {
                warn!(target: "feagi-bdu", "   FP16 quantization requested but not yet implemented.");
                warn!(target: "feagi-bdu", "   FP16 support planned for future GPU optimization.");
                // Note: Requires f16 type and implementation
            }
        }

        // Type consistency is now handled by DynamicNPU at creation time
        // The caller (main.rs) peeks at genome precision and creates the correct DynamicNPU variant
        info!(target: "feagi-bdu", "   ✓ Quantization handled by DynamicNPU (dispatches at runtime)");

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
        info!(target: "feagi-bdu","🔍 Genome brain_regions check: is_empty={}, count={}",
              genome.brain_regions.is_empty(), genome.brain_regions.len());
        if !genome.brain_regions.is_empty() {
            info!(target: "feagi-bdu","   Existing regions: {:?}", genome.brain_regions.keys().collect::<Vec<_>>());
        }

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

            trace!(target: "feagi-bdu", "Created cortical area: {} ({})", cortical_id, area.name);
        }

        // Ensure brain regions structure exists (auto-generate if missing)
        // This matches Python's normalize_brain_region_membership() behavior
        info!(target: "feagi-bdu","🔍 BRAIN REGION AUTO-GEN CHECK: genome.brain_regions.is_empty() = {}", genome.brain_regions.is_empty());
        let (brain_regions_to_add, region_parent_map) = if genome.brain_regions.is_empty() {
            info!(target: "feagi-bdu","  ✅ TRIGGERING AUTO-GENERATION: No brain_regions in genome - auto-generating default root region");
            info!(target: "feagi-bdu","  📊 Genome has {} cortical areas to process", genome.cortical_areas.len());

            // Collect all cortical area IDs
            let all_cortical_ids = genome.cortical_areas.keys().cloned().collect::<Vec<_>>();
            info!(target: "feagi-bdu","  📊 Collected {} cortical area IDs: {:?}", all_cortical_ids.len(),
            if all_cortical_ids.len() <= 5 {
                format!("{:?}", all_cortical_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            } else {
                format!("{:?}...", all_cortical_ids[0..5].iter().map(|id| id.to_string()).collect::<Vec<_>>())
            });

            // Classify areas into inputs/outputs based on their AreaType
            let mut auto_inputs = Vec::new();
            let mut auto_outputs = Vec::new();

            // Classify areas into categories following Python's normalize_brain_region_membership()
            let mut ipu_areas = Vec::new(); // Sensory inputs
            let mut opu_areas = Vec::new(); // Motor outputs
            let mut core_areas = Vec::new(); // Core/maintenance (like _power)
            let mut custom_memory_areas = Vec::new(); // CUSTOM/MEMORY (go to subregion)

            for (area_id, area) in genome.cortical_areas.iter() {
                // Classify following Python's logic with gradual migration to new type system:
                // 1. Areas starting with "_" are always CORE
                // 2. Check cortical_type_new (new strongly-typed system) - Phase 2+
                // 3. Check cortical_group property (parsed from genome)
                // 4. Fallback to area_type (old simple enum)

                let area_id_str = area_id.to_string();
                // Note: Core IDs are 8-byte padded and start with "___" (three underscores)
                let category = if area_id_str.starts_with("___") {
                    "CORE"
                } else if let Ok(cortical_type) = area.cortical_id.as_cortical_type() {
                    // Use cortical type from CorticalID
                    use feagi_structures::genomic::cortical_area::CorticalAreaType;
                    match cortical_type {
                        CorticalAreaType::Core(_) => "CORE",
                        CorticalAreaType::BrainInput(_) => "IPU",
                        CorticalAreaType::BrainOutput(_) => "OPU",
                        CorticalAreaType::Memory(_) => "MEMORY",
                        CorticalAreaType::Custom(_) => "CUSTOM",
                    }
                } else {
                    // Fallback to cortical_group property or area_type
                    let cortical_group = area
                        .properties
                        .get("cortical_group")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_uppercase());

                    match cortical_group.as_deref() {
                        Some("IPU") => "IPU",
                        Some("OPU") => "OPU",
                        Some("CORE") => "CORE",
                        Some("MEMORY") => "MEMORY",
                        Some("CUSTOM") => "CUSTOM",
                        _ => "CUSTOM", // Default fallback
                    }
                };

                // Phase 3: Enhanced logging with detailed type information
                if ipu_areas.len() + opu_areas.len() + core_areas.len() + custom_memory_areas.len()
                    < 5
                {
                    let source = if area.cortical_id.as_cortical_type().is_ok() {
                        "cortical_id_type"
                    } else if area.properties.contains_key("cortical_group") {
                        "cortical_group"
                    } else {
                        "default_fallback"
                    };

                    // Phase 3: Show detailed type information if available
                    if area.cortical_id.as_cortical_type().is_ok() {
                        let type_desc = crate::cortical_type_utils::describe_cortical_type(area);
                        let frame_handling =
                            if crate::cortical_type_utils::uses_absolute_frames(area) {
                                "absolute"
                            } else if crate::cortical_type_utils::uses_incremental_frames(area) {
                                "incremental"
                            } else {
                                "n/a"
                            };
                        info!(target: "feagi-bdu","    🔍 {}, frames={}, source={}",
                              type_desc, frame_handling, source);
                    } else {
                        info!(target: "feagi-bdu","    🔍 Area {}: category={}, source={}",
                              area_id_str, category, source);
                    }
                }

                // Assign to appropriate list
                match category {
                    "IPU" => {
                        ipu_areas.push(*area_id);
                        auto_inputs.push(*area_id);
                    }
                    "OPU" => {
                        opu_areas.push(*area_id);
                        auto_outputs.push(*area_id);
                    }
                    "CORE" => {
                        core_areas.push(*area_id);
                    }
                    "MEMORY" | "CUSTOM" => {
                        custom_memory_areas.push(*area_id);
                    }
                    _ => {}
                }
            }

            info!(target: "feagi-bdu","  📊 Classification complete: IPU={}, OPU={}, CORE={}, CUSTOM/MEMORY={}",
                  ipu_areas.len(), opu_areas.len(), core_areas.len(), custom_memory_areas.len());

            // Build brain region structure following Python's normalize_brain_region_membership()
            use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
            let mut regions_map = std::collections::HashMap::new();

            // Step 1: Create root region with only IPU/OPU/CORE areas
            let mut root_area_ids = Vec::new();
            root_area_ids.extend(ipu_areas.iter().cloned());
            root_area_ids.extend(opu_areas.iter().cloned());
            root_area_ids.extend(core_areas.iter().cloned());

            // Analyze connections to determine actual inputs/outputs for root
            let (root_inputs, root_outputs) =
                Self::analyze_region_io(&root_area_ids, &genome.cortical_areas);

            // Convert CorticalID to base64 for with_areas()
            // Create a root region with a generated RegionID
            let root_region_id = RegionID::new();
            let root_region_id_str = root_region_id.to_string();

            let mut root_region = BrainRegion::new(
                root_region_id,
                "Root Brain Region".to_string(),
                RegionType::Undefined,
            )
            .expect("Failed to create root region")
            .with_areas(root_area_ids.iter().cloned());

            // Store inputs/outputs based on connection analysis
            if !root_inputs.is_empty() {
                root_region
                    .add_property("inputs".to_string(), serde_json::json!(root_inputs.clone()));
            }
            if !root_outputs.is_empty() {
                root_region.add_property(
                    "outputs".to_string(),
                    serde_json::json!(root_outputs.clone()),
                );
            }

            info!(target: "feagi-bdu","  ✅ Created root region with {} areas (IPU={}, OPU={}, CORE={}) - analyzed: {} inputs, {} outputs",
                  root_area_ids.len(), ipu_areas.len(), opu_areas.len(), core_areas.len(),
                  root_inputs.len(), root_outputs.len());

            // Step 2: Create subregion for CUSTOM/MEMORY areas if any exist
            let mut subregion_id = None;
            if !custom_memory_areas.is_empty() {
                // Convert CorticalID to base64 for sorting and hashing
                let mut custom_memory_strs: Vec<String> = custom_memory_areas
                    .iter()
                    .map(|id| id.as_base_64())
                    .collect();
                custom_memory_strs.sort(); // Sort for deterministic hash
                let combined = custom_memory_strs.join("|");

                // Use a simple hash (matching Python's sha1[:8])
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                combined.hash(&mut hasher);
                let hash = hasher.finish();
                let hash_hex = format!("{:08x}", hash as u32);
                let region_id = format!("region_autogen_{}", hash_hex);

                // Analyze connections to determine inputs/outputs for subregion
                let (subregion_inputs, subregion_outputs) =
                    Self::analyze_region_io(&custom_memory_areas, &genome.cortical_areas);

                // Calculate smart position: Place autogen region outside root's bounding box
                let autogen_position =
                    Self::calculate_autogen_region_position(&root_area_ids, genome);

                // Create subregion
                let mut subregion = BrainRegion::new(
                    RegionID::new(), // Generate new UUID instead of using string
                    "Autogen Region".to_string(),
                    RegionType::Undefined, // RegionType no longer has Custom variant
                )
                .expect("Failed to create subregion")
                .with_areas(custom_memory_areas.iter().cloned());

                // Set 3D coordinates (place outside root's bounding box)
                subregion.add_property(
                    "coordinate_3d".to_string(),
                    serde_json::json!(autogen_position),
                );
                subregion.add_property("coordinate_2d".to_string(), serde_json::json!([0, 0]));

                // Store inputs/outputs for subregion
                if !subregion_inputs.is_empty() {
                    subregion.add_property(
                        "inputs".to_string(),
                        serde_json::json!(subregion_inputs.clone()),
                    );
                }
                if !subregion_outputs.is_empty() {
                    subregion.add_property(
                        "outputs".to_string(),
                        serde_json::json!(subregion_outputs.clone()),
                    );
                }

                let subregion_id_str = subregion.region_id.to_string();

                info!(target: "feagi-bdu","  ✅ Created subregion '{}' with {} CUSTOM/MEMORY areas ({} inputs, {} outputs)",
                      region_id, custom_memory_areas.len(), subregion_inputs.len(), subregion_outputs.len());

                regions_map.insert(subregion_id_str.clone(), subregion);
                subregion_id = Some(subregion_id_str);
            }

            regions_map.insert(root_region_id_str.clone(), root_region);

            // Count total inputs/outputs across all regions
            let total_inputs = root_inputs.len()
                + if let Some(ref sid) = subregion_id {
                    regions_map
                        .get(sid)
                        .and_then(|r| r.properties.get("inputs"))
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0)
                } else {
                    0
                };

            let total_outputs = root_outputs.len()
                + if let Some(ref sid) = subregion_id {
                    regions_map
                        .get(sid)
                        .and_then(|r| r.properties.get("outputs"))
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0)
                } else {
                    0
                };

            info!(target: "feagi-bdu","  ✅ Auto-generated {} brain region(s) with {} total cortical areas ({} total inputs, {} total outputs)",
                  regions_map.len(), all_cortical_ids.len(), total_inputs, total_outputs);

            // Return (regions_map, parent_map) so we can properly link hierarchy
            let mut parent_map = std::collections::HashMap::new();
            if let Some(ref sub_id) = subregion_id {
                parent_map.insert(sub_id.clone(), root_region_id_str.clone());
                info!(target: "feagi-bdu","  🔗 Parent relationship: {} -> {}", sub_id, root_region_id_str);
            }

            (regions_map, parent_map)
        } else {
            info!(target: "feagi-bdu","  📋 Genome already has {} brain regions - using existing structure", genome.brain_regions.len());
            // For existing genomes, we don't have parent info readily available
            // TODO: Extract parent relationships from genome structure
            (
                genome.brain_regions.clone(),
                std::collections::HashMap::new(),
            )
        };

        // Add brain regions with proper parent relationships - minimize lock scope
        {
            let mut manager = self.connectome_manager.write();
            let brain_region_count = brain_regions_to_add.len();
            info!(target: "feagi-bdu","  Adding {} brain regions from genome", brain_region_count);

            // First add root (no parent) - need to find it by iterating since key is UUID
            let root_entry = brain_regions_to_add
                .iter()
                .find(|(_, region)| region.name == "Root Brain Region");
            if let Some((root_id, root_region)) = root_entry {
                manager.add_brain_region(root_region.clone(), None)?;
                debug!(target: "feagi-bdu","    ✓ Added brain region: {} (Root Brain Region) [parent=None]", root_id);
            }

            // Then add other regions with their parent relationships
            for (region_id, region) in brain_regions_to_add.iter() {
                if region.name == "Root Brain Region" {
                    continue; // Already added
                }

                let parent_id = region_parent_map.get(region_id).cloned();
                manager.add_brain_region(region.clone(), parent_id.clone())?;
                debug!(target: "feagi-bdu","    ✓ Added brain region: {} ({}) [parent={:?}]",
                       region_id, region.name, parent_id);
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
    ///
    /// CRITICAL: Core areas (0=_death, 1=_power, 2=_fatigue) are created FIRST to ensure
    /// deterministic neuron IDs (neuron 0 for area 0, neuron 1 for area 1, neuron 2 for area 2).
    fn neurogenesis(&mut self, genome: &RuntimeGenome) -> BduResult<()> {
        self.update_stage(DevelopmentStage::Neurogenesis, 0);
        info!(target: "feagi-bdu","🔬 Stage 3: Neurogenesis - Generating neurons (SIMD-optimized batches)");

        let expected_neurons = genome.stats.innate_neuron_count;
        info!(target: "feagi-bdu","  Expected innate neurons from genome: {}", expected_neurons);

        // CRITICAL: Identify core areas first to ensure deterministic neuron IDs
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        let death_id = CoreCorticalType::Death.to_cortical_id();
        let power_id = CoreCorticalType::Power.to_cortical_id();
        let fatigue_id = CoreCorticalType::Fatigue.to_cortical_id();

        let mut core_areas = Vec::new();
        let mut other_areas = Vec::new();

        // Separate core areas from other areas
        for (cortical_id, area) in genome.cortical_areas.iter() {
            if *cortical_id == death_id {
                core_areas.push((0, *cortical_id, area)); // Area 0 = _death
            } else if *cortical_id == power_id {
                core_areas.push((1, *cortical_id, area)); // Area 1 = _power
            } else if *cortical_id == fatigue_id {
                core_areas.push((2, *cortical_id, area)); // Area 2 = _fatigue
            } else {
                other_areas.push((*cortical_id, area));
            }
        }

        // Sort core areas by their deterministic index (0, 1, 2)
        core_areas.sort_by_key(|(idx, _, _)| *idx);

        info!(target: "feagi-bdu","  🎯 Creating core area neurons FIRST ({} areas) for deterministic IDs", core_areas.len());

        let mut total_neurons_created = 0;
        let mut processed_count = 0;
        let total_areas = genome.cortical_areas.len();

        // STEP 1: Create core area neurons FIRST (in order: 0, 1, 2)
        for (core_idx, cortical_id, area) in &core_areas {
            let per_voxel_count = area
                .properties
                .get("neurons_per_voxel")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as i64;

            let cortical_id_str = cortical_id.to_string();
            info!(target: "feagi-bdu","  🔋 [CORE-AREA {}] {} - dimensions: {:?}, per_voxel: {}",
                core_idx, cortical_id_str, area.dimensions, per_voxel_count);

            if per_voxel_count == 0 {
                warn!(target: "feagi-bdu","  ⚠️ Skipping core area {} - per_voxel_neuron_cnt is 0", cortical_id_str);
                continue;
            }

            // Create neurons for core area (ensures deterministic ID: area 0→neuron 0, area 1→neuron 1, area 2→neuron 2)
            let neurons_created = {
                let manager_arc = self.connectome_manager.clone();
                let mut manager = manager_arc.write();
                manager.create_neurons_for_area(cortical_id)
            };

            match neurons_created {
                Ok(count) => {
                    total_neurons_created += count as usize;
                    info!(target: "feagi-bdu","  ✅ Created {} neurons for core area {} (deterministic ID: neuron {})",
                        count, cortical_id_str, core_idx);
                }
                Err(e) => {
                    error!(target: "feagi-bdu","  ❌ FATAL: Failed to create neurons for core area {}: {}", cortical_id_str, e);
                    return Err(e);
                }
            }

            processed_count += 1;
            let progress_pct = (processed_count * 100 / total_areas.max(1)) as u8;
            self.update_progress(|p| {
                p.neurons_created = total_neurons_created;
                p.progress = progress_pct;
            });
        }

        // STEP 2: Create neurons for all other areas
        info!(target: "feagi-bdu","  📦 Creating neurons for {} other areas", other_areas.len());
        for (cortical_id, area) in &other_areas {
            // Get neurons_per_voxel from typed field (single source of truth)
            let _per_voxel_count = area
                .properties
                .get("neurons_per_voxel")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as i64;

            let per_voxel_count = area
                .properties
                .get("neurons_per_voxel")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as i64;

            let cortical_id_str = cortical_id.to_string();

            if per_voxel_count == 0 {
                warn!(target: "feagi-bdu","  ⚠️ Skipping area {} - per_voxel_neuron_cnt is 0 (will have NO neurons!)", cortical_id_str);
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
                    trace!(
                        target: "feagi-bdu",
                        "Created {} neurons for area {}",
                        count,
                        cortical_id_str
                    );
                }
                Err(e) => {
                    // If NPU not connected, calculate expected count
                    warn!(target: "feagi-bdu","  Failed to create neurons for {}: {} (NPU may not be connected)",
                        cortical_id_str, e);
                    let total_voxels = area.dimensions.width as usize
                        * area.dimensions.height as usize
                        * area.dimensions.depth as usize;
                    let expected = total_voxels * per_voxel_count as usize;
                    total_neurons_created += expected;
                }
            }

            processed_count += 1;
            // Update progress
            let progress_pct = (processed_count * 100 / total_areas.max(1)) as u8;
            self.update_progress(|p| {
                p.neurons_created = total_neurons_created;
                p.progress = progress_pct;
            });
        }

        // Compare with genome stats (info only - stats may count only innate neurons while we create all voxels)
        if expected_neurons > 0 && total_neurons_created != expected_neurons {
            trace!(target: "feagi-bdu",
                created_neurons = total_neurons_created,
                genome_stats_innate = expected_neurons,
                "Neuron creation complete (genome stats may only count innate neurons)"
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
        for (idx, (_src_cortical_id, src_area)) in genome.cortical_areas.iter().enumerate() {
            // Check if area has mappings
            let has_dstmap = src_area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .map(|m| !m.is_empty())
                .unwrap_or(false);

            if !has_dstmap {
                trace!(target: "feagi-bdu", "No dstmap for area {}", &src_area.cortical_id);
                continue;
            }

            // Call ConnectomeManager to apply cortical mappings (delegates to NPU)
            // CRITICAL: Minimize lock scope - only hold lock during synapse creation
            // Use src_area.cortical_id (the actual ID stored in ConnectomeManager)
            let src_cortical_id = &src_area.cortical_id;
            let src_cortical_id_str = src_cortical_id.to_string(); // For logging
            let synapses_created = {
                let manager_arc = self.connectome_manager.clone();
                let mut manager = manager_arc.write();
                manager.apply_cortical_mapping(src_cortical_id)
            }; // Lock released immediately

            match synapses_created {
                Ok(count) => {
                    total_synapses_created += count as usize;
                    trace!(
                        target: "feagi-bdu",
                        "Created {} synapses for area {}",
                        count,
                        src_cortical_id_str
                    );
                }
                Err(e) => {
                    // If NPU not connected, estimate count
                    warn!(target: "feagi-bdu","  Failed to create synapses for {}: {} (NPU may not be connected)",
                        src_cortical_id_str, e);
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

        // CRITICAL: Rebuild the NPU synapse index so newly created synapses are visible to
        // queries (e.g. get_outgoing_synapses / synapse counts) and propagation.
        //
        // Note: We do this once at the end for performance.
        let npu_arc = {
            let manager = self.connectome_manager.read();
            manager.get_npu().cloned()
        };
        if let Some(npu_arc) = npu_arc {
            let mut npu_lock = npu_arc
                .lock()
                .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
            npu_lock.rebuild_synapse_index();

            // Refresh cached counts after index rebuild.
            let manager = self.connectome_manager.read();
            manager.update_cached_synapse_count();
        }

        // CRITICAL: Register memory areas with PlasticityExecutor after all mappings are created
        // This ensures memory areas have their complete upstream_cortical_areas lists populated.
        #[cfg(feature = "plasticity")]
        {
            use feagi_evolutionary::extract_memory_properties;
            use feagi_npu_plasticity::{MemoryNeuronLifecycleConfig, PlasticityExecutor};

            let manager = self.connectome_manager.read();
            if let Some(executor) = manager.get_plasticity_executor() {
                let mut registered_count = 0;

                // Iterate through all cortical areas and register memory areas
                for area_id in manager.get_cortical_area_ids() {
                    if let Some(area) = manager.get_cortical_area(area_id) {
                        if let Some(mem_props) = extract_memory_properties(&area.properties) {
                            let upstream_areas = manager.get_upstream_cortical_areas(area_id);

                            // Ensure FireLedger tracks upstream areas with at least the required temporal depth.
                            // Dense, burst-aligned tracking is required for correct memory pattern hashing.
                            if let Some(npu_arc) = manager.get_npu() {
                                if let Ok(mut npu) = npu_arc.lock() {
                                    let existing_configs = npu.get_all_fire_ledger_configs();
                                    for &upstream_idx in &upstream_areas {
                                        let existing = existing_configs
                                            .iter()
                                            .find(|(idx, _)| *idx == upstream_idx)
                                            .map(|(_, w)| *w)
                                            .unwrap_or(0);

                                        let desired = mem_props.temporal_depth as usize;
                                        let resolved = existing.max(desired);
                                        if resolved != existing {
                                            if let Err(e) = npu.configure_fire_ledger_window(
                                                upstream_idx,
                                                resolved,
                                            ) {
                                                warn!(
                                                    target: "feagi-bdu",
                                                    "Failed to configure FireLedger window for upstream area idx={} (requested={}): {}",
                                                    upstream_idx,
                                                    resolved,
                                                    e
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    warn!(target: "feagi-bdu", "Failed to lock NPU for FireLedger configuration");
                                }
                            }

                            if let Ok(exec) = executor.lock() {
                                let lifecycle_config = MemoryNeuronLifecycleConfig {
                                    initial_lifespan: mem_props.init_lifespan,
                                    lifespan_growth_rate: mem_props.lifespan_growth_rate,
                                    longterm_threshold: mem_props.longterm_threshold,
                                    max_reactivations: 1000,
                                };

                                exec.register_memory_area(
                                    area.cortical_idx,
                                    area_id.as_base_64(),
                                    mem_props.temporal_depth,
                                    upstream_areas.clone(),
                                    Some(lifecycle_config),
                                );

                                registered_count += 1;
                            }
                        }
                    }
                }
                let _ = registered_count; // count retained for future metrics if needed
            }
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
    src_area: &CorticalArea,
    genome: &feagi_evolutionary::RuntimeGenome,
) -> usize {
    let dstmap = match src_area.properties.get("cortical_mapping_dst") {
        Some(serde_json::Value::Object(map)) => map,
        _ => return 0,
    };

    let mut total = 0;

    for (dst_id, rules) in dstmap {
        // Convert string dst_id to CorticalID for lookup
        let dst_cortical_id = match feagi_evolutionary::string_to_cortical_id(dst_id) {
            Ok(id) => id,
            Err(_) => continue,
        };
        let dst_area = match genome.cortical_areas.get(&dst_cortical_id) {
            Some(area) => area,
            None => continue,
        };

        let rules_array = match rules.as_array() {
            Some(arr) => arr,
            None => continue,
        };

        for rule in rules_array {
            let morphology_id = rule
                .get("morphology_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let scalar = rule
                .get("morphology_scalar")
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as usize;

            // Simplified estimation
            let src_per_voxel = src_area
                .properties
                .get("neurons_per_voxel")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as usize;
            let dst_per_voxel = dst_area
                .properties
                .get("neurons_per_voxel")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as usize;

            let src_voxels =
                src_area.dimensions.width * src_area.dimensions.height * src_area.dimensions.depth;
            let dst_voxels =
                dst_area.dimensions.width * dst_area.dimensions.height * dst_area.dimensions.depth;

            let src_neurons = src_voxels as usize * src_per_voxel;
            let dst_neurons = dst_voxels as usize * dst_per_voxel as usize;

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
    /// Calculate position for autogen region based on root region's bounding box
    fn calculate_autogen_region_position(
        root_area_ids: &[CorticalID],
        genome: &feagi_evolutionary::RuntimeGenome,
    ) -> [i32; 3] {
        if root_area_ids.is_empty() {
            return [100, 0, 0];
        }

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        let mut min_z = i32::MAX;
        let mut max_z = i32::MIN;

        for cortical_id in root_area_ids {
            if let Some(area) = genome.cortical_areas.get(cortical_id) {
                let pos: (i32, i32, i32) = area.position.into();
                let dims = (
                    area.dimensions.width as i32,
                    area.dimensions.height as i32,
                    area.dimensions.depth as i32,
                );

                min_x = min_x.min(pos.0);
                max_x = max_x.max(pos.0 + dims.0);
                min_y = min_y.min(pos.1);
                max_y = max_y.max(pos.1 + dims.1);
                min_z = min_z.min(pos.2);
                max_z = max_z.max(pos.2 + dims.2);
            }
        }

        let bbox_width = (max_x - min_x).max(1);
        let padding = (bbox_width / 5).max(50);
        let autogen_x = max_x + padding;
        let autogen_y = (min_y + max_y) / 2;
        let autogen_z = (min_z + max_z) / 2;

        info!(target: "feagi-bdu",
              "  📐 Autogen position: ({}, {}, {}) [padding: {}]",
              autogen_x, autogen_y, autogen_z, padding);

        [autogen_x, autogen_y, autogen_z]
    }

    /// Analyze region inputs/outputs based on cortical connections
    ///
    /// Following Python's _auto_assign_region_io() logic:
    /// - OUTPUT: Any area in the region that connects to an area OUTSIDE the region
    /// - INPUT: Any area in the region that receives connection from OUTSIDE the region
    fn analyze_region_io(
        region_area_ids: &[feagi_structures::genomic::cortical_area::CorticalID],
        all_cortical_areas: &std::collections::HashMap<CorticalID, CorticalArea>,
    ) -> (Vec<String>, Vec<String>) {
        let area_set: std::collections::HashSet<_> = region_area_ids.iter().cloned().collect();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Helper to extract destination area IDs from cortical_mapping_dst (as strings)
        let extract_destinations = |area: &CorticalArea| -> Vec<String> {
            area.properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .map(|obj| obj.keys().cloned().collect())
                .unwrap_or_default()
        };

        // Find OUTPUTS: areas in region that connect to areas OUTSIDE region
        for area_id in region_area_ids {
            if let Some(area) = all_cortical_areas.get(area_id) {
                let destinations = extract_destinations(area);
                // Convert destination strings to CorticalID for comparison
                let external_destinations: Vec<_> = destinations
                    .iter()
                    .filter_map(|dest| feagi_evolutionary::string_to_cortical_id(dest).ok())
                    .filter(|dest_id| !area_set.contains(dest_id))
                    .collect();

                if !external_destinations.is_empty() {
                    outputs.push(area_id.as_base_64());
                }
            }
        }

        // Find INPUTS: areas in region receiving connections from OUTSIDE region
        for (source_area_id, source_area) in all_cortical_areas.iter() {
            // Skip areas that are inside the region
            if area_set.contains(source_area_id) {
                continue;
            }

            let destinations = extract_destinations(source_area);
            for dest_str in destinations {
                if let Ok(dest_id) = feagi_evolutionary::string_to_cortical_id(&dest_str) {
                    if area_set.contains(&dest_id) {
                        let dest_string = dest_id.as_base_64();
                        if !inputs.contains(&dest_string) {
                            inputs.push(dest_string);
                        }
                    }
                }
            }
        }

        (inputs, outputs)
    }

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
    use feagi_evolutionary::create_genome_with_core_morphologies;
    use feagi_structures::genomic::cortical_area::CorticalAreaDimensions;

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
        ConnectomeManager::reset_for_testing(); // Ensure clean state
        let manager = ConnectomeManager::instance();
        let mut neuro = Neuroembryogenesis::new(manager.clone());

        // Create a minimal genome with one cortical area
        let mut genome = create_genome_with_core_morphologies(
            "test_genome".to_string(),
            "Test Genome".to_string(),
        );

        let cortical_id = CorticalID::try_from_bytes(b"cst_neur").unwrap(); // Use valid custom cortical ID
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("Failed to get cortical type");
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .expect("Failed to create cortical area");
        genome.cortical_areas.insert(cortical_id, area);

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
        // Note: Due to parallel test execution with shared singleton, we just verify the area exists
        assert!(
            mgr.has_cortical_area(&cortical_id),
            "Cortical area should have been added to connectome"
        );

        println!("✅ Development completed in {}ms", progress.duration_ms);
    }
}
