use feagi_structures::{define_quantizable_percentage_type_family, define_quantizable_uint_type_family, define_quantizable_value_type_family};
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType, QuantizationLevel};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndexQuantization;
use feagi_structures::neurons::descriptors::NeuronCount;

//region NPU Quantization Sets

/// Defines the burst index and cortical indexing across the entire NPU, as it needs to be
/// synced across structures
pub trait NPUGlobalQuantization {
    type GlobalBurstIndexQuant: QuantizableUIntType;
    type CorticalIndexCountQuant: CorticalAreaIndexQuantization; // We want this to be global since synapses will go between cortical indexes
    type NeuronIndexCountQuant: QuantizableUIntType; // Ditto for neuron indexes
    
}

/// Shared Quantization details that all neuron types implement in some manner
pub trait NPUBaseNeuronQuantization {
    const GENERAL_CORTICAL_AREA_QUANTIZATION_LEVEL: QuantizationLevel;
    type ValueQuant: QuantizableValueType;
    type PercentageQuant: QuantizablePercentType;
    type BurstDeltaQuant: QuantizableUIntType;
}

/// Dimensional neurons quantization level for their voxel coordinates
pub trait NPUDimensionalNeuronQuantization: NPUBaseNeuronQuantization {
    type CoordQuant: QuantizableUIntType;
}


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

//region Synapse Index
define_quantizable_uint_type_family!(SynapseIndex);


//endregion

//region Synapse Count
define_quantizable_uint_type_family!(SynapseCount);

//endregion

//region Synapse Bundle Index
define_quantizable_uint_type_family!(SynapseBundleIndex);

//endregion

//region Synapse Bundle Count
define_quantizable_uint_type_family!(SynapseBundleCount);

//endregion

//endregion


//region Value (float-ish) Quantizations


//region NPU Neuron Membrane Potential (separate implementation here for this crate
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

