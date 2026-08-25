use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use crate::cortical_area::components::neuron_firing_dynamics::components::cortical_data_properties::CorticalDataProperties;
use crate::cortical_area::components::neuron_firing_dynamics::components::cortical_data_work::CorticalDataWork;
use crate::cortical_area::components::neuron_firing_dynamics::components::neuron_data_properties::NeuronDataProperties;
use crate::cortical_area::components::neuron_firing_dynamics::components::neuron_data_work::NeuronDataWork;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayoutModelTrait;
use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::wrapped_indexes::BurstIndex;
// TODO Neuron Membrane Type should be configurable

/// Defines how neuron firing should be handled. Generally has model specific implementations
pub trait NeuronFiringDynamicsModel<NPUIQ, BEIQ, CAMQ, Layout>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
    Layout: NeuronLayoutModelTrait
{
    /// The cortical level data properties that should be exposed to genome developers
    type CorticalDataProperties: CorticalDataProperties<NPUIQ, BEIQ, CAMQ>;

    /// The cortical level data properties that is for internal processing, will not be
    /// exposed to genome developers or saved
    type CorticalDataWork: CorticalDataWork<NPUIQ, BEIQ, CAMQ>;

    /// The per neuron level data properties that should be exposed to genome developers
    type NeuronDataProperties: NeuronDataProperties<NPUIQ, BEIQ, CAMQ>;

    /// The per neuron level data properties that is for internal processing, will not be
    /// exposed to genome developers or saved
    type NeuronDataWork: NeuronDataWork<NPUIQ, BEIQ, CAMQ>;


    fn process_neuron_dynamics(
        incoming_potential: &CorticalNeuronPotential<CAMQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>,
        neuron_linear_index: &CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>,
        layout_context: &Layout,
    ) -> ()
    {
        
    }

}