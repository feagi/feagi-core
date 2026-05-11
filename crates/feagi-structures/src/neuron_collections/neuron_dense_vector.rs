use crate::base_feagi_types::quantizable_types::{
    FeagiBaseSingleElementQuantizationType, QuantizableUIntType,
};
use crate::neuron_collections::neuron_structs::{
    NeuronDensityPerVoxel, NeuronIndexCount, NeuronMembranePotential,
    SingleCorticalNeuronCollectionType,
};
use crate::neuron_collections::traits::{
    SingleCorticalNeuronCollectionBase, SingleCorticalNeuronCollectionDense,
};
use crate::neuron_voxel_collections::voxel_structs::{
    NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount,
};
use crate::quantization_level::CorticalAreaNeuronQuantization;
use core::marker::PhantomData;
use crate::neuron_collections::FeagiStructuresNeuronError;

pub struct NeuronDenseVector<CANQ: CorticalAreaNeuronQuantization> {
    potentials: Vec<NeuronMembranePotential<CANQ::NeuronValueQuant>>,
    cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_density_per_voxel: NeuronDensityPerVoxel,
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronDenseVector<CANQ> {
    pub fn new(
        dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
        neuron_density_per_voxel: NeuronDensityPerVoxel,
    ) -> Result<Self, FeagiStructuresNeuronError> {
        let number_neurons: NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> =
            dimensions.get_number_neurons(&neuron_density_per_voxel);
        Ok(Self {
            potentials: vec![NeuronMembranePotential::ZERO; number_neurons.to_usize()],
            cortical_dimensions: dimensions,
            neuron_density_per_voxel,
        })
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> SingleCorticalNeuronCollectionBase<CANQ>
    for NeuronDenseVector<CANQ>
{
    const COLLECTION_TYPE: SingleCorticalNeuronCollectionType =
        SingleCorticalNeuronCollectionType::DenseArray;

    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel {
        self.neuron_density_per_voxel
    }

    fn is_single_neuron_per_voxel(&self) -> bool {
        self.neuron_density_per_voxel == NeuronDensityPerVoxel::ONE
    }

    fn get_representing_cortical_area_dimensions(
        &self,
    ) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant> {
        &self.cortical_dimensions
    }

    fn get_neuron_max_index(&self) -> NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.cortical_dimensions.get_number_neurons(&self.neuron_density_per_voxel)
    }

    fn get_number_contained_neurons(&self) -> NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.cortical_dimensions.get_number_neurons(&self.neuron_density_per_voxel)
    }

    fn get_number_contained_voxels(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        self.cortical_dimensions.get_number_voxels()
    }

    fn iter_index(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .iter()
            .enumerate()
            .map(|(i, &potential)| (NeuronIndexCount::from_usize(i), potential))
    }

    #[cfg(feature = "rayon")]
    fn iter_index_par(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .iter()
            .enumerate()
            .map(|(i, &potential)| (NeuronIndexCount::from_usize(i), potential))
    }

    fn iter_nonzero_potential_index(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .iter()
            .enumerate()
            .filter(|(_, &potential)| potential != NeuronMembranePotential::ZERO)
            .map(|(i, &potential)| (NeuronIndexCount::from_usize(i), potential))
    }

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_index_par(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        self.potentials
            .iter()
            .enumerate()
            .filter(|(_, &potential)| potential != NeuronMembranePotential::ZERO)
            .map(|(i, &potential)| (NeuronIndexCount::from_usize(i), potential))
    }

    fn iter_coordinate(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        let density = self.neuron_density_per_voxel.get().to_usize();
        let dimensions = &self.cortical_dimensions;
        self.potentials
            .iter()
            .enumerate()
            .map(move |(i, &potential)| {
                let voxel_index = i / density;
                let voxel_coord = dimensions.linear_index_to_standard_voxel_coordinate(
                    NeuronVoxelIndexCount::from_usize(voxel_index),
                );
                (voxel_coord, potential)
            })
    }

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        let density = self.neuron_density_per_voxel.get().to_usize();
        let dimensions = &self.cortical_dimensions;
        self.potentials
            .iter()
            .enumerate()
            .map(move |(i, &potential)| {
                let voxel_index = i / density;
                let voxel_coord = dimensions.linear_index_to_standard_voxel_coordinate(
                    NeuronVoxelIndexCount::from_usize(voxel_index),
                );
                (voxel_coord, potential)
            })
    }

    fn iter_nonzero_potential_coordinate(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        let density = self.neuron_density_per_voxel.get().to_usize();
        let dimensions = &self.cortical_dimensions;
        self.potentials
            .iter()
            .enumerate()
            .filter(|(_, &potential)| potential != NeuronMembranePotential::ZERO)
            .map(move |(i, &potential)| {
                let voxel_index = i / density;
                let voxel_coord = dimensions.linear_index_to_standard_voxel_coordinate(
                    NeuronVoxelIndexCount::from_usize(voxel_index),
                );
                (voxel_coord, potential)
            })
    }

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_coordinate_par(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>,
            NeuronMembranePotential<CANQ::NeuronValueQuant>,
        ),
    > {
        let density = self.neuron_density_per_voxel.get().to_usize();
        let dimensions = &self.cortical_dimensions;
        self.potentials
            .iter()
            .enumerate()
            .filter(|(_, &potential)| potential != NeuronMembranePotential::ZERO)
            .map(move |(i, &potential)| {
                let voxel_index = i / density;
                let voxel_coord = dimensions.linear_index_to_standard_voxel_coordinate(
                    NeuronVoxelIndexCount::from_usize(voxel_index),
                );
                (voxel_coord, potential)
            })
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> SingleCorticalNeuronCollectionDense<CANQ> for NeuronDenseVector<CANQ>
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<CANQ::NeuronValueQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_potentials_mut(
        &mut self,
    ) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>] {
        self.potentials.as_mut_slice()
    }

    fn iter_voxel_neuron_slice(
        &self,
    ) -> impl Iterator<Item = (&[NeuronMembranePotential<CANQ::NeuronValueQuant>])> {
        let density = self.neuron_density_per_voxel.get().to_usize();
        self.potentials.chunks(density).map(|chunk| chunk)
    }

    fn iter_voxel_neuron_slice_par(
        &self,
    ) -> impl Iterator<Item = (&[NeuronMembranePotential<CANQ::NeuronValueQuant>])> {
        let density = self.neuron_density_per_voxel.get().to_usize();
        self.potentials.chunks(density).map(|chunk| chunk)
    }

    // TODO write into a a dense voxel vector
}

