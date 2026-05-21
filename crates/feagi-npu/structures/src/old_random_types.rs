use feagi_structures::{define_quantizable_uint_type_family, define_quantizable_value_type_family, QuantizationLevel};
use feagi_structures::base_feagi_types::quantizable_types::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndexQuantization;

//region NPU Quantization Sets


/// Shared Quantization details that all neuron types implement in some manner

pub trait NPUSynapseQuantization {
    type SynapseValueType: QuantizableValueType;
    type SynapseIndexCountQuant: QuantizableUIntType;
    type SynapseBundleIndexCountQuant: QuantizableUIntType;
    type BurstDelay: QuantizableUIntType;
}

//endregion

//region UInt Quantizations

// Cortical Index and Number Neurons Per Voxel are in the 'Feagi-Structures' Crate!

//region Burst Global Index
define_quantizable_uint_type_family!(BurstGlobalIndex);

//endregion

//region Burst Delta
define_quantizable_uint_type_family!(BurstDelta);

//endregion

//region NPU Neuron Index
define_quantizable_uint_type_family!(NPUNeuronIndex);

impl<T: QuantizableUIntType> NPUNeuronIndex<T> {
    pub fn get_count_from_block(range: &core::ops::Range<NPUNeuronIndex<T>>) -> NeuronCount<T> {
        (range.end - range.start).0.into()
    }
}


//endregion

// Use neuron count from 'Feagi-Structures' Crate!


//endregion


//region Value (float-ish) Quantizations



//region Fire Threshold
define_quantizable_value_type_family!(FireThreshold);

//endregion

//region Fire Threshold Limit
define_quantizable_value_type_family!(FireThresholdLimit);

//endregion

//region Consecutive Fire Limit
define_quantizable_value_type_family!(ConsecutiveFireLimit);

//endregion

//region PSP Multiplier
define_quantizable_value_type_family!(PSPMultiplier);


//endregion

//region PSP Max
define_quantizable_value_type_family!(PSPMax);


//endregion

//region Synaptic Weight
define_quantizable_value_type_family!(SynapticWeight);


//endregion

//region Degeneracy Constant
define_quantizable_value_type_family!(DegeneracyConstant);


//endregion


//endregion



/*
//region Percentage Quantizations

//region Fire Threshold
define_quantizable_percentage_type_family!(NeuronExcitability);

//endregion

//region Synaptic Attractivity
define_quantizable_percentage_type_family!(SynapticAttractivity);

//endregion

//region Leak Variability
define_quantizable_percentage_type_family!(LeakVariability);

//endregion

//region Leak Coefficient
define_quantizable_percentage_type_family!(LeakCoefficient);

//endregion



 */
//endregion

