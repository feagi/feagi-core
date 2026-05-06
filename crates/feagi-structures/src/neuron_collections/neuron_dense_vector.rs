use core::marker::PhantomData;
use crate::base_feagi_types::quantizable_types::{FeagiBaseSingleElementQuantizationType, QuantizableUIntType};
use crate::neuron_voxel_collections::data_values::{NeuronVoxelDimensions, NeuronVoxelIndexCount};
use crate::neuron_collections::data_values::{NeuronDensityPerVoxel, NeuronMembranePotential, NeuronIndexCount};
use crate::neuron_collections::FeagiStructuresNeuronError;
use crate::neuron_collections::traits::{
    NeuronCollectionQuantizationLevelType, SingleCorticalNeuronCollectionBase,
    SingleCorticalNeuronCollectionDense,
};

pub struct NeuronDenseVector<NCQL: NeuronCollectionQuantizationLevelType>
{
    potentials: Vec<NeuronMembranePotential<NCQL::NeuronPotentialQuant>>,
    cortical_dimensions: NeuronVoxelDimensions<NCQL::VoxelCoordQuant>,
    neuron_density_per_voxel: NeuronDensityPerVoxel,
}

impl<NCQL: NeuronCollectionQuantizationLevelType> NeuronDenseVector<NCQL>
{
    pub fn new(
        dimensions: NeuronVoxelDimensions<NCQL::VoxelCoordQuant>,
        neuron_density_per_voxel: NeuronDensityPerVoxel,
    ) -> Result<Self, FeagiStructuresNeuronError> {

        let number_neurons: NeuronIndexCount<NCQL::NeuronIndexCountQuant> =
            dimensions.get_number_neurons(&neuron_density_per_voxel);
        Ok(Self {
            potentials: vec![NeuronMembranePotential::ZERO; number_neurons.to_usize()],
            cortical_dimensions: dimensions,
            neuron_density_per_voxel,
        })
    }
}

impl<NCQL: NeuronCollectionQuantizationLevelType> SingleCorticalNeuronCollectionBase<NCQL>
for NeuronDenseVector<NCQL>
{
    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel {
        
        self.neuron_density_per_voxel
    }

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<NCQL::VoxelCoordQuant> {
        &self.cortical_dimensions
    }

    fn number_neurons(&self) -> NeuronIndexCount<NCQL::NeuronIndexCountQuant> {
        self.cortical_dimensions.get_number_neurons::<NCQL::NeuronIndexCountQuant>(&self.neuron_density_per_voxel)
    }

    fn number_voxels(&self) -> NeuronVoxelIndexCount<NCQL::VoxelCoordQuant> {
        self.cortical_dimensions.get_number_voxels()
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