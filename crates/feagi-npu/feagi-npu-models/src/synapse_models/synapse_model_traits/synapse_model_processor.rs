use crate::synapse_models::synapse_model_traits::synapse_model_data::{
    SynapseModelAxonBundleData, SynapseModelQuantization, SynapseModelSynapseData,
};
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialCPUQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root base trait for defining synapse firing and alteration of
/// transmitting neuron potentials between neurons. Does NOT store actual data,
pub trait SynapseModelProcessorBase<FIQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    CPQIn: CorticalPotentialCPUQuantization,
    CPQOut: CorticalPotentialCPUQuantization,
{ 
    // TODO custom context may need to be moved somewhere else, cause wouldnt this cause slowdown for memory stuff?
    /// Some synapses require custom contexts / additional information. This advanced parameter
    /// effectively allows passing any data by reference for use in synapse calculations
    type CustomContext;
    
}

pub trait SynapseModelProcessorAxonBundleOnly<FIQ, SMQ, SMABD, CPQIn, CPQOut>:
    SynapseModelProcessorBase<FIQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    CPQIn: CorticalPotentialCPUQuantization,
    CPQOut: CorticalPotentialCPUQuantization,
{
    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NeuronMembranePotential<CPQIn::MembranePotentialQuant>,
        axon_bundle_data: &SMABD,
        custom_context: &Self::CustomContext,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::MembranePotentialQuant>,
    );

    fn get_psp_uniformity_weight(axon_bundle_data: &SMABD) -> CPQIn::MembranePotentialQuant;
}

pub trait SynapseModelProcessorWithPerSynapse<FIQ, SMQ, SMABD, SMSD, CPQIn, CPQOut>:
    SynapseModelProcessorBase<FIQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    CPQIn: CorticalPotentialCPUQuantization,
    CPQOut: CorticalPotentialCPUQuantization,
{
    fn process_neuron_potential_through_synapse(
        outgoing_potential: &NeuronMembranePotential<CPQIn::MembranePotentialQuant>,
        axon_bundle_data: &SMABD,
        synapse_data: &mut SMSD,
        custom_context: &Self::CustomContext,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::MembranePotentialQuant>,
    );

    fn get_psp_uniformity_weight(axon_bundle_data: &SMABD, synapse_data: &SMSD) -> CPQIn::MembranePotentialQuant;
}
