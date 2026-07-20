use crate::synapse_models::models::uniform_weight::data::BasicSynapseModelAxonBundleData;
use crate::synapse_models::models::uniform_weight::quantization::UniformSynapseModelQuantization;
use crate::synapse_models::shared::processor::{SynapseModelProcessorAxonBundleOnly, SynapseModelProcessorBase};
use core::marker::PhantomData;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedDecimalTrait;

pub struct BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: UniformSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    // No actual members
    _p: PhantomData<(FIQ, SMQ, CPQIn, CPQOut)>,
}

impl<FIQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorBase<FIQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut>
    for BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: UniformSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    type CustomContext = ();
}

impl<FIQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorAxonBundleOnly<FIQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut>
    for BasicSynapseModelProcessor<FIQ, SMQ, CPQIn, CPQOut>
where
    FIQ: FeagiIndexQuantization,
    SMQ: UniformSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NeuronMembranePotential<CPQIn::MembranePotentialQuant>,
        axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>,
        _custom_context: &Self::CustomContext,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::MembranePotentialQuant>,
    ) {
        let incoming_potential = outgoing_potential.deref().to_quantization::<SMQ::MultiplierQuant>() * axon_bundle_data.multiplier.deref();
        *potential_write_target += NeuronMembranePotential::from_quantization(incoming_potential);
    }

    fn get_psp_uniformity_weight(axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>) -> CPQIn::MembranePotentialQuant {
        // TODO this is dumb
        CPQIn::MembranePotentialQuant::from_quantization(axon_bundle_data.multiplier.deref())
    }
}
