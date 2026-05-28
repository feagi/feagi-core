use crate::core_numerical_types::{SupportsBasicCoreMathOps, SupportsUintOps};
use crate::quantizable_linear::base_types::{QuantizedElementBase, QuantizedIndexCountTrait, QuantizedSignedIntegerTrait, QuantizedUnsignedIntegerTrait};
use crate::quantizable_linear::base_types::QuantizedDecimalTrait;


// region Wrapper Traits

#[doc(hidden)]
/// Base Quantized Element Wrapper Base. This trait makes use of some unsafe functionality. ONLY
/// use the wrappers to generate these wrappers, or you will be in for a bad time!
pub trait QuantizedElementWrapperBase<QE: QuantizedElementBase>:
SupportsBasicCoreMathOps
{
    fn wrap(quantizable: QE) -> Self;
    
    
    fn wrap_ref(quantizable: &QE) -> &Self; // Scary unsafe implementation behind the scenes!

    fn unwrap(self) -> QE;
    
    /// Get ref access to the wrapped quant type
    fn quant_ref(&self) -> &QE;

    /// Get mut ref access to the wrapped quant type
    fn quant_ref_mut(&mut self) -> &mut QE;
    
    // Cant do const functions here
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

pub trait QuantizedElementWrapperSignedInteger<QE: QuantizedSignedIntegerTrait>:
QuantizedElementWrapperBase<QE>
{

}

pub trait QuantizedElementWrapperDecimal<QE: QuantizedDecimalTrait>:
QuantizedElementWrapperBase<QE>
{

}

//endregion
