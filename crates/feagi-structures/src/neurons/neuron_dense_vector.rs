use core::marker::PhantomData;
use crate::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxels::descriptors::NeuronVoxelDimensions;
use crate::neurons::descriptors::{NeuronPotential, NumberNeuronsPerVoxel};
use crate::neurons::FeagiStructuresNeuronError;
use crate::neurons::traits::{SingleCorticalNeuronCollectionBase, SingleCorticalNeuronCollectionDense};

pub struct NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{
    potentials: Vec<NeuronPotential<PotentialQuant>>,
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    cortical_density: NumberNeuronsPerVoxel,
    /// Holds the index quant type so `NeuronVoxelIndexQuant` is not an unused struct parameter.
    _index_quant: PhantomData<NeuronVoxelIndexQuant>,
}

impl <PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    pub fn new(dimensions: NeuronVoxelDimensions<CoordQuant>, density: NumberNeuronsPerVoxel) -> Result<Self, FeagiStructuresNeuronError> {
        if density == 0 {
            return Err(FeagiStructuresNeuronError::BadParameters {context: "Neuron density cannot be zero!"})
        }

        let number_neurons: usize = dimensions.get_number_neurons(density);
        Ok(Self {
            potentials: vec![NeuronPotential::ZERO; number_neurons],
            cortical_dimensions: dimensions,
            cortical_density: density,
            _index_quant: PhantomData,
        })
    }
}

impl <PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> SingleCorticalNeuronCollectionBase<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> for NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_neuron_voxel_density(&self) -> NumberNeuronsPerVoxel {
        self.cortical_density
    }

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(
            self.cortical_dimensions.get_number_neurons(self.cortical_density),
        )
    }
}

impl <PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> SingleCorticalNeuronCollectionDense<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> for NeuronDenseVector<PotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_all_neuron_potentials(&self) -> &[NeuronPotential<PotentialQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronPotential<PotentialQuant>] {
        self.potentials.as_mut_slice()
    }

    // TODO write into a a dense voxel vector
}

// TODO static

// Note: no need for a sparse trait. The only implementation is with indexing