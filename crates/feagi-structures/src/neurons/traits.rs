use serde_json::Number;
use crate::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxels::descriptors::{NeuronVoxelCount, NeuronVoxelDimensions, NeuronVoxelIndex};
use crate::neurons::descriptors::{NeuronCount, NeuronIndex, NeuronMembranePotential, NumberNeuronsPerVoxel};

pub trait SingleCorticalNeuronCollectionBase<PotentialQuant, CoordQuant, IndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    IndexQuant: QuantizableUIntType
{
    fn get_neuron_voxel_density(&self) -> NeuronCount<NumberNeuronsPerVoxel>;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant>;

    fn neuron_index_max_limit(&self) -> NeuronIndex<IndexQuant>;

    fn neuron_voxel_index_max_limit(&self) -> NeuronVoxelIndex<IndexQuant>;

    fn number_of_voxels(&self) -> NeuronVoxelCount<IndexQuant>;
}

pub trait SingleCorticalNeuronCollectionDense<PotentialQuant, CoordQuant, IndexQuant>:
SingleCorticalNeuronCollectionBase<PotentialQuant, CoordQuant, IndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    IndexQuant: QuantizableUIntType
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<PotentialQuant>];

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<PotentialQuant>];

    // TODO iterators?

    // TODO par iterators?
}