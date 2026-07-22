use crate::neuron::model_and_quantization::{NestedNeuronModelTypeAndQuantization, NeuronModelType, PackedNeuronModelTypeAndQuantization};
use crate::neuron::model_extensions::neuron_burst_index_rollover_handling::NeuronModelBurstIndexRolloverHandling;
use crate::neuron::model_extensions::neuron_history::NeuronModelHistory;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Root base trait for defining neuron firing and other dynamics. Does NOT store actual data,
pub trait NeuronModel<FIQ, NMQ>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
{
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

    // Proxied properties, here to make using this easier
    /// A flat enum denoting what type of neuron model this is
    const NEURON_MODEL: NeuronModelType  = NMQ::NEURON_MODEL;
    /// A flat enum value denoting the quantization level of this neuron model instance
    const NEURON_QUANTIZATION: NMQ::QuantLevelType = NMQ::NEURON_QUANTIZATION;
    /// A nested enum that denotes both the neuron model and the quantization at runtime.
    const NEURON_MODEL_AND_QUANTIZATION: NestedNeuronModelTypeAndQuantization = NMQ::NESTED_NEURON_MODEL_AND_QUANTIZATION;
    /// A flat enum (byte) that denotes both the neuron model and the quantization at runtime. Mainly
    /// useful for NPU
    const NEURON_PACKED_MODEL_AND_QUANTIZATION: PackedNeuronModelTypeAndQuantization = NMQ::PACKED_NEURON_MODEL_AND_QUANTIZATION;
    /// If the neuron model has per neuron model. This will always be the case except if your model
    /// uses `EmptyPerNeuronData`
    const NEURON_MODEL_USES_PER_NEURON_DATA: bool = Self::NeuronData::NEURON_MODEL_USES_PER_NEURON_DATA;
}

// NOTE: You will need to add neuron layouts trait off this trait to have the neuron model
// to actually be usable!
