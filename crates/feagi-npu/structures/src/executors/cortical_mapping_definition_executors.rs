use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUNeuronIndex};
use crate::synapse::non_plastic::NonPlasticSynapseFull;
// NOTE: We cannot enum this as then we cannot add types outside of this crate

/// Allows custom mapping of synapses from source neurons to destination neurons
pub trait NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant> where
    NeuronIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,

{
    // NOTE: NPUNeuronIndex is global along all dimensional neurons, ergo you may see a range like
    // neurons 64-96, which contains 32 neurons within the given dimensions and density!
    
    
    fn non_plastic_synapse_iterator(&self,
                                    source_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                    source_neuron_flags: &[NeuronFlag],
                                    source_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                                    source_neuron_density: NumberNeuronsPerVoxel,
                                    destination_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                    destination_neuron_flags: &[NeuronFlag],
                                    destination_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                                    destination_neuron_density: NumberNeuronsPerVoxel)
        -> Result<impl Iterator<Item=NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, FeagiNPUNeuronError>;
}