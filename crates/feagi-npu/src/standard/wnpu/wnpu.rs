use feagi_data::neurons::voxel_potentials::wrapped_values::{
    NeuronVoxelCoordinateGenomic, NeuronVoxelDimensionsGenomic,
};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
use feagi_genomic_context::cortical_area::CorticalID;

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
pub struct WrappedNeuronProcessingUnit {
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
        todo!()
    }

    /// Sets the burst rate to the given frequency, resuming the engine if it was paused.
    pub fn run_at_frequency(&mut self, new_frequency: NPUTargetFrequency) -> Result<(), WNPUError> {  // TODO LOAD
        todo!()
    }

    /// Pauses burst calculations without clearing any data; connectome data continues to exist
    /// inside the NPU.
    pub fn pause(&mut self) -> Result<(), WNPUError> {
        todo!()
    }

    /// Terminates the burst engines. Intended for shutdown; the NPU is unusable afterwards.
    pub fn stop(&mut self) -> Result<(), WNPUError> {
        todo!()
    }

    /// Whether the burst engines are currently running, as opposed to paused, failed or stopped.
    pub fn is_running(&self) -> bool {  // TODO LOAD
        todo!()
    }

    /// Number of bursts the engine has completed since the connectome was last cleared.
    pub fn bursts_completed(&self) -> Result<u64, WNPUError> {  // TODO LOAD
        todo!()
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
        todo!()
    }

    /// Applies new parameters to an existing cortical area. If `parameters` changes the dimensions
    /// or neuron density, the area's neurons are reallocated and all mappings touching it are
    /// rebuilt; otherwise the existing neurons are rewritten in place.
    pub fn reconfigure_cortical_area(
        &mut self,
        cortical_id: &CorticalID,
        parameters: CorticalAreaParameters,
    ) -> Result<(), WNPUError> {
        todo!()
    }

    /// Rebinds an existing cortical area to a new `CorticalID`. This only updates the translation
    /// table; the engine is not involved because it addresses areas by index.
    pub fn change_cortical_area_id(
        &mut self,
        current_cortical_id: &CorticalID,
        new_cortical_id: &CorticalID,
    ) -> Result<(), WNPUError> {
        todo!()
    }

    /// Removes a cortical area, its neurons, and every mapping that references it.
    pub fn remove_cortical_area(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Result<CorticalAreaRemovalCounts, WNPUError> {
        todo!()
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
        todo!()
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
    /// individually would cost one engine round trip per structure.
    pub fn apply_connectome_edits( // TODO Investigate
        &mut self,
        edits: Vec<WnpuConnectomeEdit>,
    ) -> Result<Vec<WnpuConnectomeEditOutcome>, WNPUError> {
        todo!()
    }

    /// Discards the entire connectome, leaving the NPU empty and paused. Used when preparing to
    /// load a different genome.
    pub fn clear_connectome(&mut self) -> Result<(), WNPUError> {
        todo!()
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
        todo!()
    }

    /// Every cortical area currently present in the NPU.
    pub fn cortical_area_ids(&self) -> Vec<CorticalID> {
        todo!()
    }

    /// Voxel dimensions of an area, or `None` if the area does not exist.
    pub fn cortical_area_dimensions(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<NeuronVoxelDimensionsGenomic> {
        todo!()
    }

    /// Neurons per voxel for an area, or `None` if the area does not exist.
    pub fn cortical_area_neurons_per_voxel(&self, cortical_id: &CorticalID) -> Option<u32> {
        todo!()
    }

    /// Neuron count of one area, or `None` if the area does not exist.
    pub fn cortical_area_neuron_count(&self, cortical_id: &CorticalID) -> Option<usize> {
        todo!()
    }

    /// Neuron count across the whole connectome.
    pub fn total_neuron_count(&self) -> usize {
        todo!()
    }

    /// Synapse count across the whole connectome.
    pub fn total_synapse_count(&self) -> usize {
        todo!()
    }

    /// Whether a mapping is registered for this source to destination pair.
    pub fn has_cortical_mapping(&self, source: &CorticalID, destination: &CorticalID) -> bool {
        todo!()
    }

    /// Rule set currently registered for a mapping, or `None` if there is no such mapping.
    pub fn cortical_mapping_rules(
        &self,
        source: &CorticalID,
        destination: &CorticalID,
    ) -> Option<Vec<CorticalMappingRuleParameters>> {
        todo!()
    }

    /// Synapses created by one mapping, or `None` if there is no such mapping.
    pub fn cortical_mapping_synapse_count(
        &self,
        source: &CorticalID,
        destination: &CorticalID,
    ) -> Option<usize> {
        todo!()
    }

    /// Destination areas this area maps to.
    pub fn cortical_mapping_destinations(&self, source: &CorticalID) -> Vec<CorticalID> {
        todo!()
    }

    /// Source areas that map into this area.
    pub fn cortical_mapping_sources(&self, destination: &CorticalID) -> Vec<CorticalID> {
        todo!()
    }

    /// Total synapses leaving an area across all of its outgoing mappings.
    pub fn cortical_area_outgoing_synapse_count(&self, cortical_id: &CorticalID) -> usize {
        todo!()
    }

    /// Total synapses entering an area across all of its incoming mappings.
    pub fn cortical_area_incoming_synapse_count(&self, cortical_id: &CorticalID) -> usize {
        todo!()
    }

    // ==================================================================================
    // Runtime probes
    //
    // These require an engine round trip and return owned snapshots keyed by CorticalID.
    // A probe blocks until the engine reaches a safe point, so they are unsuitable for
    // paths that must never wait on the burst loop.
    // ==================================================================================

    // NOTE: Not including this in the new arch for now!

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
    // ==================================================================================

    /// Subscribes an agent to some NPU data and vice versa.
    pub fn subscribe_agent_to_npu(&mut self, subscription_details: ()) -> Result<(), WNPUError> {
        todo!()
    }

    /// Returns a subscription of an agent, which allows the NPU to free the memory.
    pub fn unsubscribe_agent_from_npu(
        &mut self,
        subscription_details: (),
    ) -> Result<(), WNPUError> {
        todo!()
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
#[derive(Debug, Clone)]
pub struct CorticalAreaParameters {
    pub dimensions: NeuronVoxelDimensionsGenomic,
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
