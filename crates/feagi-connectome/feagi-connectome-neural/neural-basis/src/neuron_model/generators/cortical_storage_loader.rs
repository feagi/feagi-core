use crate::neuron_model::neuron_model::{LayoutDataOf, NeuronModel};
use crate::neuron_model::data::data_containers::CorticalModelFullStorageData;
use crate::neuron_model::generators::cortical_generator_definition::CorticalGeneratorDefinition;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStore;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;


pub trait CorticalScratchGenerator<FIQ, NM>: Sized
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// From the given storage cortical level data, generate the initial scratch data
    fn generate_cortical_scratch(
        properties: &NM::CorticalDataProperties,
        internal: &NM::CorticalDataInternal
    ) -> NM::CorticalDataScratch;


    fn generate_neuron_scratch_at(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        internal: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch;
}

/// A Cortical Generator that reads from storage the data
pub struct CorticalStorageLoader<FIQ, NM, PDS>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    PDS: QuantContiguousStore<Elem = NM::NeuronDataInternal>,
{
    loaded_data: CorticalModelFullStorageData<FIQ, NM, PDS>,
}

impl<FIQ, NM, PDS> CorticalStorageLoader<FIQ, NM, PDS>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    PDS: QuantContiguousStore<Elem = NM::NeuronDataInternal>,
{
    pub fn new(data: CorticalModelFullStorageData<FIQ, NM, PDS>) -> Self {
        Self { loaded_data: data}
    }
}

impl<FIQ, NM, PDS> CorticalGeneratorDefinition<FIQ, NM> for CorticalStorageLoader<FIQ, NM, PDS>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    PDS: QuantContiguousStore<Elem = NM::NeuronDataInternal>,
{
    fn get_neuron_layout(&self) -> &LayoutDataOf<NM, FIQ> {
        &self.loaded_data.layout
    }

    fn get_cortical_level_data<LSG: CorticalScratchGenerator<FIQ, NM>>(&self)
                                                                            -> (NM::CorticalDataProperties, NM::CorticalDataInternal, NM::CorticalDataScratch) {
        
        let scratch = LSG::generate_cortical_scratch(
            &self.loaded_data.cortical_properties,
            &self.loaded_data.cortical_internal
        );
        
        (
            self.loaded_data.cortical_properties.clone(),
            self.loaded_data.cortical_internal.clone(),
            scratch
        )
    }

    fn generate_neuron_data_at<LSG: CorticalScratchGenerator<FIQ, NM>>(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        let internal = &self.loaded_data.neuron_internal[*index];
        let scratch = LSG::generate_neuron_scratch_at(index, internal);
        (
            internal.clone(),
            scratch
        )

    }
}

unsafe impl<FIQ, NM, PDS> Send for CorticalStorageLoader<FIQ, NM, PDS>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    PDS: QuantContiguousStore<Elem = NM::NeuronDataInternal>,
{
}
