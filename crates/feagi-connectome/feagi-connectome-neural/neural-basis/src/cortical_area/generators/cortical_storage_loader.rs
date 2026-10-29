use crate::cortical_area::cortical_dynamics_model::CorticalDynamicsModel;
use crate::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use crate::cortical_area::data::data_containers::CorticalModelFullStorageData;
use crate::cortical_area::data::neuron_layout::NeuronLayout;
use crate::cortical_area::generators::cortical_generator_definition::CorticalGeneratorDefinition;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::par_data::par_data::ParDataStore;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;

pub trait CorticalStorageLoaderScratchGenerator<FIQ, NL, CDM, CDQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
{
    /// From the given storage cortical level data, generate the initial scratch data
    fn generate_cortical_scratch(
        &self,
        properties: &CDQ::CorticalDataProperties,
        internal: &CDQ::CorticalDataInternal
    ) -> CDQ::CorticalDataScratch;


    fn generate_neuron_scratch_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        internal: &CDQ::NeuronDataInternal,
    ) -> CDQ::NeuronDataScratch;
}

/// A Cortical Generator that reads from storage the data
pub struct CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS, LSG>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: ParDataStore<Elem = CDQ::NeuronDataInternal>,
    LSG: CorticalStorageLoaderScratchGenerator<FIQ, NL, CDM, CDQ>
{
    loaded_data: CorticalModelFullStorageData<FIQ, NL, CDM, CDQ, PDS>,
    _p: core::marker::PhantomData<LSG>
}

impl<FIQ, NL, CDM, CDQ, PDS, LSG> CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS, LSG>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: ParDataStore<Elem = CDQ::NeuronDataInternal>,
    LSG: CorticalStorageLoaderScratchGenerator<FIQ, NL, CDM, CDQ>
{
    pub fn new(data: CorticalModelFullStorageData<FIQ, NL, CDM, CDQ, PDS>) -> Self {
        Self { loaded_data: data, _p: core::marker::PhantomData }
    }
}

impl<FIQ, NL, CDM, CDQ, PDS, LSG> CorticalGeneratorDefinition<FIQ, NL, CDQ> for CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS, LSG>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: ParDataStore<Elem = CDQ::NeuronDataInternal>,
    LSG: CorticalStorageLoaderScratchGenerator<FIQ, NL, CDM, CDQ>
{
    type CorticalModel = CDM;

    fn get_neuron_layout(&self) -> &NL {
        &self.loaded_data.layout
    }

    fn get_cortical_level_data(self) -> (CDQ::CorticalDataProperties, CDQ::CorticalDataInternal, CDQ::CorticalDataScratch) {
        (self.loaded_data.cortical_properties)
    }

    fn generate_neuron_data_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (CDQ::NeuronDataInternal, CDQ::NeuronDataScratch) {
        let internal = self.loaded_data.neuron_internal.store().get(index);

    }
}

unsafe impl<FIQ, NL, CDM, CDQ, PDS, LSG> Send for CorticalStorageLoader<FIQ, NL, CDM, CDQ, PDS, LSG>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    PDS: ParDataStore<Elem = CDQ::NeuronDataInternal>,
    LSG: CorticalStorageLoaderScratchGenerator<FIQ, NL, CDM, CDQ>
{
}
