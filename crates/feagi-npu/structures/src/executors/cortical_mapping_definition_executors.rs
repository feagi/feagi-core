use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::neuron::dimensional_neurons::shared_structs::DimensionalNeuronCorticalData;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUNeuronIndex, NPUQuantization, SynapseCount};
use crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull;
// NOTE: We cannot enum this as then we cannot add types outside of this crate

/// Allows custom mapping of synapses from source neurons to destination neurons
pub trait NonPlasticCorticalMappingDefinitionExecutor<Q: NPUQuantization>
{
    // NOTE: NPUNeuronIndex is global along all dimensional neurons, ergo you may see a range like
    // neurons 64-96, which contains 32 neurons within the given dimensions and density!

    /// Creates an iterator to create synapses, and returns the count of the total number of synapses that will be made
    fn non_plastic_synapse_iterator(&self,
                                    source_area_type: DimensionCorticalAreaType,
                                    source_cortical_data: &DimensionalNeuronCorticalData<Q>,
                                    source_neuron_flags: &[NeuronFlag],
                                    destination_area_type: DimensionCorticalAreaType,
                                    destination_cortical_data: &DimensionalNeuronCorticalData<Q>,
                                    destination_neuron_flags: &[NeuronFlag])
        -> Result<(impl Iterator<Item=NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, SynapseCount<Q::SynapseIndex>), FeagiStructuresError>;

    // TODO we should have a check to ensure that the count matches that runs in debug mode ( do this as a check natively)

}