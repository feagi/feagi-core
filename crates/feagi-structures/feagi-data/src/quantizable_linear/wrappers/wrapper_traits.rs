use crate::core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
use crate::quantizable_linear::base_types::{QuantizedElementBase, QuantizedIndexCountTrait, QuantizedSignedIntegerTrait};
use crate::quantizable_linear::base_types::QuantizedDecimalTrait;


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

pub trait QuantizedElementWrapperUnsignedInteger<QE: QuantizedIndexCountTrait>:
QuantizedElementWrapperBase<QE>
+ SupportsUintOps
{

}

pub trait QuantizedElementWrapperSignedInteger<QE: QuantizedSignedIntegerTrait>:
QuantizedElementWrapperBase<QE>
{

}

pub trait QuantizedElementWrapperDecimal<QE: QuantizedDecimalTrait>:
QuantizedElementWrapperBase<QE>
{

}

//endregion
