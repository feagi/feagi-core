use core::marker::PhantomData;
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedDecimalTrait;
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::synapse_model_traits::synapse_model_processor::{SynapseModelProcessorAxonBundleOnly, SynapseModelProcessorAxonBundleOnlyCPU, SynapseModelProcessorBase};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::synapse_models::basic_synapse::data::BasicSynapseModelAxonBundleDataCPU;
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::synapse_models::basic_synapse::quantization::BasicSynapseModelQuantization;
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedNeuronMembranePotential;

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

impl<FGQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorAxonBundleOnly<FGQ, SMQ, BasicSynapseModelAxonBundleDataCPU<SMQ>, CPQIn, CPQOut> for BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut> where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization
{}

impl<FGQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorBase<FGQ, SMQ, BasicSynapseModelAxonBundleDataCPU<SMQ>, CPQIn, CPQOut> for BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut>where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{}

impl<FGQ, SMQ, CPQIn, CPQOut> SynapseModelProcessorAxonBundleOnlyCPU<FGQ, SMQ, BasicSynapseModelAxonBundleDataCPU<SMQ>, CPQIn, CPQOut> for BasicSynapseModelProcessorCPU<FGQ, SMQ, CPQIn, CPQOut>
where
    FGQ: FeagiGlobalQuantization,
    SMQ: BasicSynapseModelQuantization,
    CPQIn: CorticalPotentialQuantization,
    CPQOut: CorticalPotentialQuantization,
{
    fn process_neuron_potential_through_bundle(
        outgoing_potential: &NPUWrappedNeuronMembranePotential<CPQIn::NeuronPotentialQuant>,
        axon_bundle_data: &BasicSynapseModelAxonBundleDataCPU<SMQ>,
        potential_write_target: &mut NPUWrappedNeuronMembranePotential<CPQOut::NeuronPotentialQuant>) 
    {
        let multiplier = axon_bundle_data.multiplier.unwrap();
        let potential = outgoing_potential.unwrap();
        *potential_write_target += NPUWrappedNeuronMembranePotential::wrap(
            CPQOut::NeuronPotentialQuant::from_f32(multiplier.to_f32() * potential.to_f32())
        );
    }
}