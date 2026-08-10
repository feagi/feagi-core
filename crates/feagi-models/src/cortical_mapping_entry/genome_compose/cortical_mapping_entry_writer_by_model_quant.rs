// TODO this should be macro generated

use crate::cortical_mapping_entry::components::doublet::doublet_iterator_type::DoubletIteratorDimensionalTypeGenomic;
use crate::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::data::UniformSynapseMultiplier;
use crate::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelStandardQuant;

/// This enum defines what cortical mapping entry will be created and how. Different Synapse
/// models have different instantiation parameters and quantization levels, and this handles that
pub enum CorticalMappingEntryWriterByModelQuant {
    Uniform(UniformWriter),
}

// TODO a bit rushed right now, needs some more work as this is a temp solution
pub enum UniformWriter {
    Standard {
        /// Describes the source / destination neuron pairing. Kept in its genomic form because the
        /// concrete iterator cannot be built until both cortical area layouts are known, which is
        /// only true inside the engine.
        doublet: DoubletIteratorDimensionalTypeGenomic,
        /// Scales the potential crossing every synapse of this mapping entry. The Uniform model
        /// applies one weight to the whole entry rather than one per synapse.
        uniform_weight: UniformSynapseMultiplier<<UniformSynapseModelStandardQuant as SynapseModelQuantization>::JunctionPotentialQuant>,
        /// Delay in bursts between the source firing and the potential arriving. 0 means none.
        propagation_delay: u16,
        /// Firing this mapping entry inhibits rather than excites the destination.
        is_inhibitory: bool,
    },
}
