use core::marker::PhantomData;
use crate::base_feagi_types::quantizable_types::{FeagiBaseSingleElementQuantizationType, QuantizableNonzeroUIntType, QuantizableUIntType};
use crate::neuron_voxel_collections::data_values::{NeuronVoxelCount, NeuronVoxelDimensions, NeuronVoxelIndex};
use crate::neuron_collections::data_values::{NeuronCount, NeuronDensityPerVoxel, NeuronIndex, NeuronMembranePotential};
use crate::neuron_collections::FeagiStructuresNeuronError;
use crate::neuron_collections::traits::{
    NeuronCollectionQuantizationLevelType, SingleCorticalNeuronCollectionBase,
    SingleCorticalNeuronCollectionDense,
};

pub struct NeuronDenseVector<NCQL>
where
    NCQL: NeuronCollectionQuantizationLevelType,
{
    potentials: Vec<NeuronMembranePotential<NCQL::NeuronPotentialQuant>>,
    cortical_dimensions: NeuronVoxelDimensions<NCQL::VoxelCoordQuant>,
    number_neurons_per_voxel: NeuronDensityPerVoxel,
    _quantization_level: PhantomData<NCQL>,
}

impl<NCQL> NeuronDenseVector<NCQL>
where
    NCQL: NeuronCollectionQuantizationLevelType,
{
    pub fn new(
        dimensions: NeuronVoxelDimensions<NCQL::VoxelCoordQuant>,
        density: NeuronDensityPerVoxel,
    ) -> Result<Self, FeagiStructuresNeuronError> {

        let number_neurons: NeuronCount<NCQL::NeuronIndexCountQuant> =
            dimensions.get_number_neurons(&density);
        Ok(Self {
            potentials: vec![NeuronMembranePotential::ZERO; number_neurons.to_usize()],
            cortical_dimensions: dimensions,
            number_neurons_per_voxel: density,
            _quantization_level: PhantomData,
        })
    }
}

impl<NCQL> SingleCorticalNeuronCollectionBase<NCQL> for NeuronDenseVector<NCQL>
where
    NCQL: NeuronCollectionQuantizationLevelType,
{
    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel {
        
        self.number_neurons_per_voxel
    }

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<NCQL::VoxelCoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronIndex<NCQL::NeuronIndexCountQuant> {
        NeuronIndex::from_usize(
            self.cortical_dimensions
                .get_number_neurons::<NCQL::NeuronIndexCountQuant>(&self.number_neurons_per_voxel)
                .to_usize(),
        )
    }

    fn neuron_voxel_index_max_limit(&self) -> NeuronVoxelIndex<NCQL::NeuronIndexCountQuant> {
        NeuronVoxelIndex::from_usize(
            self.cortical_dimensions
                .get_number_voxels::<NCQL::NeuronIndexCountQuant>()
                .to_usize(),
        )
    }

    fn number_of_voxels(&self) -> NeuronVoxelCount<NCQL::NeuronIndexCountQuant> {
        let number: NeuronVoxelCount<NCQL::NeuronIndexCountQuant> =
            self.cortical_dimensions.get_number_voxels();
        number
    }
}

impl<NCQL> SingleCorticalNeuronCollectionDense<NCQL> for NeuronDenseVector<NCQL>
where
    NCQL: NeuronCollectionQuantizationLevelType,
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<NCQL::NeuronPotentialQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<NCQL::NeuronPotentialQuant>] {
        self.potentials.as_mut_slice()
    }

    // TODO write into a a dense voxel vector
}

// TODO static

// Note: no need for a sparse trait. The only implementation is with indexing