use feagi_data::neurons::{NeuronCorticalLocalIndex, NeuronMembranePotential};
use crate::neuron_model::cortical_area::burst_index_rollover_handling::neuron_burst_index_rollover_handling::NeuronModelBurstIndexRolloverHandling;
use crate::neuron_model::cortical_area::neuron_history::neuron_history::NeuronModelHistory;
use crate::neuron_model::cortical_area::cortical_data::NeuronModelCorticalData;
use crate::neuron_model::neuron::neuron_data::NeuronModelNeuronData;
use crate::neuron_model::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::neuron_model::neuron_model_implementations::generated_enums::{
    NeuronModelType, NeuronModelTypeAndQuantizationNested, NeuronModelTypeAndQuantizationPacked,
};
use crate::wrapped_indexes::BurstIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron_model::cortical_area::cortical_layout::cortical_layout::CorticalLayout;
use crate::neuron_model::cortical_area::cortical_layout::cortical_layout_enum::CorticalLayoutTypeEnum;

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModel<FIQ, NMQ>: Sized
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
{
    /// How the neurons are laid out within the cortical area
    type CorticalLayout: CorticalLayout<FIQ>;

    /// The type of cortical data this neuron model needs
    type CorticalData: NeuronModelCorticalData<NMQ>;
    /// The per neuron data needed by this neuron model. To have none, use `EmptyPerNeuronData`
    type NeuronData: NeuronModelNeuronData<NMQ>;

    /// The type (including a 'none') type of neuron history
    type NeuronHistoryType: NeuronModelHistory<FIQ>;

    /// Allows denoting a custom function to be called per neuron with the burst index is about to
    /// roll over. Most models don't need to do this and should use `NeuronModelNoSpecialBurstIndexRolloverHandling`
    /// in their implementation.
    type BurstIndexRolloverHandling: NeuronModelBurstIndexRolloverHandling<FIQ, NMQ, Self::CorticalData, Self::NeuronData>;

    /// When creating a neuron, how far ba
    const INITIAL_BURST_INDEX_OFFSET: BurstIndex<FIQ::GlobalBurstIndexQuant> = BurstIndex::QUANT_MAX;

    // Proxied properties, here to make using this easier
    /// A flat enum denoting what type of neuron model this is
    const NEURON_MODEL: NeuronModelType = NMQ::NEURON_MODEL;
    /// A flat enum value denoting the quantization level of this neuron model instance
    const NEURON_QUANTIZATION: NMQ::QuantLevelType = NMQ::NEURON_QUANTIZATION;
    /// A nested enum that denotes both the neuron model and the quantization at runtime.
    const NEURON_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationNested = NMQ::NESTED_NEURON_MODEL_AND_QUANTIZATION;
    /// A flat enum (byte) that denotes both the neuron model and the quantization at runtime. Mainly
    /// useful for NPU
    const NEURON_PACKED_MODEL_AND_QUANTIZATION: NeuronModelTypeAndQuantizationPacked = NMQ::PACKED_NEURON_MODEL_AND_QUANTIZATION;

    /// How the neurons are laid out within the cortical area
    const NEURON_MODEL_LAYOUT: CorticalLayoutTypeEnum = Self::CorticalLayout::CORTICAL_LAYOUT;

    /// If the neuron model has per neuron model. This will always be the case except if your model
    /// uses `EmptyPerNeuronData`
    const NEURON_MODEL_USES_PER_NEURON_DATA: bool = Self::NeuronData::NEURON_MODEL_USES_PER_NEURON_DATA;




    /// Neuron received input potential. Process it, updating any internal states and
    /// update this neurons potential. Return true if it results in this neuron firing, otherwise
    /// return false.
    fn process_incoming_potential_for_dimensional_area(
        incoming_potential: &NeuronMembranePotential<NMQ::MembranePotentialQuant>,
        neuron_linear_index: &NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>,
        burst_index: &BurstIndex<FIQ::GlobalBurstIndexQuant>,
        cortical_layout: &Self::CorticalLayout,
        neuron_history: &Self::NeuronHistoryType,
        cortical_area_data: &Self::CorticalData,
        neuron_model_data: &mut Self::NeuronData,
        this_neuron_potential: &mut NeuronMembranePotential<NMQ::MembranePotentialQuant>,
    ) -> bool;

}

// NOTE: You will need to add neuron layouts trait off this trait to have the neuron model
// to actually be usable!
