use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_structures::feagi_data::quantization_levels::extendable_quantizations::{SynapseModelQuantization};
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::synapse_model_traits::synapse_model_axon_bundle_data::SynapseModelAxonBundleData;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::synapse_model_traits::synapse_model_synapse_data::SynapseModelSynapseData;
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedNeuronMembranePotential;

/// Root base trait for defining synapse firing and alteration of
/// transmitting neuron potentials between neurons. Does NOT store actual data,
pub trait SynapseModelProcessorBase<FGQ, SMQ, SMABD, CPQIn, CPQOut>:
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization
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
    CPQOut: CorticalPotentialQuantization
{
    // Method for Executing synapse (using only axon bundle level data)
}

pub trait SynapseModelProcessorWithPerSynapse<FGQ, SMQ, SMABD, SMSD, CPQIn, CPQOut>:
SynapseModelProcessorBase<FGQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization
{
    // Method for Executing synapse (also with per synapse level mutable data)
}


// region CPU implementation

pub trait SynapseModelProcessorAxonBundleOnlyCPU<FGQ, SMQ, SMABD, CPQIn, CPQOut>:
SynapseModelProcessorAxonBundleOnly<FGQ, SMQ, SMABD, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization
{
    // TODO custom Context

    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NPUWrappedNeuronMembranePotential<CPQIn::NeuronPotentialQuant>,
        axon_bundle_data: &SMABD,
        potential_write_target: &mut NPUWrappedNeuronMembranePotential<CPQOut::NeuronPotentialQuant>
    );
}

pub trait SynapseModelProcessorWithPerSynapseCPU<FGQ, SMQ, SMABD, SMSD, CPQIn, CPQOut>:
SynapseModelProcessorWithPerSynapse<FGQ, SMQ, SMABD, SMSD, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization
{
    // TODO custom Context

    fn process_neuron_potential_through_synapse(
        outgoing_potential: &NPUWrappedNeuronMembranePotential<CPQIn::NeuronPotentialQuant>,
        axon_bundle_data: &SMABD,
        synapse_data: &mut SMSD,
        potential_write_target: &mut NPUWrappedNeuronMembranePotential<CPQOut::NeuronPotentialQuant>
    );
}

//endregion

