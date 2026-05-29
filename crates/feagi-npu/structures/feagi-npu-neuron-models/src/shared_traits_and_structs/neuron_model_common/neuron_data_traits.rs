use feagi_structures::feagi_data::feagi_ecs::collection::FeagiECSCollectionDataLivesOnCPU;
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedDecimalTrait;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};
use crate::shared_traits_and_structs::neuron_model_common::cortical_data_traits::{NeuronModelCorticalDataCommonCPU, NeuronModelCorticalDataCommonDevice, NeuronModelCorticalDataDimensionalDevice, NeuronModelCorticalDataLinearCPUTemplate, NeuronModelCorticalDataLinearDevice};
use crate::shared_traits_and_structs::neuron_model_common::cortical_state_data::{CorticalDataStateDataCommonCPU, CorticalDataStateDataCommonDevice, CorticalDataStateDataDimensionalDevice, CorticalDataStateDataLinearDevice};

//region Base "Tag" Traits


#[doc(hidden)]
/// Root base trait for the data of a neuron model
pub trait NeuronDataCommonDevice<CAIQ, NMQ, CSD, CNMCD>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
    CSD: CorticalDataStateDataCommonDevice<CAIQ>,
    CNMCD: NeuronModelCorticalDataCommonDevice<CAIQ, NMQ>
{
    // As per CorticalAreasIndexQuantization, this takes in GlobalBurstIndexQuant,
    // and NeuronIndexCountQuant. These are not settable by the model and instead picked by
    // FEAGI's NPU

    // NOTE: Implementations of Neuron Models do not store their own membrane potential! They
    // will be passed in by reference if need be!

    const NEURON_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;

    // method for calling right before burst index rollover

    // method for calculating the resultant neuron potential of the neuron following input
}

//endregion

//region CPU

#[doc(hidden)]
/// Root base trait for the data of a neuron model
pub trait NeuronDataCommonCPU<CAIQ, NMQ, CSD, CNMCD>:
NeuronDataCommonDevice<CAIQ, NMQ, CSD, CNMCD>
where
    CAIQ: CorticalAreasIndexQuantization,
    NMQ: CorticalAreaModelQuantizationBase,
    CSD: CorticalDataStateDataCommonCPU<CAIQ>, // Can be dimensional or linear
    CNMCD: NeuronModelCorticalDataCommonCPU<CAIQ, NMQ>
{

    /// Neuron received input potential. Process it, updating any internal states and update
    /// this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false. Note that incoming neuron potential may be an arbitrary quantization.
    fn process_neuron_potential<IPQuant: QuantizedDecimalTrait>(
        &mut self,
        incoming_neuron_potential: &IPQuant,
        this_neuron_linear_index: &CAIQ::NeuronIndexCountQuant,
        cortical_state_data: &CSD,
        cortical_neuron_model_data: &CNMCD,
        self_neuron_potential: &mut NMQ::NeuronPotentialQuant
    ) -> bool;

    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_neuron_for_burst_index_rollover(&mut self) {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have NEURON_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }
}






//endregion



