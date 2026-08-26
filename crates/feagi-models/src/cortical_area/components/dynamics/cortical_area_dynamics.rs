use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use crate::cortical_area::components::dynamics::components::data::{CorticalDataProperties, CorticalDataInternal, CorticalDataShared, NeuronDataProperties, NeuronDataInternal};
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayoutModelTrait;
use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::wrapped_indexes::BurstIndex;

pub trait CorticalAreaDynamics<NPUIQ, BEIQ, CAMQ, NL>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
    NL: NeuronLayoutModelTrait
{
    /// The cortical level data that should be exposed to genome developers
    type CorticalDataProperties: CorticalDataProperties<NPUIQ, BEIQ, CAMQ>;

    /// The cortical level data that is for internal processing, will not be
    /// exposed to genome developers
    type CorticalDataInternal: CorticalDataInternal<NPUIQ, BEIQ, CAMQ>;

    /// The cortical level data that is shared and accessible by mappings (not other areas)
    type CorticalDataShared: CorticalDataShared<NPUIQ, BEIQ, CAMQ>;

    /// The per neuron level data that should be exposed to genome developers
    type NeuronDataProperties: NeuronDataProperties<NPUIQ, BEIQ, CAMQ>;

    /// The per neuron level data that is for internal processing, will not be
    /// exposed to genome developers
    type NeuronDataInternal: NeuronDataInternal<NPUIQ, BEIQ, CAMQ>;


    // TODO any sort of input? Mapping handling?
    
    /// called per area
    fn process_cortical_dynamics(
        burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>, 
        cortical_properties: &mut Self::CorticalDataProperties,
        cortical_internal: &mut Self::CorticalDataInternal,
        cortical_shared: &mut Self::CorticalDataShared,
        layout_context: &NL,
    ) -> () // TODO return type?
    {
        // By Default, nothing!
    }
    
    /// called per neuron
    fn process_neuron_dynamics(
        incoming_potential: &CorticalNeuronPotential<CAMQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>,
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &Self::CorticalDataInternal,
        cortical_shared: &Self::CorticalDataShared,
        neuron_properties: &mut Self::NeuronDataProperties,
        neuron_internal: &mut Self::NeuronDataInternal,
        neuron_linear_index: &CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>,
        layout_context: &NL,
        
    ) -> bool;

}