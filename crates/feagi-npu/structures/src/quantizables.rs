use feagi_structures::{define_quantizable_percentage_type_family, define_quantizable_uint_type_family, define_quantizable_value_type_family};
use feagi_structures::neurons::descriptors::{NeuronMembranePotential, NumberNeuronsPerVoxel};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;

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

//endregion

//region NPU Synapse Index
define_quantizable_uint_type_family!(NPUSynapseIndex);


//endregion

//endregion


//region Value (float-ish) Quantizations

// Neuron Membrane Potential is in the 'Feagi-Structures' Crate!

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


