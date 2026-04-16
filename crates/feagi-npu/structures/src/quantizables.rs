use feagi_structures::{define_quantizable_percentage_type_family, define_quantizable_uint_type_family, define_quantizable_value_type_family};
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::neurons::descriptors::NeuronCount;

/// Defines the quantization for all most uses in this crate // TODO this may need to be moved up a level?
pub trait NPUQuantization {
    type NeuronIndex: QuantizableUIntType;
    type SynapseIndex: QuantizableUIntType;
    type SynapseBundleIndex: QuantizableUIntType;
    type CorticalIndex: QuantizableUIntType;
    type Coord: QuantizableUIntType;
    type BurstDelta: QuantizableUIntType;
    type BurstIndex: QuantizableUIntType;
    type Value: QuantizableValueType;
    type Percentage: QuantizablePercentType;
}

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

//region NPU Synapse Index
define_quantizable_uint_type_family!(NPUSynapseIndex);


//endregion

//region Synapse Count
define_quantizable_uint_type_family!(SynapseCount);

//endregion

//region Synapse Bundle Index
define_quantizable_uint_type_family!(SynapseBundleIndex);

//endregion

//endregion


//region Value (float-ish) Quantizations


//region NPU Neuron Membrane Potential (seperate implementation here for this crate
define_quantizable_value_type_family!(NPUNeuronMembranePotential);

impl NPUNeuronMembranePotential<f32> {
    pub fn update_threshold_nonplastic(&mut self, synaptic_weight: SynapticWeight<f32>, upstream_potential: NPUNeuronMembranePotential<f32>) {
        self.0 = self.0 * synaptic_weight.0 * upstream_potential.0;
    }
}
//endregion

//region Fire Threshold
define_quantizable_value_type_family!(FireThreshold);

//endregion

//region Fire Threshold Limit
define_quantizable_value_type_family!(FireThresholdLimit);

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


//endregion


