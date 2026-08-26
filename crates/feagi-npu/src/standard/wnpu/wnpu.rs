use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use feagi_data::neurons::wrapped_types::CorticalVoxelDimensionsGenomic;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantizationStandard32Bit;
use crate::standard::npu::neural_processing_unit::NeuronProcessingUnit;
use crate::standard::npu::npu_target_frequency::NPUTargetFrequency;
use crate::standard::wnpu::connectome_request::connectome_request::ConnectomeRequest;
use crate::standard::wnpu::wrapped_neuron_processor_unit_error::WNPUError;

/// Translation layer between the legacy (genome / metadata) architecture and the new NPU.
///
/// The new NPU owns its own engine indexes and its own quantized data layout; neither escapes
/// this type. Every function below is addressed by `CorticalID` so that callers above the NPU
/// never observe an engine index.
///
/// To serve legacy call sites that expect cheap synchronous answers, this type holds duplicate
/// (shadow) state describing what has been submitted to the engine: the `CorticalID` to engine
/// index table, per-area dimensions and neuron counts, and the mapping registry with its synapse
/// counts. Registry queries are answered from that shadow state without contacting the engine.
/// Runtime probes are the only reads that require an engine round trip.
///
/// # Placeholder status
///
/// Every method on this type is currently a **placeholder**: the surface exists so that FEAGI's
/// service-layer adapters can forward every REST/ZMQ/WebSocket call through here without paying a
/// separate migration step, but the returned values are hard-coded defaults (`Ok(())`, empty
/// vectors, zero counts, `is_running` mirrored from the last lifecycle call, ...). The only
/// state that changes at runtime is the small placeholder bookkeeping declared on
/// [`WrappedNeuronProcessingUnit`]. Real implementations will replace these bodies in place; the
/// signatures are stable so the adapters above WNPU do not need to move again.
pub struct WrappedNeuronProcessingUnit {
    npu: NeuronProcessingUnit<NeuronProcessingUnitIndexQuantizationStandard32Bit>,

    /// Placeholder mirror of "the burst engine is turning the connectome over". Real engine state
    /// will replace this once WNPU owns a running burst pool; keeping it here now lets the
    /// service adapters observe the same lifecycle they will observe against the real engine
    /// (`run_at_frequency` → running, `pause` / `stop` → not running) without any of them
    /// hard-coding an expectation.
    is_running_placeholder: bool,
    // TODO owns the NPU, the CorticalID <-> engine index table, and the shadow registry state
}

// Stub phase: parameters are declared for the eventual implementations but not yet consumed.
#[allow(unused_variables)]
impl WrappedNeuronProcessingUnit {
    // ==================================================================================
    // Lifecycle and burst control
    // ==================================================================================

    /// Initializes the given burst engine configurations (including hardware init) and returns a
    /// ready but empty and paused `WrappedNeuronProcessingUnit`.
    pub fn new(
        global_quantization: FeagiIndexQuantizationLevel, // TODO LOAD
        burst_engine_configurations: Vec<()>, // TODO
    ) -> Result<Self, WNPUError> {
        Ok(Self {
            npu: NeuronProcessingUnit::new(),
            is_running_placeholder: false,
        })
    }

    /// Sets the burst rate to the given frequency, resuming the engine if it was paused.
    pub fn run_at_frequency(&mut self, new_frequency: NPUTargetFrequency) -> Result<(), WNPUError> {  // TODO LOAD

        self.is_running_placeholder = true;
        Ok(())
    }

    /// Pauses burst calculations without clearing any data; connectome data continues to exist
    /// inside the NPU.
    pub fn pause(&mut self) -> Result<(), WNPUError> {

        self.is_running_placeholder = false;
        Ok(())
    }

    /// Terminates the burst engines. Intended for shutdown; the NPU is unusable afterwards.
    pub fn stop(&mut self) -> Result<(), WNPUError> {

        self.is_running_placeholder = false;
        Ok(())
    }

    /// Executes exactly one burst and then leaves the engine paused. Placeholder implementation
    /// simply records that a step would have run and returns without turning the engine over.
    pub fn step_once(&mut self) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Whether the burst engines are currently running, as opposed to paused, failed or stopped.
    pub fn is_running(&self) -> bool {  // TODO LOAD

        self.is_running_placeholder
    }

    /// Number of bursts the engine has completed since the connectome was last cleared.
    pub fn bursts_completed(&self) -> Result<u64, WNPUError> {  // TODO LOAD

        Ok(999)
    }

    /// Resets the burst counter to zero without touching the connectome. Placeholder: the counter
    /// is already zero, so this is a no-op.
    pub fn reset_burst_count(&mut self) -> Result<(), WNPUError> {
        Ok(())
    }

    // ==================================================================================
    // Connectome structure: create, edit, remove
    //
    // These accept legacy-shaped parameters and perform the translation into the new
    // request / writer format internally. Returned counts exist so callers can keep their
    // own metadata and statistics in step without probing the engine.
    // ==================================================================================

    /// Creates a cortical area and its neurons in a single step, returning the number of neurons
    /// created. The engine allocates the neurons as part of area creation, so there is no separate
    /// neurogenesis call.
    pub fn add_cortical_area(  // TODO LOAD
        &mut self,
        cortical_id: &CorticalID,
        parameters: CorticalAreaParameters,
    ) -> Result<usize, WNPUError> {

        let a = serde_json::json!(parameters);

        println!("{}", a.to_string());

        Ok(1)
    }

    /// Applies new parameters to an existing cortical area. If `parameters` changes the dimensions
    /// or neuron density, the area's neurons are reallocated and all mappings touching it are
    /// rebuilt; otherwise the existing neurons are rewritten in place.
    pub fn reconfigure_cortical_area(
        &mut self,
        cortical_id: &CorticalID,
        parameters: CorticalAreaParameters,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Applies a set of property-name → value updates to an existing cortical area, mirroring the
    /// legacy REST update surface. Placeholder implementation acknowledges the request without
    /// mutating shadow state; a real implementation will merge these into the area's parameters
    /// and reconfigure the underlying engine slot.
    pub fn apply_cortical_area_property_updates(
        &mut self,
        cortical_id: &CorticalID,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Rebinds an existing cortical area to a new `CorticalID`. This only updates the translation
    /// table; the engine is not involved because it addresses areas by index.
    pub fn change_cortical_area_id(
        &mut self,
        current_cortical_id: &CorticalID,
        new_cortical_id: &CorticalID,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Removes a cortical area, its neurons, and every mapping that references it.
    pub fn remove_cortical_area(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Result<CorticalAreaRemovalCounts, WNPUError> {
        Ok(CorticalAreaRemovalCounts{
            neurons_removed: 0,
            synapses_removed: 0,
        })
    }

    /// Sets the complete rule set for one source to destination mapping, replacing whatever was
    /// previously registered for that pair. Replacement is atomic inside the engine, so stale
    /// synapses from removed or edited rules cannot survive. Passing an empty rule set is
    /// equivalent to [`Self::remove_cortical_mapping`].
    pub fn set_cortical_mapping(  // TODO LOAD
        &mut self,
        source: &CorticalID,
        destination: &CorticalID,
        rules: Vec<CorticalMappingRuleParameters>,
    ) -> Result<CorticalMappingSynapseChange, WNPUError> {
        Ok(CorticalMappingSynapseChange{ synapses_created: 0, synapses_removed: 0 })
    }

    /// Legacy REST-shaped mapping update: takes an opaque JSON array so the adapter above WNPU can
    /// forward `POST /v1/connectome/cortical_mapping` payloads verbatim. Returns the number of
    /// synapses the mapping produced. Placeholder: no synapses are produced.
    pub fn apply_cortical_mapping_update(
        &mut self,
        source: &CorticalID,
        destination: &CorticalID,
        mapping_data: Vec<serde_json::Value>,
    ) -> Result<usize, WNPUError> {
        Ok(0)
    }

    /// Removes a mapping and all synapses it created, returning how many synapses were removed.
    pub fn remove_cortical_mapping(
        &mut self,
        source: &CorticalID,
        destination: &CorticalID,
    ) -> Result<usize, WNPUError> {
        Ok(
            self.set_cortical_mapping(
            source,
            destination,
            vec![])?
                .synapses_removed
        )
    }

    /// Applies many structure edits as one engine transaction, returning one outcome per edit in
    /// submission order. Required for genome load, where submitting each area and mapping
    /// individually would cost one engine round trip per structure. Placeholder: acknowledges
    /// every edit with a zero-count outcome without touching any state.
    pub fn apply_connectome_edits( // TODO Investigate
        &mut self,
        edits: Vec<WnpuConnectomeEdit>,
    ) -> Result<Vec<WnpuConnectomeEditOutcome>, WNPUError> {
        Ok(edits
            .into_iter()
            .map(|edit| match edit {
                WnpuConnectomeEdit::AddCorticalArea { .. } => {
                    WnpuConnectomeEditOutcome::CorticalAreaAdded { neurons_created: 0 }
                }
                WnpuConnectomeEdit::ReconfigureCorticalArea { .. } => {
                    WnpuConnectomeEditOutcome::CorticalAreaReconfigured
                }
                WnpuConnectomeEdit::RemoveCorticalArea { .. } => {
                    WnpuConnectomeEditOutcome::CorticalAreaRemoved(CorticalAreaRemovalCounts {
                        neurons_removed: 0,
                        synapses_removed: 0,
                    })
                }
                WnpuConnectomeEdit::SetCorticalMapping { .. } => {
                    WnpuConnectomeEditOutcome::CorticalMappingSet(CorticalMappingSynapseChange {
                        synapses_created: 0,
                        synapses_removed: 0,
                    })
                }
                WnpuConnectomeEdit::RemoveCorticalMapping { .. } => {
                    WnpuConnectomeEditOutcome::CorticalMappingRemoved { synapses_removed: 0 }
                }
            })
            .collect())
    }

    /// Discards the entire connectome, leaving the NPU empty and paused. Used when preparing to
    /// load a different genome.
    pub fn clear_connectome(&mut self) -> Result<(), WNPUError> {
        Ok(())
    }

    /*
    /// Submits a request already expressed in the new NPU's native format. Callers still on the
    /// legacy data model should use the typed functions above instead.
    pub fn request_connectome_change(
        &mut self,
        connectome_request: ConnectomeRequest,
    ) -> Result<(), WNPUError> {
        todo!()
    }

     */

    // TODO Get cortical properties

    // ==================================================================================
    // Genome I/O
    //
    // Placeholder implementations accept the legacy REST payloads verbatim (JSON strings) so the
    // adapter above WNPU does not need to know how the eventual loader/exporter will decompose
    // them. Nothing is parsed, stored or emitted yet.
    // ==================================================================================

    /// Loads a genome JSON document into the NPU, replacing any current connectome. Placeholder:
    /// accepts any input and returns success without touching state.
    pub fn load_genome_json(&mut self, genome_json: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Serializes the current connectome back into a genome JSON document. Placeholder: returns
    /// an empty object.
    pub fn save_genome_json(&self) -> Result<String, WNPUError> {
        Ok("{}".to_string())
    }

    /// Exports the subtree rooted at `region_id` as a standalone genome JSON document.
    /// Placeholder: returns an empty object.
    pub fn export_region_genome_json(&self, region_id: &str) -> Result<String, WNPUError> {
        Ok("{}".to_string())
    }

    /// Validates a genome JSON document without loading it. Placeholder: reports every document
    /// as valid.
    pub fn validate_genome_json(&self, genome_json: &str) -> Result<bool, WNPUError> {
        Ok(true)
    }

    /// Metadata describing the currently loaded genome. Placeholder: reports an unnamed empty
    /// genome. `simulation_timestep` is populated with the FEAGI default (0.025 s = 40 Hz) rather
    /// than zero, because the service layer derives the runtime burst frequency as
    /// `1 / simulation_timestep` immediately after a genome load — a zero timestep here would
    /// short-circuit that path with a "non-finite frequency" error before any structure edits
    /// can run.
    pub fn genome_metadata(&self) -> WnpuGenomeMetadata {
        WnpuGenomeMetadata {
            genome_id: String::new(),
            genome_title: String::new(),
            version: String::new(),
            simulation_timestep: 0.025,
            genome_num: None,
            genome_timestamp: None,
        }
    }

    // ==================================================================================
    // Registry queries
    //
    // Answered from shadow state. These never touch the engine and never block on it.
    // ==================================================================================

    /// Whether a cortical area with this ID exists in the NPU.
    pub fn has_cortical_area(&self, cortical_id: &CorticalID) -> bool {
        // Placeholder: no shadow state, so nothing exists.
        false
    }

    /// Every cortical area currently present in the NPU.
    pub fn cortical_area_ids(&self) -> Vec<CorticalID> {
        vec![]
    }

    /// Voxel dimensions of an area, or `None` if the area does not exist.
    pub fn cortical_area_dimensions(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<CorticalVoxelDimensionsGenomic> {
        None
    }

    /// Voxel dimensions of an area as a plain `(usize, usize, usize)` triple, or `None` if the
    /// area does not exist. Provided so adapters that report REST-shaped `(x, y, z)` do not need
    /// to unwrap WNPU's quantized wrapper type themselves. Placeholder: mirrors
    /// [`Self::cortical_area_dimensions`] and returns `None` for every ID.
    pub fn cortical_area_dimensions_xyz(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<(usize, usize, usize)> {
        None
    }

    /// Neurons per voxel for an area, or `None` if the area does not exist.
    pub fn cortical_area_neurons_per_voxel(&self, cortical_id: &CorticalID) -> Option<u32> {
        None
    }

    /// Neuron count of one area, or `None` if the area does not exist.
    pub fn cortical_area_neuron_count(&self, cortical_id: &CorticalID) -> Option<usize> {
        None
    }

    /// Neuron count across the whole connectome.
    pub fn total_neuron_count(&self) -> usize {
        0
    }

    /// Synapse count across the whole connectome.
    pub fn total_synapse_count(&self) -> usize {
        0
    }

    /// Whether a mapping is registered for this source to destination pair.
    pub fn has_cortical_mapping(&self, source: &CorticalID, destination: &CorticalID) -> bool {
        false
    }

    /// Rule set currently registered for a mapping, or `None` if there is no such mapping.
    pub fn cortical_mapping_rules(
        &self,
        source: &CorticalID,
        destination: &CorticalID,
    ) -> Option<Vec<CorticalMappingRuleParameters>> {
        None
    }

    /// Synapses created by one mapping, or `None` if there is no such mapping.
    pub fn cortical_mapping_synapse_count(
        &self,
        source: &CorticalID,
        destination: &CorticalID,
    ) -> Option<usize> {
        None
    }

    /// Destination areas this area maps to. Placeholder: no mappings, so no destinations.
    pub fn cortical_mapping_destinations(&self, source: &CorticalID) -> Vec<CorticalID> {
        vec![]
    }

    /// Source areas that map into this area. Placeholder: no mappings, so no sources.
    pub fn cortical_mapping_sources(&self, destination: &CorticalID) -> Vec<CorticalID> {
        vec![]
    }

    /// Total synapses leaving an area across all of its outgoing mappings. Placeholder: zero.
    pub fn cortical_area_outgoing_synapse_count(&self, cortical_id: &CorticalID) -> usize {
        0
    }

    /// Total synapses entering an area across all of its incoming mappings. Placeholder: zero.
    pub fn cortical_area_incoming_synapse_count(&self, cortical_id: &CorticalID) -> usize {
        0
    }

    // ==================================================================================
    // Brain regions
    //
    // Brain-region hierarchy lives above the NPU proper; WNPU still tracks it so adapters can
    // answer `/v1/regions/*` without a separate registry. Placeholder implementations report an
    // empty hierarchy.
    // ==================================================================================

    /// Registers a brain region under `parent_id`, creating a root region when `parent_id` is
    /// `None`. Placeholder: accepts the request without recording it.
    pub fn add_brain_region(
        &mut self,
        region_id: &str,
        parent_id: Option<&str>,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Removes a brain region and every subtree beneath it. Placeholder: no-op.
    pub fn remove_brain_region(&mut self, region_id: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Applies a set of property-name → value updates to an existing brain region. Placeholder:
    /// no-op.
    pub fn update_brain_region(
        &mut self,
        region_id: &str,
        properties: HashMap<String, serde_json::Value>,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Whether a brain region with this ID exists. Placeholder: reports absence.
    pub fn has_brain_region(&self, region_id: &str) -> bool {
        false
    }

    /// Every brain region currently present. Placeholder: none.
    pub fn brain_region_ids(&self) -> Vec<String> {
        vec![]
    }

    /// Root brain region ID (the region with no parent). Placeholder: none.
    pub fn root_brain_region_id(&self) -> Option<String> {
        None
    }

    /// Registry entry for one brain region, or `None` if the region does not exist. Placeholder:
    /// `None`.
    pub fn brain_region_info(&self, region_id: &str) -> Option<WnpuBrainRegionInfo> {
        None
    }

    // ==================================================================================
    // Morphologies
    //
    // Morphologies (connectivity patterns) live in the genome and are resolved into WNPU rule
    // sets by `set_cortical_mapping`. WNPU tracks the registered set so adapters can answer
    // `/v1/morphologies/*`. Placeholder implementations report an empty registry.
    // ==================================================================================

    /// Registers a morphology definition under `morphology_id`. Placeholder: no-op.
    pub fn add_morphology(&mut self, morphology_id: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Replaces the definition for an existing morphology. Placeholder: no-op.
    pub fn update_morphology_definition(
        &mut self,
        morphology_id: &str,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Removes a morphology and rewrites every mapping rule that referenced it. Placeholder:
    /// no-op.
    pub fn remove_morphology(&mut self, morphology_id: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Renames a morphology and updates every mapping rule that referenced the old ID.
    /// Placeholder: no-op.
    pub fn rename_morphology(&mut self, old_id: &str, new_id: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Every morphology currently registered. Placeholder: none.
    pub fn morphology_ids(&self) -> Vec<String> {
        vec![]
    }

    /// Registry entry for one morphology, or `None` if it is not registered. Placeholder: `None`.
    pub fn morphology_info(&self, morphology_id: &str) -> Option<WnpuMorphologyInfo> {
        None
    }

    // ==================================================================================
    // Neuron-level queries
    //
    // The engine addresses neurons by (cortical area, voxel, index-within-voxel); adapters that
    // still use the flat `u64` neuron ID from the legacy REST surface talk through these thin
    // methods. Placeholder implementations report absence for every ID.
    // ==================================================================================

    /// Inserts an individual neuron at the given voxel of the given area. The engine allocates
    /// neurons as part of area creation, so this is only useful for adapters that still expose a
    /// per-neuron insertion path. Placeholder: returns `0` as the freshly-assigned neuron ID.
    pub fn add_neuron_at(
        &mut self,
        cortical_id: &CorticalID,
        coordinates: (u32, u32, u32),
    ) -> Result<u64, WNPUError> {
        Ok(0)
    }

    /// Removes an individual neuron. Placeholder: no-op.
    pub fn remove_neuron(&mut self, neuron_id: u64) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Whether a neuron with this ID exists. Placeholder: reports absence.
    pub fn has_neuron(&self, neuron_id: u64) -> bool {
        false
    }

    /// Resolves a neuron ID into `(cortical_id, (x, y, z))`. Placeholder: `None`.
    pub fn neuron_location(
        &self,
        neuron_id: u64,
    ) -> Option<(CorticalID, (u32, u32, u32))> {
        None
    }

    /// Neuron ID at the given voxel coordinates, or `None` if the voxel is empty. Placeholder:
    /// `None`.
    pub fn neuron_at_voxel(
        &self,
        cortical_id: &CorticalID,
        coordinates: (u32, u32, u32),
    ) -> Option<u64> {
        None
    }

    /// Every neuron ID present in an area, capped at `limit` when supplied. Placeholder: empty.
    pub fn list_neuron_ids_in_area(
        &self,
        cortical_id: &CorticalID,
        limit: Option<usize>,
    ) -> Vec<u64> {
        vec![]
    }

    /// Property snapshot for a single neuron (membrane potential, threshold, refractory
    /// countdown, ...), keyed by the REST field names used at the API surface. Placeholder:
    /// empty.
    pub fn neuron_properties(&self, neuron_id: u64) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    // ==================================================================================
    // Runtime probes
    //
    // These require an engine round trip and return owned snapshots keyed by CorticalID.
    // A probe blocks until the engine reaches a safe point, so they are unsuitable for
    // paths that must never wait on the burst loop. Placeholder implementations return empty /
    // zeroed data.
    // ==================================================================================

    /// Fire-candidate list snapshot from the most recent burst, as `(neuron_id, membrane_potential)`
    /// pairs. Placeholder: empty.
    pub fn fcl_snapshot(&self) -> Vec<(u64, f32)> {
        vec![]
    }

    /// Fire-candidate list snapshot annotated with the cortical index each neuron belongs to.
    /// Placeholder: empty.
    pub fn fcl_snapshot_with_cortical_idx(&self) -> Vec<(u64, u32, f32)> {
        vec![]
    }

    /// Fire queue sample from the most recent burst, keyed by cortical index. Each value is
    /// `(voxel_x, voxel_y, voxel_z, index_within_voxel, membrane_potential)` as parallel arrays.
    /// Placeholder: empty.
    pub fn fire_queue_sample(
        &self,
    ) -> HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)> {
        HashMap::new()
    }

    /// Fire-ledger window configurations, as `(cortical_idx, window_size)` pairs. Placeholder:
    /// empty.
    pub fn fire_ledger_configs(&self) -> Vec<(u32, usize)> {
        vec![]
    }

    /// Sets the fire-ledger window size for a cortical area. Placeholder: no-op.
    pub fn configure_fire_ledger_window(
        &mut self,
        cortical_idx: u32,
        window_size: usize,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Current sampler configuration, as `(sampling_frequency_hz, consumer_type)`. Placeholder:
    /// zeros.
    pub fn fcl_sampler_config(&self) -> (f64, u32) {
        (0.0, 0)
    }

    /// Updates the sampler configuration. Placeholder: no-op.
    pub fn set_fcl_sampler_config(
        &mut self,
        frequency: Option<f64>,
        consumer: Option<u32>,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// FCL sample rate for a specific cortical area (by NPU-internal index). Placeholder: zero.
    pub fn area_fcl_sample_rate(&self, area_id: u32) -> f64 {
        0.0
    }

    /// Updates the FCL sample rate for a specific cortical area. Placeholder: no-op.
    pub fn set_area_fcl_sample_rate(
        &mut self,
        area_id: u32,
        sample_rate: f64,
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Injects sensory activation into an area as `(x, y, z, membrane_potential)` tuples and
    /// returns the number of neurons successfully injected. Placeholder: acknowledges every input
    /// (returns `xyzp_data.len()`).
    pub fn inject_sensory_by_coordinates(
        &mut self,
        cortical_id: &CorticalID,
        xyzp_data: &[(u32, u32, u32, f32)],
    ) -> Result<usize, WNPUError> {
        Ok(xyzp_data.len())
    }

    /// Clears runtime state for a cortical area addressed by NPU-internal index, returning how
    /// many neurons had their state cleared. Placeholder: reports zero neurons cleared.
    pub fn reset_cortical_area_state_by_idx(
        &mut self,
        cortical_idx: u32,
    ) -> Result<usize, WNPUError> {
        Ok(0)
    }

    /*

    /// Neurons that fired during the most recent burst, as
    /// `(area, voxel, index within voxel, membrane potential)`.
    pub fn fired_neurons_last_burst(
        &self,
    ) -> Result<Vec<(CorticalID, NeuronVoxelCoordinateGenomic, u32, f32)>, WNPUError> {
        todo!()
    }

    /// Count of neurons that fired during the most recent burst, per cortical area.
    pub fn fired_neuron_counts_last_burst(&self) -> Result<Vec<(CorticalID, usize)>, WNPUError> {
        todo!()
    }



    /// Membrane potentials for every neuron in one area, as
    /// `(voxel, index within voxel, membrane potential)`.
    pub fn cortical_area_membrane_potentials(
        &self,
        cortical_id: &CorticalID,
    ) -> Result<Vec<(NeuronVoxelCoordinateGenomic, u32, f32)>, WNPUError> {
        todo!()
    }

    /// Runtime state of a single neuron, or `None` if the address does not resolve.
    pub fn neuron_runtime_detail(
        &self,
        cortical_id: &CorticalID,
        voxel: NeuronVoxelCoordinateGenomic,
        index_within_voxel: u32,
    ) -> Result<Option<NeuronRuntimeDetail>, WNPUError> {
        todo!()
    }

    /// Synapses leaving one neuron, as
    /// `(target area, target voxel, target index within voxel, weight, psp)`.
    pub fn neuron_outgoing_synapses(
        &self,
        cortical_id: &CorticalID,
        voxel: NeuronVoxelCoordinateGenomic,
        index_within_voxel: u32,
    ) -> Result<Vec<(CorticalID, NeuronVoxelCoordinateGenomic, u32, f32, f32)>, WNPUError> {
        todo!()
    }

    /// Synapses entering one neuron, as
    /// `(source area, source voxel, source index within voxel, weight, psp)`.
    pub fn neuron_incoming_synapses(
        &self,
        cortical_id: &CorticalID,
        voxel: NeuronVoxelCoordinateGenomic,
        index_within_voxel: u32,
    ) -> Result<Vec<(CorticalID, NeuronVoxelCoordinateGenomic, u32, f32, f32)>, WNPUError> {
        todo!()
    }

     */

    // ==================================================================================
    // System introspection
    //
    // Placeholder implementations return zeroed / empty structures. Real implementations will
    // read from the burst engine's metrics.
    // ==================================================================================

    /// Aggregate health snapshot describing what would be reported through `/v1/system/health`.
    pub fn system_health(&self) -> WnpuSystemHealth {
        WnpuSystemHealth {
            overall_status: "healthy".to_string(),
            components: Vec::new(),
        }
    }

    /// Aggregate runtime statistics.
    pub fn runtime_stats(&self) -> WnpuRuntimeStats {
        WnpuRuntimeStats::default()
    }

    /// Memory usage across the NPU and its shadow state.
    pub fn memory_usage(&self) -> WnpuMemoryUsage {
        WnpuMemoryUsage::default()
    }

    /// Capacity limits for neurons, synapses and cortical areas.
    pub fn capacity(&self) -> WnpuCapacity {
        WnpuCapacity::default()
    }

    // ==================================================================================
    // Agent data exchange
    //
    // Subscribe/unsubscribe currently accept `()` because the eventual subscription record has
    // not been designed yet. Placeholder implementations acknowledge the request without doing
    // anything, and expose enough read-side surface for adapters to answer `/v1/agents/*`.
    // ==================================================================================

    /// Subscribes an agent to some NPU data and vice versa. Placeholder: accepts every request.
    pub fn subscribe_agent_to_npu(&mut self, subscription_details: ()) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Returns a subscription of an agent, which allows the NPU to free the memory. Placeholder:
    /// accepts every request.
    pub fn unsubscribe_agent_from_npu(
        &mut self,
        subscription_details: (),
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// IDs of every subscribed agent. Placeholder: none.
    pub fn subscribed_agent_ids(&self) -> Vec<String> {
        vec![]
    }

    /// Property snapshot for one agent, or `None` if it is not subscribed. Placeholder: `None`.
    pub fn subscribed_agent_properties(&self, agent_id: &str) -> Option<WnpuAgentProperties> {
        None
    }

    /// Shared-memory descriptors for subscribed agents, keyed by agent ID. Placeholder: empty.
    pub fn shared_memory_info(
        &self,
    ) -> HashMap<String, HashMap<String, serde_json::Value>> {
        HashMap::new()
    }

    /// Manual stimulation, as `{cortical_id_base64 -> [[x, y, z, potential], ...]}`. Placeholder:
    /// acknowledges every entry (returns the total neuron count that would have been stimulated).
    pub fn manual_stimulate(
        &mut self,
        stimulation_payload: HashMap<String, Vec<Vec<i32>>>,
    ) -> Result<usize, WNPUError> {
        Ok(stimulation_payload.values().map(|v| v.len()).sum())
    }

    // ==================================================================================
    // Snapshots
    //
    // Placeholder implementations acknowledge snapshot lifecycle calls without persisting
    // anything and report an empty snapshot list.
    // ==================================================================================

    /// Creates a snapshot of the current connectome (and optionally runtime state) and returns
    /// its metadata. Placeholder: fabricates metadata whose `snapshot_id` is a monotonic token
    /// but persists no data.
    pub fn create_snapshot(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        stateful: bool,
    ) -> WnpuSnapshotMetadata {
        WnpuSnapshotMetadata {
            snapshot_id: String::new(),
            created_at: String::new(),
            name: name.unwrap_or_default(),
            description,
            stateful,
            size_bytes: 0,
        }
    }

    /// Restores a previously-created snapshot. Placeholder: no-op.
    pub fn restore_snapshot(&mut self, snapshot_id: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Every snapshot currently retained. Placeholder: none.
    pub fn list_snapshots(&self) -> Vec<WnpuSnapshotMetadata> {
        vec![]
    }

    /// Deletes a snapshot. Placeholder: no-op.
    pub fn delete_snapshot(&mut self, snapshot_id: &str) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Raw artifact bytes for one snapshot in the requested `format` (`json`, `binary`, ...).
    /// Placeholder: empty.
    pub fn snapshot_artifact_bytes(&self, snapshot_id: &str, format: &str) -> Vec<u8> {
        Vec::new()
    }

    // ==================================================================================
    // Connectome import / export (transport-level)
    //
    // The service layer's `ConnectomeSnapshot` type is defined outside WNPU; these functions deal
    // in the raw bytes so WNPU does not need to depend on the service crate. Adapters serialize
    // the DTO around this call.
    // ==================================================================================

    /// Serializes the current connectome to opaque bytes. Placeholder: empty.
    pub fn export_connectome_bytes(&self) -> Result<Vec<u8>, WNPUError> {
        Ok(Vec::new())
    }

    /// Restores a previously-exported connectome from opaque bytes. Placeholder: no-op.
    pub fn import_connectome_bytes(&mut self, bytes: &[u8]) -> Result<(), WNPUError> {
        Ok(())
    }
}

// ======================================================================================
// Legacy-shaped parameter and result types
//
// These mirror the shapes the legacy architecture already produces after it decodes the
// genome JSON. WNPU converts them into the new quantized writer format. They are expected
// to be deleted along with the legacy architecture.
// ======================================================================================

/// Neural parameters for a cortical area, in the legacy unquantized form.
///
/// `dimensions` counts voxels only; `neurons_per_voxel` supplies the density that the new NPU
/// folds into its own four dimensional area description.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorticalAreaParameters {
    pub dimensions: CorticalVoxelDimensionsGenomic,
    pub neurons_per_voxel: u32,
    pub fire_threshold: f32,
    pub fire_threshold_increment_x: f32,
    pub fire_threshold_increment_y: f32,
    pub fire_threshold_increment_z: f32,
    /// Upper bound above which a neuron will not fire. Zero means no limit.
    pub fire_threshold_limit: f32,
    pub leak_coefficient: f32,
    pub resting_potential: f32,
    pub refractory_period: u16,
    pub excitability: f32,
    /// Consecutive fires before the snooze period applies. Zero means no limit.
    pub consecutive_fire_limit: u16,
    pub snooze_period: u16,
    pub membrane_potential_accumulation: bool,
    pub degeneration: f32,
    pub is_psp_uniform: bool,
    pub is_membrane_potential_driven_psp: bool,
}

/// One connectivity rule of a cortical mapping, in the legacy unquantized form.
#[derive(Debug, Clone)]
pub struct CorticalMappingRuleParameters {
    /// Morphology name from the genome, resolved by the NPU into a connectivity pattern.
    pub morphology_id: String,
    pub synapse_weight: f32,
    pub synapse_psp: f32,
    pub synapse_attractivity: u8,
    pub is_inhibitory: bool,
    pub delay_bursts: u8,
    /// Present when the legacy rule carried a plasticity flag.
    pub plasticity: Option<CorticalMappingPlasticityParameters>,
    /// Area whose activity gates propagation through synapses created by this rule.
    pub gate_source: Option<CorticalID>,
}

/// Plasticity settings carried by a legacy mapping rule.
///
/// The new NPU does not treat plastic mappings as a distinct category, so WNPU accepts these to
/// keep legacy call sites intact and discards whatever the engine does not model.
#[derive(Debug, Clone)]
pub struct CorticalMappingPlasticityParameters {
    pub mode: WnpuPlasticityMode,
    pub window: usize,
    pub constant: f32,
    pub ltp_multiplier: f32,
    pub ltd_multiplier: f32,
    pub is_bidirectional: bool,
    pub max_weight: f32,
    pub eligibility_decay_bursts: usize,
    pub learning_rate: Option<f32>,
    pub reward_source: Option<CorticalID>,
    pub punishment_source: Option<CorticalID>,
}

/// Plasticity flavor requested by a legacy mapping rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WnpuPlasticityMode {
    Stdp,
    RewardModulatedStdp,
}

/// Synapse delta produced by setting a mapping, so callers can maintain their own statistics.
#[derive(Debug, Clone, Copy)]
pub struct CorticalMappingSynapseChange {
    pub synapses_created: usize,
    pub synapses_removed: usize,
}

/// Structures discarded when a cortical area is removed.
#[derive(Debug, Clone, Copy)]
pub struct CorticalAreaRemovalCounts {
    pub neurons_removed: usize,
    pub synapses_removed: usize,
}

/// Per-neuron runtime state, for inspection endpoints.
#[derive(Debug, Clone, Copy)]
pub struct NeuronRuntimeDetail {
    pub membrane_potential: f32,
    pub fire_threshold: f32,
    pub refractory_countdown: u16,
    pub consecutive_fire_count: u16,
    pub consecutive_fire_limit: u16,
    pub snooze_countdown: u16,
}

/// One structure edit within a batched transaction.
#[derive(Debug, Clone)]
pub enum WnpuConnectomeEdit {
    AddCorticalArea {
        cortical_id: CorticalID,
        parameters: CorticalAreaParameters,
    },
    ReconfigureCorticalArea {
        cortical_id: CorticalID,
        parameters: CorticalAreaParameters,
    },
    RemoveCorticalArea {
        cortical_id: CorticalID,
    },
    SetCorticalMapping {
        source: CorticalID,
        destination: CorticalID,
        rules: Vec<CorticalMappingRuleParameters>,
    },
    RemoveCorticalMapping {
        source: CorticalID,
        destination: CorticalID,
    },
}

/// Result of one edit within a batched transaction, in submission order.
#[derive(Debug, Clone, Copy)]
pub enum WnpuConnectomeEditOutcome {
    CorticalAreaAdded { neurons_created: usize },
    CorticalAreaReconfigured,
    CorticalAreaRemoved(CorticalAreaRemovalCounts),
    CorticalMappingSet(CorticalMappingSynapseChange),
    CorticalMappingRemoved { synapses_removed: usize },
}

// ======================================================================================
// Extra placeholder types used by the read-only surfaces
//
// These mirror the shapes the REST adapters need so WNPU does not have to depend on
// `feagi-services` (which would create a dependency cycle). Adapters map field-for-field.
// ======================================================================================

/// Metadata for the currently-loaded genome.
#[derive(Debug, Clone, Default)]
pub struct WnpuGenomeMetadata {
    pub genome_id: String,
    pub genome_title: String,
    pub version: String,
    pub simulation_timestep: f64,
    pub genome_num: Option<i32>,
    pub genome_timestamp: Option<i64>,
}

/// Registry entry for one brain region.
#[derive(Debug, Clone, Default)]
pub struct WnpuBrainRegionInfo {
    pub region_id: String,
    pub name: String,
    pub region_type: String,
    pub parent_id: Option<String>,
    pub cortical_areas: Vec<String>,
    pub child_regions: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Registry entry for one morphology definition.
#[derive(Debug, Clone, Default)]
pub struct WnpuMorphologyInfo {
    pub morphology_type: String,
    pub class: String,
    pub parameters: serde_json::Value,
}

/// Overall system health, as reported through `/v1/system/health`.
#[derive(Debug, Clone, Default)]
pub struct WnpuSystemHealth {
    pub overall_status: String,
    pub components: Vec<WnpuComponentHealth>,
}

/// One component within [`WnpuSystemHealth`].
#[derive(Debug, Clone, Default)]
pub struct WnpuComponentHealth {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

/// Aggregate runtime statistics.
#[derive(Debug, Clone, Default)]
pub struct WnpuRuntimeStats {
    pub total_bursts: u64,
    pub total_neurons_fired: u64,
    pub total_processing_time_ms: u64,
    pub avg_burst_time_ms: f64,
    pub avg_neurons_per_burst: f64,
    pub current_rate_hz: f64,
    pub peak_rate_hz: f64,
    pub uptime_seconds: u64,
}

/// Memory usage across NPU and shadow state.
#[derive(Debug, Clone, Default)]
pub struct WnpuMemoryUsage {
    pub npu_neurons_bytes: usize,
    pub npu_synapses_bytes: usize,
    pub npu_total_bytes: usize,
    pub connectome_metadata_bytes: usize,
    pub total_allocated_bytes: usize,
    pub system_total_bytes: usize,
    pub system_available_bytes: usize,
}

/// Capacity limits.
#[derive(Debug, Clone, Default)]
pub struct WnpuCapacity {
    pub current_neurons: usize,
    pub max_neurons: usize,
    pub neuron_utilization_percent: f64,
    pub current_synapses: usize,
    pub max_synapses: usize,
    pub synapse_utilization_percent: f64,
    pub current_cortical_areas: usize,
    pub max_cortical_areas: usize,
}

/// Property snapshot for one subscribed agent.
#[derive(Debug, Clone, Default)]
pub struct WnpuAgentProperties {
    pub agent_type: String,
    pub agent_ip: String,
    pub agent_data_port: u16,
    pub agent_router_address: String,
    pub agent_version: String,
    pub controller_version: String,
    pub capabilities: HashMap<String, serde_json::Value>,
    pub chosen_transport: Option<String>,
}

/// Snapshot metadata.
#[derive(Debug, Clone, Default)]
pub struct WnpuSnapshotMetadata {
    pub snapshot_id: String,
    pub created_at: String,
    pub name: String,
    pub description: Option<String>,
    pub stateful: bool,
    pub size_bytes: u64,
}
