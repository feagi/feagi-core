use crate::core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};

/// Specifies that the type can be used for Linear Indexing
pub trait LinearIndexCountType:
SupportsBasicCoreMathOps
+ SupportsUintOps
{
}

impl LinearIndexCountType for usize {
}

impl LinearIndexCountType for u8 {
}

impl LinearIndexCountType for u16 {
}

impl LinearIndexCountType for u32 {
}

#[cfg(feature = "support_64bit_indexing")]
impl LinearIndexCountType for u64 {
}