use crate::core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
use crate::quantizable::base_types::{QuantizedElementBase, QuantizedIndexCountTrait};
use crate::quantizable::base_types::decimal::QuantizedDecimalTrait;
use crate::quantizable::base_types::unsigned_integer::QuantizedUnsignedIntegerTrait;


// region Wrapper Traits

/// Base Quantized Element Wrapper Base
pub trait QuantizedElementWrapperBase<QE: QuantizedElementBase>:
SupportsBasicCoreMathOps
{
    fn wrap(quantizable: QE) -> Self;

    fn unwrap(self) -> QE;

    /// Get ref access to the wrapped quant type
    fn quant_ref(&self) -> &QE;

    /// Get mut ref access to the wrapped quant type
    fn quant_ref_mut(&mut self) -> &mut QE;
}

pub trait QuantizedElementWrapperIndexCount<QE: QuantizedIndexCountTrait>:
QuantizedElementWrapperBase<QE>
+ SupportsUintOps
{

}

pub trait QuantizedElementWrapperUnsignedInteger<QE: QuantizedUnsignedIntegerTrait>:
QuantizedElementWrapperBase<QE>
+ SupportsUintOps
{

}

pub trait QuantizedElementWrapperDecimal<QE: QuantizedDecimalTrait>:
QuantizedElementWrapperBase<QE>
{

}

//endregion
