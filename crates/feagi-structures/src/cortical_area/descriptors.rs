use core::fmt::{Debug, Display};
use crate::base_quantizable::quantizable_uints::QuantizableUInt;

//region Cortical Area Index
pub type CorticalAreaIndexU32 = u32;
pub type CorticalAreaIndexU16 = u16;
pub type CorticalAreaIndexU8 = u8;

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

impl CorticalAreaIndex for u32 {

}

impl CorticalAreaIndex for u16 {

}

impl CorticalAreaIndex for u8 {

}

//endregion