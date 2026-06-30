use core::marker::PhantomData;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_data::values::quantizable::QuantizedDecimalTrait;
use feagi_npu_common::wrapped_values::NeuronMembranePotential;
use crate::synapse_models::basic_synapse::data::BasicSynapseModelAxonBundleData;
use crate::synapse_models::basic_synapse::quantization::BasicSynapseModelQuantization;
use crate::synapse_models::synapse_model_traits::synapse_model_processor::{SynapseModelProcessorAxonBundleOnly, SynapseModelProcessorBase};

pub struct BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    // No actual members
    _p: PhantomData<(FGQ, SMQ, CPQIn, CPQOut)>,
}

impl<FGQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorAxonBundleOnly<FGQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut> for BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut> where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization
{
    fn process_neuron_potential_through_bundle(outgoing_potential: &NeuronMembranePotential<CPQIn::NeuronPotentialQuant>, axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>, potential_write_target: &mut NeuronMembranePotential<CPQOut::NeuronPotentialQuant>) {
        todo!()
    }
}

impl<FGQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorBase<FGQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut> for BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut>where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{}

impl<FGQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorAxonBundleOnly<FGQ, SMQ, BasicSynapseModelAxonBundleData<SMQ>, CPQIn, CPQOut> for BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NeuronMembranePotential<CPQIn::NeuronPotentialQuant>,
        axon_bundle_data: &BasicSynapseModelAxonBundleData<SMQ>,
        potential_write_target: &mut NeuronMembranePotential<CPQOut::NeuronPotentialQuant>) 
    {
        let multiplier = *axon_bundle_data.multiplier.as_ref();
        let potential = *outgoing_potential.as_ref();

        // TODO going through f32?
        *potential_write_target += NeuronMembranePotential::from(multiplier.to_f32() * potential.to_f32());
    }
}