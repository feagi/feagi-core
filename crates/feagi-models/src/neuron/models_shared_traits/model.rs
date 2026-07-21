use crate::burst_index::BurstIndex;
use crate::neuron::models_shared_traits::data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronMembranePotential};
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModel<FIQ, CPQ, NMCD, NMND>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialQuantization,
    NMCD: NeuronModelCorticalData<CPQ>,
    NMND: NeuronModelNeuronData<CPQ>,
{
    /// Set to true if the neuron model needs to be informed if the global burst index counter is
    /// about to overflow. Otherwise, set to false. Note that Neuron history takes care of this
    /// automatically so if thats the only time reference kept, then this can be false.
    const MODEL_NEURON_DATA_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER: bool;

    /// Set to true if the neuron model can run under dimensional cortical areas (cortical areas
    /// in which 
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_DIMENSIONAL: bool;

    /// Set to true if a neuron model can run under no cortical layout at all (neuron position
    /// within a cortical area is irrelevant, IE memory cortical areas)
    const MODEL_SUPPORTS_CORTICAL_LAYOUT_NONE: bool;

    /// The type of neuron history supported. Is empty if none are supported by the model
    type UsedNeuronHistory;

    /// Dimensional Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false. Panics if called when the model does not support this.
    fn process_neuron_potential_for_dimensional_layout_cortical_area(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        dimensional_cortical_dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::UsedNeuronHistory,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool;

    /// Memory Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false. Panics if called when the model does not support this.
    fn process_neuron_potential_for_none_layout_cortical_area(
        incoming_potential: &NeuronMembranePotential<CPQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        number_neurons: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        neuron_history: &Self::UsedNeuronHistory,
        cortical_area_data: &NMCD,
        neuron_model_data: &mut NMND,
        this_neuron_potential: &mut NeuronMembranePotential<CPQ::MembranePotentialQuant>,
    ) -> bool;

    fn default_neuron_writer_dimensional_layout_cortical_area(dimensional_cortical_dimensions: DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>) -> Option<impl NeuronModelWriter<CPQ, NMCD, NMND>>;

    fn default_neuron_writer_none_layout_cortical_area(number_neurons: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>) -> Option<impl NeuronModelWriter<CPQ, NMCD, NMND>>;
    
    /// If enabled via the const, this method will be called on all neurons of that
    /// neuron model type right before the global burst index overflows and resets to 0. Use this
    /// method to update any values that need to be updated in that case
    fn prepare_neuron_data_for_burst_index_rollover(
        &self,
        _neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        _neuron_model_data: &mut NMND,
    ) {
        // by default nothing. Override me if you have something you need to do, but remember
        // to have MODEL_NEEDS_TO_BE_INFORMED_OF_BURST_INDEX_ROLLOVER set to true
    }
}
