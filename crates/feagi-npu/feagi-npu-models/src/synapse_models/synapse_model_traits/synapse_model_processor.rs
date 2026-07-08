use crate::synapse_models::synapse_model_traits::synapse_model_data::{
    SynapseModelAxonBundleData, SynapseModelSynapseData,
};
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::extendable_quantizations::SynapseModelQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::wrapped_values::NeuronMembranePotential;

/// Root base trait for defining synapse firing and alteration of
/// transmitting neuron potentials between neurons. Does NOT store actual data,
pub trait SynapseModelProcessorBase<FGQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    // Methods for
    // Synapse model custom context
    // Getting mut output write target (fclc / fcl)
    // Executing synapse
}

pub trait SynapseModelProcessorAxonBundleOnly<FGQ, SMQ, SMABD, CPQIn, CPQOut>:
    SynapseModelProcessorBase<FGQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    // TODO custom Context

    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NeuronMembranePotential<CPQIn::NeuronPotentialQuant>,
        axon_bundle_data: &SMABD,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::NeuronPotentialQuant>,
    );
}

pub trait SynapseModelProcessorWithPerSynapse<FGQ, SMQ, SMABD, SMSD, CPQIn, CPQOut>:
    SynapseModelProcessorBase<FGQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    fn process_neuron_potential_through_synapse(
        outgoing_potential: &NeuronMembranePotential<CPQIn::NeuronPotentialQuant>,
        axon_bundle_data: &SMABD,
        synapse_data: &mut SMSD,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::NeuronPotentialQuant>,
    );
}
