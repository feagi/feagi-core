use core::marker::PhantomData;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::useful_structs::{IndexedDataTracker};
use crate::neuron::FeagiNPUNeuronError;
// TODO Migrate away from the general IndexedDataTracker approach and instead use something
// capable of reusing memory!

pub struct DimensionalCorticalNeuronsCollection<CorticalIndexQuant, CorticalCountQuant, NeuronIndexQuant, NeuronCountQuant, NeuronDataContainer>
where
    CorticalIndexQuant: QuantizableUIntType,
    NeuronIndexQuant: QuantizableUIntType,

{
    cortical_mapped_neuron_data: IndexedDataTracker<NeuronDataContainer, CorticalAreaIndex<CorticalIndexQuant>>,
    cortical_count_quant: PhantomData<CorticalCountQuant>,
    neuron_index_quant: PhantomData<NeuronIndexQuant>,
    neuron_count_quant: PhantomData<NeuronCountQuant>,
}

impl<CorticalIndexQuant, CorticalCountQuant, NeuronIndexQuant, NeuronCountQuant, NeuronDataContainer>
DimensionalCorticalNeuronsCollection<CorticalIndexQuant, CorticalCountQuant, NeuronIndexQuant, NeuronCountQuant, NeuronDataContainer>
where
    CorticalIndexQuant: QuantizableUIntType,
    NeuronIndexQuant: QuantizableUIntType,
{
    pub fn new() -> Self {
        Self {
            cortical_mapped_neuron_data: IndexedDataTracker::new(),
            cortical_count_quant: Default::default(),
            neuron_index_quant: Default::default(),
            neuron_count_quant: Default::default(),
        }
    }

    pub fn try_get_neuron_data(&self, cortical_index: CorticalAreaIndex<CorticalIndexQuant>) -> Result<&NeuronDataContainer, FeagiNPUNeuronError> {
        self.cortical_mapped_neuron_data.get(cortical_index).ok_or_else(
            || FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "Unable to get nonexistent cortical index!",
                given_cortical_index: cortical_index.to_usize() as u32
            }
        )
    }

    pub fn get_neuron_data_mut(&mut self, cortical_index: CorticalAreaIndex<CorticalIndexQuant>) -> Result<&mut NeuronDataContainer, FeagiNPUNeuronError> {
        self.cortical_mapped_neuron_data.get_mut(cortical_index).ok_or_else(
            || FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "Unable to get nonexistent cortical index!",
                given_cortical_index: cortical_index.to_usize() as u32
            }
        )
    }

    pub fn insert_new_cortical_area(&mut self, new_neuron_data: NeuronDataContainer) -> CorticalAreaIndex<CorticalIndexQuant> {
        self.cortical_mapped_neuron_data.insert_data_and_get_unique_index(new_neuron_data) // TODO inefficient, see top!
    }

    pub fn remove_cortical_area(&mut self, cortical_index: CorticalAreaIndex<CorticalIndexQuant>) -> Result<(), FeagiNPUNeuronError> {
        let result = self.cortical_mapped_neuron_data.return_index_and_invalidate_data(cortical_index);
        result.map_err(|e| FeagiNPUNeuronError::InvalidCorticalIndex {
            given_cortical_index: cortical_index.to_usize() as u32,
            context: "Unable to remove nonexistent cortical index!"
        })
    }
}