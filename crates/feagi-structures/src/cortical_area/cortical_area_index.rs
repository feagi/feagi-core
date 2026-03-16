use core::fmt::{Debug, Display};
use crate::base_quantizable::unsigned_integer::QuantizableUInt;

//region Cortical Area Index
pub type CorticalAreaIndexUsize = usize;
pub type CorticalAreaIndexU64 = u64;
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

impl CorticalAreaIndex for usize {

}

impl CorticalAreaIndex for u64 {

}

impl CorticalAreaIndex for u32 {

}

impl CorticalAreaIndex for u16 {

}

impl CorticalAreaIndex for u8 {

}

//endregion