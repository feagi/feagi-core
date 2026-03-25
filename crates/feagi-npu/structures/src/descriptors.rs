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

//endregion

//region Burst Delta Count
crate::define_quantizable_uint_type_family!(BurstCount);

#[cfg(not(feature = "alloc"))]
pub trait BurstCount:
Copy + Clone + Send + Sync + QuantizableUInt + 'static
{
}

#[cfg(feature = "alloc")]
pub trait BurstCount:
Copy + Clone + Send + Sync + Debug + Display + Default + QuantizableUInt + 'static
{
}

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

//endregion
