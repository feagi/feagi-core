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
/// # Scope
///
/// This surface is deliberately narrow: it exposes only what the NPU itself owns — burst
/// lifecycle and frequency, cortical-area create/edit/remove, cortical mappings, the registry
/// queries over those two, runtime probes / sensory injection into live neuron state, and the
/// NPU-side registration of agent data channels. Everything else that the legacy architecture
/// still owns — genome load/validate/export, the brain-region hierarchy, morphology definitions,
/// system health, snapshots, connectome transport, and the agent registry with its network
/// connections — stays in the legacy Brain Development code above WNPU and is not mirrored here.
///
/// # Placeholder status
///
/// Every method on this type is currently a **placeholder**: the surface exists so callers can be
/// wired against the stable signatures now, but the returned values are hard-coded defaults
/// (`Ok(())`, empty vectors, zero counts, `is_running` mirrored from the last lifecycle call,
/// ...). The only state that changes at runtime is the small placeholder bookkeeping declared on
/// [`WrappedNeuronProcessingUnit`]. Real implementations will replace these bodies in place; the
/// signatures are stable so callers above WNPU do not need to move again.
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

    /// Creates or replaces the mapping identified by `name`, wiring `source` to `destination` with
    /// the given rule set. `name` is the mapping's external identifier: callers reference it by
    /// name for every later query and for removal, and it must be unique across mappings. The NPU
    /// keys the mapping by an internal index that is never exposed on this surface. Replacement is
    /// atomic inside the engine, so stale synapses from removed or edited rules cannot survive.
    /// Passing an empty rule set is equivalent to [`Self::remove_cortical_mapping`].
    pub fn set_cortical_mapping(  // TODO LOAD
        &mut self,
        name: &str,
        source: &CorticalID,
        destination: &CorticalID,
        rules: Vec<CorticalMappingRuleParameters>,
    ) -> Result<CorticalMappingSynapseChange, WNPUError> {
        Ok(CorticalMappingSynapseChange{ synapses_created: 0, synapses_removed: 0 })
    }

    /// Legacy REST-shaped mapping update: takes an opaque JSON array so the adapter above WNPU can
    /// forward `POST /v1/connectome/cortical_mapping` payloads verbatim. `name` is the mapping's
    /// external identifier (see [`Self::set_cortical_mapping`]); the adapter supplies it so the
    /// resulting mapping can later be removed by name. Returns the number of synapses the mapping
    /// produced. Placeholder: no synapses are produced.
    pub fn apply_cortical_mapping_update(
        &mut self,
        name: &str,
        source: &CorticalID,
        destination: &CorticalID,
        mapping_data: Vec<serde_json::Value>,
    ) -> Result<usize, WNPUError> {
        Ok(0)
    }

    /// Removes the mapping identified by `name` and all synapses it created, returning how many
    /// synapses were removed. The NPU resolves the name to its internal index and clears it; the
    /// source and destination are recovered from the stored mapping, so the caller need not repeat
    /// them.
    pub fn remove_cortical_mapping(
        &mut self,
        name: &str,
    ) -> Result<usize, WNPUError> {
        Ok(0)
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

    /// Whether a mapping with this name is registered.
    pub fn has_cortical_mapping(&self, name: &str) -> bool {
        false
    }

    /// Rule set currently registered for the mapping named `name`, or `None` if there is no such
    /// mapping.
    pub fn cortical_mapping_rules(
        &self,
        name: &str,
    ) -> Option<Vec<CorticalMappingRuleParameters>> {
        None
    }

    /// Synapses created by the mapping named `name`, or `None` if there is no such mapping.
    pub fn cortical_mapping_synapse_count(
        &self,
        name: &str,
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
    // Agent data exchange
    //
    // The NPU side of agent data exchange: registering an agent as a data source/sink returns the
    // channels used to feed data in and read data out. Only that channel-level registration lives
    // here. The agent registry itself — network connections, transport negotiation, agent
    // identity/properties — is owned by the legacy code above WNPU, not by the NPU.
    //
    // Subscribe/unsubscribe currently accept `()` because the eventual channel-handle type has not
    // been designed yet. Placeholder implementations acknowledge the request without doing
    // anything.
    // ==================================================================================

    /// Registers an agent as a data source/sink on the NPU, wiring up the channels used to feed
    /// data in and read data out. Placeholder: accepts every request.
    pub fn subscribe_agent_to_npu(&mut self, subscription_details: ()) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Tears down an agent's data channels, allowing the NPU to free the associated memory.
    /// Placeholder: accepts every request.
    pub fn unsubscribe_agent_from_npu(
        &mut self,
        subscription_details: (),
    ) -> Result<(), WNPUError> {
        Ok(())
    }

    /// Manual stimulation, as `{cortical_id_base64 -> [[x, y, z, potential], ...]}`. Placeholder:
    /// acknowledges every entry (returns the total neuron count that would have been stimulated).
    pub fn manual_stimulate(
        &mut self,
        stimulation_payload: HashMap<String, Vec<Vec<i32>>>,
    ) -> Result<usize, WNPUError> {
        Ok(stimulation_payload.values().map(|v| v.len()).sum())
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
        /// External identifier of the mapping; see [`WrappedNeuronProcessingUnit::set_cortical_mapping`].
        name: String,
        source: CorticalID,
        destination: CorticalID,
        rules: Vec<CorticalMappingRuleParameters>,
    },
    RemoveCorticalMapping {
        /// External identifier of the mapping to remove.
        name: String,
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

