use crate::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxels::descriptors::NeuronVoxelDimensions;
use crate::neurons::descriptors::{NeuronMembranePotential, NumberNeuronsPerVoxel};

pub trait SingleCorticalNeuronCollectionBase<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_neuron_voxel_density(&self) -> NumberNeuronsPerVoxel;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant>;

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant;
}

pub trait SingleCorticalNeuronCollectionDense<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant>:
SingleCorticalNeuronCollectionBase<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<PotentialQuant>];

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<PotentialQuant>];

    // TODO iterators?

    // TODO par iterators?
}