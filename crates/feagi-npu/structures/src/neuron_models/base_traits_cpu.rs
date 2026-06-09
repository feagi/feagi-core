//! CPU implementations of the base traits of all neuron models. Since these are on the CPU, we
//! can actually have methods for accessing the data!

use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagCPU;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_neuron::NPUNeuronMembranePotential;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cpu_wrappers::cortical_spatial::NPUNeuronIndexCorticalLocal;
use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::tables::cortical_structure_configuration::cortical_configuration::CorticalConfigurationBase;
use crate::neuron_models::base_traits_all_devices::{NeuronModelCorticalData, NeuronModelNeuronData, NeuronModelProcessor};
use crate::npu_descriptors::NPUGlobalBurstCounter;

/// Root CPU trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model
pub trait NeuronModelCorticalDataCPU<FGQ, NMQ, CCB>:
NeuronModelCorticalData<FGQ, NMQ, CCB>
+ PDITagCPU
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ>
{
    // Implement any cortical level data members (or make members pub)
}

/// Root CPU trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model
pub trait NeuronModelNeuronDataCPU<FGQ, NMQ, CCB, NMCD>:
NeuronModelNeuronData<FGQ, NMQ, CCB, NMCD>
+ PDITagCPU
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ>,
    NMCD: NeuronModelCorticalDataCPU<FGQ, NMQ, CCB>,
{
    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members (or make members pub)
    fn create_blank_neuron(
        neuron_linear_index: &NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_area_configuration: &CCB,
        cortical_area_data: &NMCD,
    ) -> Self;
    
}

/// Root base trait for defining neuron firing and other dynamics on the CPU.
/// Does NOT store actual data
pub trait NeuronModelProcessorCPU<FGQ, NMQ, CCB, NMCD, NMND>:
NeuronModelProcessor<FGQ, NMQ, CCB, NMCD, NMND>
+ PDITagCPU
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCB: CorticalConfigurationBase<FGQ, NMQ>, // TODO CPU version
    NMCD: NeuronModelCorticalDataCPU<FGQ, NMQ, CCB>,
    NMND: NeuronModelNeuronDataCPU<FGQ, NMQ, CCB, NMCD>
{
    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false
    fn process_neuron_potential
    (
        &self,
        incoming_potential: &NPUNeuronMembranePotential<NMQ::CorticalPotentialQuant::NeuronPotentialQuant>,
        neuron_linear_index: &NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
        burst_index: &NPUGlobalBurstCounter<FGQ::GlobalBurstIndexQuant>,
        cortical_area_configuration: &CCB,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NPUNeuronMembranePotential<NMQ::CorticalPotentialQuant::NeuronPotentialQuant>
    ) -> bool;



    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        cortical_area_data: &mut NMCD)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        neuron_linear_index: &FGQ::NeuronIndexCountQuant,
        neuron_model_data: &mut NMND)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

}