use core::marker::PhantomData;
use crate::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxels::descriptors::{NeuronVoxelCount, NeuronVoxelDimensions, NeuronVoxelIndex};
use crate::neurons::descriptors::{NeuronCount, NeuronIndex, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::neurons::FeagiStructuresNeuronError;
use crate::neurons::traits::{SingleCorticalNeuronCollectionBase, SingleCorticalNeuronCollectionDense};

pub struct NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{
    potentials: Vec<NeuronMembranePotential<PotentialQuant>>,
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    number_neurons_per_voxel: NeuronCount<NumberNeuronsPerVoxel>,
    /// Holds the index quant type so `NeuronVoxelIndexQuant` is not an unused struct parameter.
    _index_quant: PhantomData<NeuronVoxelIndexQuant>,
}

impl <PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    pub fn new(dimensions: NeuronVoxelDimensions<CoordQuant>, density: NeuronCount<NumberNeuronsPerVoxel>) -> Result<Self, FeagiStructuresNeuronError> {
        if density == NeuronCount::ZERO {
            return Err(FeagiStructuresNeuronError::BadParameters {context: "Neuron density cannot be zero!"})
        }

        let number_neurons: NeuronCount<NeuronVoxelIndexQuant> = dimensions.get_number_neurons(density);
        Ok(Self {
            potentials: vec![NeuronMembranePotential::ZERO; number_neurons.to_usize()],
            cortical_dimensions: dimensions,
            number_neurons_per_voxel: density,
            _index_quant: PhantomData,
        })
    }
}

impl <PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> SingleCorticalNeuronCollectionBase<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> for NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_neuron_voxel_density(&self) -> NeuronCount<NumberNeuronsPerVoxel> {
        
        self.number_neurons_per_voxel
    }

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronIndex<NeuronVoxelIndexQuant> {
        NeuronIndex::from_usize(self.cortical_dimensions.get_number_neurons::<NeuronVoxelIndexQuant>(self.number_neurons_per_voxel).to_usize())
    }

    fn neuron_voxel_index_max_limit(&self) -> NeuronVoxelIndex<NeuronVoxelIndexQuant> {
        NeuronVoxelIndex::from_usize(self.cortical_dimensions.get_number_voxels::<NeuronVoxelIndexQuant>().to_usize())
    }

    fn number_of_voxels(&self) -> NeuronVoxelCount<NeuronVoxelIndexQuant> {
        let number: NeuronVoxelCount<NeuronVoxelIndexQuant> = self.cortical_dimensions.get_number_voxels();
        number
    }
}

impl <PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> SingleCorticalNeuronCollectionDense<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> for NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<PotentialQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<PotentialQuant>] {
        self.potentials.as_mut_slice()
    }

    // TODO write into a a dense voxel vector
}

// TODO static

// Note: no need for a sparse trait. The only implementation is with indexing