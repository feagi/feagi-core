use crate::synapse_models::basic_synapse::data::BasicSynapseModelAxonBundleData;
use crate::synapse_models::basic_synapse::quantization::BasicSynapseModelQuantization;
use crate::synapse_models::synapse_model_traits::synapse_model_processor::{
    SynapseModelProcessorAxonBundleOnly, SynapseModelProcessorBase,
};
use core::marker::PhantomData;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    // No actual members
    _p: PhantomData<(FIQ, SMQ, CPQIn, CPQOut)>,
}

impl<FIQ, SMQ, CPQIn, CPQOut>
    SynapseModelProcessorBase<FIQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut>
    for BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
}

impl<FIQ, SMQ, CPQIn, CPQOut>
    SynapseModelProcessorAxonBundleOnly<
        FIQ,
        SMQ,
        BasicSynapseModelAxonBundleData<SMQ>,
        CPQIn,
        CPQOut,
    > for BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NeuronMembranePotential<CPQIn::MembranePotentialQuant>,
        axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::MembranePotentialQuant>,
    ) {
        // TODO going through f32?
        *potential_write_target += NeuronMembranePotential::from_f32(
            outgoing_potential.to_f32() * axon_bundle_data.multiplier.to_f32(),
        );
    }
}
