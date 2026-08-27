use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use crate::cortical_area::parameters::dynamics::components::data::{CorticalDataProperties, CorticalDataInternal, CorticalDataShared, NeuronDataProperties, NeuronDataInternal};
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout};
use crate::cortical_area::parameters::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::wrapped_indexes::BurstIndex;

// TODO maybe we should allow the BEIQ type in here so people can better use the layout information (tie it to a unique data struct that is not saved, that must instead be generated at instantiation / layout edit)

pub trait CorticalAreaDynamics<NPUIQ, BEIQ, NL, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    NL: NeuronLayout<BEIQ>,
    CAMQ: CorticalAreaQuantization,
{
    /// The cortical level data that should be exposed to genome developers
    type CorticalDataProperties: CorticalDataProperties<CAMQ>;

    /// The cortical level data that is for internal processing, will not be
    /// exposed to genome developers
    type CorticalDataInternal: CorticalDataInternal<CAMQ>;

    /// The cortical level data that is shared and accessible by mappings (not other areas)
    type CorticalDataShared: CorticalDataShared<CAMQ>;

    /// The per neuron level data that should be exposed to genome developers
    type NeuronDataProperties: NeuronDataProperties<CAMQ>;

    /// The per neuron level data that is for internal processing, will not be
    /// exposed to genome developers
    type NeuronDataInternal: NeuronDataInternal<CAMQ>;

    // TODO we should probably break the processing functions also into subtraits
    
    // TODO any sort of input? Mapping handling?
    
    /// called per area. Called before neuron call
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