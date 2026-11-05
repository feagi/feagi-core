use feagi_basis::BurstIndex;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::prelude::QuantizedDecimalUnwrappedTrait;
use neural_basis::connectivity_rule::AxonBundleSynapseLocalIndex;
use neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use neural_basis::synapse_model::{SynapseBurstEngineDynamics, SynapseModel};

pub trait SynapseCPUParallelDynamics<FIQ, SM>: SynapseBurstEngineDynamics<FIQ, SM>
where
    FIQ: FeagiIndexQuantization,
    SM: SynapseModel
{

    /// Called per axon bundle before the synapse call
    fn process_model_axon_bundle_dynamics<
        IncomingQuant: QuantizedDecimalUnwrappedTrait,
        OutgoingQuant: QuantizedDecimalUnwrappedTrait
    > (
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        axon_bundle_data: (
            &SM::AxonBundleDataProperties,
            &mut SM::AxonBundleDataInternal,
            &mut SM::AxonBundleDataScratch,
        )
    ); // no return type

    fn process_model_synapse_dynamics<
        IncomingQuant: QuantizedDecimalUnwrappedTrait,
        OutgoingQuant: QuantizedDecimalUnwrappedTrait
    > (
        incoming_potential: &CorticalAreaNeuronPotential<IncomingQuant>,
        outgoing_potential: &mut CorticalAreaNeuronPotential<IncomingQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        synapse_linear_index: &AxonBundleSynapseLocalIndex<FIQ::SynapseIndexCountQuant>,
        axon_bundle_data: (
            &SM::AxonBundleDataProperties,
            &SM::AxonBundleDataInternal,
            &SM::AxonBundleDataScratch,
        ),
        synapse_data: (
            &mut SM::SynapseDataInternal,
            &mut SM::SynapseDataScratch,
        )
    ); // No return type




}