use crate::base_feagi_types::quantizable_types::{
    FeagiBaseSingleElementQuantizationType, QuantizableUIntType,
};
use crate::neuron_collections::individual_neuron_collections::traits::{
    IndividualNeuronCollectionBase, IndividualNeuronCollectionDense,
};
use crate::neuron_collections::individual_neuron_collections::FeagiIndividualNeuronCollectionError;
use crate::quantization_level::CorticalAreaNeuronQuantization;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use crate::neuron_collections::base_neuron_collection_traits::NeuronCollectionBase;
use crate::neuron_collections::common_neuron_structs::{IndividualNeuronIndexCount, IndividualNeuronMembranePotential, NeuronCollectionType, NeuronDensityPerVoxel, NeuronPotentialType, NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelMultiPotentialCalculationMethod, NeuronVoxelPotential};

pub struct IndividualNeuronDenseVector<CANQ: CorticalAreaNeuronQuantization> {
    potentials: Vec<IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>>,
    cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_density_per_voxel: NeuronDensityPerVoxel,
}

impl<CANQ: CorticalAreaNeuronQuantization> IndividualNeuronDenseVector<CANQ> {
    pub fn new(
        dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
        neuron_density_per_voxel: NeuronDensityPerVoxel,
    ) -> Result<Self, FeagiIndividualNeuronCollectionError> {
        let number_neurons: IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> =
            dimensions.get_number_neurons(&neuron_density_per_voxel);
        Ok(Self {
            potentials: vec![IndividualNeuronMembranePotential::ZERO; number_neurons.to_usize()],
            cortical_dimensions: dimensions,
            neuron_density_per_voxel,
        })
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronCollectionBase<CANQ> for IndividualNeuronDenseVector<CANQ> {
    const COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::DenseArray;
    const NEURON_DATA_TYPE: NeuronPotentialType = NeuronPotentialType::IndividualNeuron;

    fn get_representing_cortical_area_voxel_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant> {
        &self.cortical_dimensions
    }

    fn is_single_neuron_per_voxel(&self) -> bool {
        self.neuron_density_per_voxel == NeuronDensityPerVoxel::ONE
    }

    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel {
        self.neuron_density_per_voxel
    }

    fn get_neuron_value_max_index(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.cortical_dimensions
            .get_number_neurons(&self.neuron_density_per_voxel)
    }

    fn get_neuron_voxel_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        todo!()
    }

    fn get_number_contained_neuron_values(&self) -> IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.cortical_dimensions
            .get_number_neurons(&self.neuron_density_per_voxel)
    }

    fn get_number_contained_neuron_voxels(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.cortical_dimensions.get_number_voxels()
    }

    fn iter_voxel_index(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_index_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_index_nonzero_potential(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_index_nonzero_potential_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_coordinate(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_coordinate_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_coordinate_nonzero_potential(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }

    fn iter_voxel_coordinate_nonzero_potential_par(&self, voxel_potential_method: NeuronVoxelMultiPotentialCalculationMethod) -> impl ParallelIterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        todo!()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> IndividualNeuronCollectionBase<CANQ>
    for IndividualNeuronDenseVector<CANQ>
{

    fn iter_individual_neuron_index(
        &self,
    ) -> impl Iterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .iter()
            .enumerate()
            .map(|(i, &potential)| (IndividualNeuronIndexCount::from_usize(i), potential))
    }

    #[cfg(feature = "rayon")]
    fn iter_individual_neuron_index_par(
        &self,
    ) -> impl rayon::iter::ParallelIterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .par_iter()
            .enumerate()
            .map(|(i, &potential)| (IndividualNeuronIndexCount::from_usize(i), potential))
    }

    fn iter_nonzero_potential_neuron_index(
        &self,
    ) -> impl Iterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .iter()
            .enumerate()
            .filter(|(_, &potential)| potential != IndividualNeuronMembranePotential::ZERO)
            .map(|(i, &potential)| (IndividualNeuronIndexCount::from_usize(i), potential))
    }

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_neuron_index_par(
        &self,
    ) -> impl rayon::iter::ParallelIterator<
        Item = (
            IndividualNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .par_iter()
            .enumerate()
            .filter(|(_, &potential)| potential != IndividualNeuronMembranePotential::ZERO)
            .map(|(i, &potential)| (IndividualNeuronIndexCount::from_usize(i), potential))
    }

}

impl<CANQ: CorticalAreaNeuronQuantization> IndividualNeuronCollectionDense<CANQ>
    for IndividualNeuronDenseVector<CANQ>
{
    fn get_all_individual_neuron_potentials(&self) -> &[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_individual_neuron_potentials_mut(
        &mut self,
    ) -> &mut [IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>] {
        self.potentials.as_mut_slice()
    }

    fn iter_voxel_neuron_slice(
        &self,
    ) -> impl Iterator<Item = (&[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>])> {
        let density = self.neuron_density_per_voxel.get().to_usize();
        self.potentials.chunks(density).map(|chunk| chunk)
    }

    fn iter_voxel_neuron_slice_par(
        &self,
    ) -> impl rayon::iter::ParallelIterator<Item = (&[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>])>
    {
        let density = self.neuron_density_per_voxel.get().to_usize();
        self.potentials.par_chunks(density).map(|chunk| chunk)
    }

    // TODO write into a a dense voxel vector
}
