use crate::neuron_collections::base_neuron_collection_traits::NeuronCollectionBase;
use crate::neuron_collections::common_neuron_structs::{IndividualNeuronIndexCount, IndividualNeuronMembranePotential, NeuronVoxelCoordinate};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub trait IndividualNeuronCollectionBase<CANQ: CorticalAreaNeuronQuantization>:
NeuronCollectionBase<CANQ>{
    fn iter_individual_neuron_index(
        &self,
    ) -> impl Iterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    >;

    #[cfg(feature = "rayon")]
    fn iter_individual_neuron_index_par(
        &self,
    ) -> impl rayon::iter::ParallelIterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    >;

    /// Iterate over non-zero potential values by neuron index
    fn iter_nonzero_potential_neuron_index(
        &self,
    ) -> impl Iterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    >;

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_neuron_index_par(
        &self,
    ) -> impl rayon::iter::ParallelIterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    >;
}

pub trait IndividualNeuronCollectionDense<CANQ: CorticalAreaNeuronQuantization>:
    IndividualNeuronCollectionBase<CANQ>
{
    fn get_all_individual_neuron_potentials(&self) -> &[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>];

    fn get_all_individual_neuron_potentials_mut(
        &mut self,
    ) -> &mut [IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>];

    /// Iterate over slices of neurons that would compose a voxel (slice len = density)
    fn iter_voxel_neuron_slice(
        &self,
    ) -> impl Iterator<Item = (&[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>])>;

    #[cfg(feature = "rayon")]
    fn iter_voxel_neuron_slice_par(
        &self,
    ) -> impl rayon::iter::ParallelIterator<Item = (&[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>])>;
}

// TODO Sparse

// TODO Resizable (Vector?)

// TODO Fixed (Array)
