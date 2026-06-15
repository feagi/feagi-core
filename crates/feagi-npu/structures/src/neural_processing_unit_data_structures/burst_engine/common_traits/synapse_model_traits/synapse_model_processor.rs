use feagi_structures::feagi_data::quantization_levels::extendable_quantizations::{NeuronModelQuantization, SynapseModelQuantization};
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::synapse_model_traits::synapse_model_axon_bundle_data::SynapseModelAxonBundleData;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::synapse_model_traits::synapse_model_synapse_data::SynapseModelSynapseData;
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedNeuronMembranePotential;

/// Root base trait for defining synapse firing and alteration of
/// transmitting neuron potentials between neurons. Does NOT store actual data,
pub trait SynapseModelProcessor<FGQ, SMQ, SMABD, SMSD, NMQIn, NMQOut>:
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    NMQIn: NeuronModelQuantization,
    NMQOut: NeuronModelQuantization
{
    // Methods for
    // Synapse model custom context
    // Getting mut output write target (fclc / fcl)
    // Executing synapse
}


pub trait SynapseModelProcessorCPU<FGQ, SMQ, SMABD, SMSD, NMQIn, NMQOut>:
SynapseModelProcessor<FGQ, SMQ, SMABD, SMSD, NMQIn, NMQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: SynapseModelQuantization,
    SMABD: SynapseModelAxonBundleData<SMQ>,
    SMSD: SynapseModelSynapseData<SMQ>,
    NMQIn: NeuronModelQuantization,
    NMQOut: NeuronModelQuantization
{
    // TODO custom Context

    // TODO seperate per synapse per axon

    fn process_neuron_potential_through_bundle_simple(
        outgoing_potential: &NPUWrappedNeuronMembranePotential<NMQIn::CorticalPotentialQuant>,
        axon_bundle_data: &SMABD,
        potential_write_target: &mut NPUWrappedNeuronMembranePotential<NMQOut::CorticalPotentialQuant>
    );

    fn process_neuron_potential_through_synapse_simple(
        outgoing_potential: &NPUWrappedNeuronMembranePotential<NMQIn::CorticalPotentialQuant>,
        axon_bundle_data: &SMABD,
        synapse_data: &mut SMSD,
        potential_write_target: &mut NPUWrappedNeuronMembranePotential<NMQOut::CorticalPotentialQuant>
    );





}