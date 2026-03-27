use feagi_structures::define_quantizable_value_type_family;

//region Fire Threshold
define_quantizable_value_type_family!(FireThreshold);


//endregion

//region Membrane Potential
define_quantizable_value_type_family!(MembranePotential);

//endregion

//region Fire Threshold Limit
define_quantizable_value_type_family!(FireThresholdLimit);


//endregion

//region PSP Multiplier
define_quantizable_value_type_family!(PSPMultiplier);


//endregion

//region Burst Delta Count
define_quantizable_uint_type_family!(BurstCount);


//endregion

//region Neuron Index

crate::define_quantizable_uint_type_family!(NeuronNPUIndex);

//endregion

//region Cortical Area Index
define_quantizable_uint_type_family!(CorticalAreaIndex);


//endregion

//region Synapse Index
define_quantizable_uint_type_family!(SynapseIndex);


//endregion

