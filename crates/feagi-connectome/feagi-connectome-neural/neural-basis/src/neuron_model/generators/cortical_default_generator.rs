use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::neuron_model::generators::cortical_generator_definition::CorticalGeneratorDefinition;
use crate::neuron_model::generators::cortical_storage_loader::CorticalScratchGenerator;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronLayoutType, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// TODO some pass in trait to run the neuron scratch data generator per neuron (or maybe leave it as some combo overlay generator?





/// A Cortical Area Generator that simply uses default values, for the cortical area level properties
/// and all neuron properties. Note the Cortical Function Context will init the neuron data after the
/// default value is loaded
pub struct CorticalDefaultGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    pub neuron_layout: NeuronLayoutOf<NM, FIQ>,

    pub default_cortical_data_properties: NM::CorticalDataProperties,
    pub default_cortical_data_internal: NM::CorticalDataInternal,
    pub default_cortical_data_scratch: NM::CorticalDataScratch,
    pub default_cortical_data_function_context: NM::CorticalFunctionContext,

    pub default_neuron_data_internal: NM::NeuronDataInternal,
    pub default_neuron_data_scratch: NM::NeuronDataScratch,

    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ, NM> CorticalDefaultGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Create new from the fields directly
    pub fn new_individual(
        neuron_layout: NeuronLayoutOf<NM, FIQ>,
        default_cortical_data_properties: NM::CorticalDataProperties,
        default_cortical_data_internal: NM::CorticalDataInternal,
        default_cortical_data_scratch: NM::CorticalDataScratch,
        default_cortical_data_function_context: NM::CorticalFunctionContext,
        default_neuron_data_internal: NM::NeuronDataInternal,
        default_neuron_data_scratch: NM::NeuronDataScratch,
    ) -> Self {
        Self {
            neuron_layout,
            default_cortical_data_properties,
            default_cortical_data_internal,
            default_cortical_data_scratch,
            default_cortical_data_function_context,
            default_neuron_data_internal,
            default_neuron_data_scratch,
            _p: core::marker::PhantomData,
        }
    }
}

impl<FIQ, NM> CorticalGeneratorDefinition<FIQ, NM> for CorticalDefaultGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.neuron_layout
    }

    fn get_cortical_level_data<LSG: CorticalScratchGenerator<FIQ, NM>>(&self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal, NM::CorticalDataScratch) {
        (self.default_cortical_data_properties.clone(), self.default_cortical_data_internal.clone(), self.default_cortical_data_scratch.clone())
    }

    fn generate_neuron_data_at<LSG: CorticalScratchGenerator<FIQ, NM>>(&self, _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        (self.default_neuron_data_internal.clone(), self.default_neuron_data_scratch.clone())
        
        // TODO function context
    }
}

/// A trait meant to be implemented manually on a zero sized struct for a
pub trait CorticalCommonDefaults<FIQ, NM>: Sized
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    const DEFAULT_CORTICAL_INTERNAL: NM::CorticalDataInternal;
    const DEFAULT_CORTICAL_SCRATCH: NM::CorticalDataScratch;
    const DEFAULT_CORTICAL_FUNCTION_CONTEXT: NM::CorticalFunctionContext;

    const DEFAULT_NEURON_INTERNAL: NM::NeuronDataInternal;
    const DEFAULT_NEURON_SCRATCH: NM::NeuronDataScratch;


}