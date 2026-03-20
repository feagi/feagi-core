use std::fmt::{Debug, Display};


//region Percentage Scale
crate::define_quantizable_value_type_family!(PercentageScale);

#[cfg(not(feature = "alloc"))]
pub trait PercentageScale:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
}

#[cfg(feature = "alloc")]
pub trait PercentageScale:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PercentageScale for PercentageScaleF64 {}
impl PercentageScale for PercentageScaleF32 {}
impl PercentageScale for PercentageScaleF16 {}
impl PercentageScale for PercentageScaleU8 {}
//endregion

//region PSP Multiplier
crate::define_quantizable_value_type_family!(PSPMultiplier);

#[cfg(not(feature = "alloc"))]
pub trait PSPMultiplier:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
}

#[cfg(feature = "alloc")]
pub trait PSPMultiplier:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PSPMultiplier for PSPMultiplierF64 {}
impl PSPMultiplier for PSPMultiplierF32 {}
impl PSPMultiplier for PSPMultiplierF16 {}
impl PSPMultiplier for PSPMultiplierU8 {}
//endregion

//region Burst Delta Count
crate::define_quantizable_uint_type_family!(BurstDeltaCount);

#[cfg(not(feature = "alloc"))]
pub trait BurstDeltaCount:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{
}

#[cfg(feature = "alloc")]
pub trait BurstDeltaCount:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl BurstDeltaCount for BurstDeltaCountU64 {}
impl BurstDeltaCount for BurstDeltaCountU32 {}
impl BurstDeltaCount for BurstDeltaCountU16 {}
impl BurstDeltaCount for BurstDeltaCountU8 {}
//endregion

//region Interneuron Index
crate::define_quantizable_uint_type_family!(InterneuronIndex);

#[cfg(not(feature = "alloc"))]
pub trait InterneuronIndex:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{
}

#[cfg(feature = "alloc")]
pub trait InterneuronIndex:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl InterneuronIndex for InterneuronIndexU64 {}
impl InterneuronIndex for InterneuronIndexU32 {}
impl InterneuronIndex for InterneuronIndexU16 {}
impl InterneuronIndex for InterneuronIndexU8 {}
//endregion

//region Cortical Area Index
crate::define_quantizable_uint_type_family!(CorticalAreaIndex);

#[cfg(not(feature = "alloc"))]
pub trait CorticalAreaIndex:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{
}

#[cfg(feature = "alloc")]
pub trait CorticalAreaIndex:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl CorticalAreaIndex for CorticalAreaIndexU64 {}
impl CorticalAreaIndex for CorticalAreaIndexU32 {}
impl CorticalAreaIndex for CorticalAreaIndexU16 {}
impl CorticalAreaIndex for CorticalAreaIndexU8 {}
//endregion

//region Synapse Index
crate::define_quantizable_uint_type_family!(SynapseIndex);

#[cfg(not(feature = "alloc"))]
pub trait SynapseIndex:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{
}

#[cfg(feature = "alloc")]
pub trait SynapseIndex:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl SynapseIndex for SynapseIndexU64 {}
impl SynapseIndex for SynapseIndexU32 {}
impl SynapseIndex for SynapseIndexU16 {}
impl SynapseIndex for SynapseIndexU8 {}
//endregion

//region PSP Multiplier
crate::define_quantizable_value_type_family!(PSPMultiplier);

#[cfg(not(feature = "alloc"))]
pub trait PSPMultiplier:
Copy + Clone + Send + Sync + QuantizableValue + 'static
{
}

#[cfg(feature = "alloc")]
pub trait PSPMultiplier:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableValue + 'static
{
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl PSPMultiplier for PSPMultiplierF64 {}
impl PSPMultiplier for PSPMultiplierF32 {}
impl PSPMultiplier for PSPMultiplierF16 {}
impl PSPMultiplier for PSPMultiplierU8 {}
//endregion
