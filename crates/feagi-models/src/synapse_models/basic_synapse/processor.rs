use crate::synapse_models::basic_synapse::data::BasicSynapseModelAxonBundleData;
use crate::synapse_models::basic_synapse::quantization::BasicSynapseModelQuantization;
use crate::synapse_models::synapse_model_traits::synapse_model_processor::{SynapseModelProcessorAxonBundleOnly, SynapseModelProcessorBase};
use core::marker::PhantomData;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialCPUQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedDecimalTrait;

pub struct BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialCPUQuantization,
    CPQOut: CorticalPotentialCPUQuantization,
{
    // No actual members
    _p: PhantomData<(FIQ, SMQ, CPQIn, CPQOut)>,
}

impl<FIQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorBase<FIQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut>
    for BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialCPUQuantization,
    CPQOut: CorticalPotentialCPUQuantization,
{
    type CustomContext = ();
}

impl<FIQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorAxonBundleOnly<FIQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut>
    for BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialCPUQuantization,
    CPQOut: CorticalPotentialCPUQuantization,
{
    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NeuronMembranePotential<CPQIn::MembranePotentialQuant>,
        axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>,
        _custom_context: &Self::CustomContext,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::MembranePotentialQuant>,
    ) {
        // TODO going through f32?
        *potential_write_target += NeuronMembranePotential::from_f32(outgoing_potential.to_f32() * axon_bundle_data.multiplier.to_f32());
    }

    fn get_psp_uniformity_weight(axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>) -> CPQIn::MembranePotentialQuant {
        // TODO this is dumb :3
        CPQIn::MembranePotentialQuant::from_f32(axon_bundle_data.multiplier.to_f32())
    }
}
