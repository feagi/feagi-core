use feagi_structures::feagi_data::create_quantized_index_count_wrapper;

/// Denotes the number of microseconds a change in burst engine state took
create_quantized_index_count_wrapper!(NPUWrappedBurstEngineMicroSecondsElapsed);

#[repr(u8)]
pub enum BurstEngineJustCompletedPhase
{
    /// The start of a burst right before the neurons start acting upon their inputs and running
    /// their models. The Engine Burst Index Increments by one, or if it is about to overflow, will
    /// reset to half the max possible uint value
    BurstCounterIndexIncrement,

    /// If the burst index is about to roll over, some neuron models may need to update
    /// their internal state to account for this. This runs that function for neuron models with
    /// that flag
    NeuronModelUpdatedForBurstIndexRollover,

    /// If the burst index is about to roll over, some synapse models may need to update
    /// their internal state to account for this. This runs that function for synapse models with
    /// that flag
    SynapseModelUpdatedForBurstIndexRollover,

    /// Optional dependent on engine. Consolidates any active (non zero) from the sparse FCL array
    /// into a dense one to iterate for neuron dynamics
    FCLConsolidation,

    /// A selection of neurons have received inputs. During this phase, those
    /// neurons shall process their input as per their Neuron Model definitions, and update their
    /// membrane potential and if they are firing.
    NeuronDynamics,

    /// A dense bitpacked bitfield of what neurons are firing have been updated using the per
    /// neuron firing data before. Is required due to the following phase
    UpdateFiringNeuronBitfield,

    /// Per cortical_area area that needs it, counts the number of firing neurons and updates the
    /// internal value of that area. This is important due to being required for some mappings
    /// down stream of a cortical_area area
    CountFiringNeuronsPerCorticalArea,

    /// In this optional phase, if there is sensor / motor / visualization data to take in and out,
    /// it is to be done during this phase. If any incoming firing neuron data is coming from
    /// another burst engine, it is injected here too
    PreSynapseDataExchange,
    
    /// Following neuron firings, which is denoted in the complete neuron array in a sparse fashion,
    /// firing neurons are sorted in a separate array into a dense firing array. Dependent on
    /// engine, and on the updating bitfield phase. Not used by all engines.
    FiringNeuronConsolidation,
    
    /// Firing neurons pass their data along their synapses which mutate the firing potential,
    /// and is mapped to downstream neurons either to their FCL input directly or to a FCLC
    SynapseDynamics,

    /// An optional phase for if data that can from another burst engines synapse dynamics must
    /// be copied back here
    PostSynapseDataExchange,

    /// An optional phase that only runs if an FCLC (or more) exist. Consolidates amny inputs into
    /// a single FCL per neuron
    FCLCConsolidation,
}
