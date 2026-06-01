//! CPU implementations of the base traits of all neuron models. Since these are on the CPU, we
//! can actually have methods for accessing the data!

use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagCPU;
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedDecimalTrait;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::shared_traits_and_structs::base_traits_all_devices::{NeuronModelProcessor, CorticalModelData, NeuronModelData};
use crate::shared_traits_and_structs::cortical_configuration::{CorticalConfiguration};

/// Root CPU trait for all cortical data implementations, essentially any cortical level data shared
/// by all neurons in a cortical area of a given neuron model
pub trait CorticalModelDataCPU<FGQ, NMQ>:
CorticalModelData<FGQ, NMQ>
+ PDITagCPU
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{
    // Implement any cortical level data members (or make members pub)
}

/// Root CPU trait for all neuron data implementation, essentially per neuron data for a given
/// neuron model
pub trait NeuronModelDataCPU<FGQ, NMQ>:
NeuronModelData<FGQ, NMQ>
+ PDITagCPU
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
{
    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    // Implement any per-neuron level data members (or make members pub)
}


/// Root base trait for defining neuron firing and other dynamics on the CPU.
/// Does NOT store actual data
pub trait NeuronModelProcessorCPU<FGQ, NMQ, CCC, CMD, NMD>:
NeuronModelProcessor<FGQ, NMQ, CCC, CMD, NMD>
+ PDITagCPU
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfiguration<FGQ>, // Extend to dimensional
    CMD: CorticalModelDataCPU<FGQ, NMQ>,
    NMD: NeuronModelDataCPU<FGQ, NMQ>
{
    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false. Note that incoming neuron potential may be an arbitrary quantization.
    fn process_neuron_potential<IPQuant: QuantizedDecimalTrait,>
    (
        &self,
        incoming_neuron_potential: &IPQuant,
        neuron_linear_index: &FGQ::NeuronIndexCountQuant,
        burst_index: &FGQ::GlobalBurstIndexQuant,
        cortical_area_configuration: &CCC,
        cortical_area_data: &CMD,
        neuron_model_data: &mut NMD,
        this_neuron_potential: &mut NMQ::NeuronPotentialQuant
    ) -> bool;

    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_cortical_data_for_burst_index_rollover(
        &self,
        cortical_area_data: &mut CMD)
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
        neuron_model_data: &mut NMD)
    {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }

}