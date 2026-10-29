use crate::cortical_area::cortical_dynamics_model::CorticalDynamicsModel;
use crate::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use crate::cortical_area::data::data_containers::CorticalModelFullStorageData;
use crate::cortical_area::data::neuron_layout::NeuronLayout;
use crate::cortical_area::generators::cortical_generator_definition::CorticalGeneratorDefinition;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStore;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;


pub trait CorticalScratchGenerator<FIQ, NL, CDQ>: Sized
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
{
    /// From the given storage cortical level data, generate the initial scratch data
    fn generate_cortical_scratch(
        properties: &CDQ::CorticalDataProperties,
        internal: &CDQ::CorticalDataInternal
    ) -> CDQ::CorticalDataScratch;


    fn generate_neuron_scratch_at(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        internal: &CDQ::NeuronDataInternal,
    ) -> CDQ::NeuronDataScratch;
}

/// A Cortical Generator that reads from storage the data
pub struct CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: QuantContiguousStore<Elem = CDQ::NeuronDataInternal>,
{
    loaded_data: CorticalModelFullStorageData<FIQ, NL, CDM, CDQ, PDS>,
}

impl<FIQ, NL, CDM, CDQ, PDS> CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: QuantContiguousStore<Elem = CDQ::NeuronDataInternal>,
{
    pub fn new(data: CorticalModelFullStorageData<FIQ, NL, CDM, CDQ, PDS>) -> Self {
        Self { loaded_data: data}
    }
}

impl<FIQ, NL, CDM, CDQ, PDS> CorticalGeneratorDefinition<FIQ, NL, CDQ> for CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: QuantContiguousStore<Elem = CDQ::NeuronDataInternal>,
{
    type CorticalModel = CDM;

    fn get_neuron_layout(&self) -> &NL {
        &self.loaded_data.layout
    }

    fn get_cortical_level_data<LSG: CorticalScratchGenerator<FIQ, NL, CDQ>>(&self)
                                                                            -> (CDQ::CorticalDataProperties, CDQ::CorticalDataInternal, CDQ::CorticalDataScratch) {
        
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

    fn generate_neuron_data_at<LSG: CorticalScratchGenerator<FIQ, NL, CDQ>>(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (CDQ::NeuronDataInternal, CDQ::NeuronDataScratch) {
        let internal = &self.loaded_data.neuron_internal[*index];
        let scratch = LSG::generate_neuron_scratch_at(index, internal);
        (
            internal.clone(),
            scratch
        )

    }
}

unsafe impl<FIQ, NL, CDM, CDQ, PDS> Send for CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: QuantContiguousStore<Elem = CDQ::NeuronDataInternal>,
{
}
