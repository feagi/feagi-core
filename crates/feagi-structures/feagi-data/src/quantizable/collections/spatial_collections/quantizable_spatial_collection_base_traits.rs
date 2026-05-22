use crate::quantizable::base_types::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable::collections::quantizable_collection_base_traits::QuantizableCollectionBaseTrait;

pub trait QuantizableSpatialCollectionBaseTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Value
>:
QuantizableCollectionBaseTrait<QuantIndex, WrappedLinearIndexCountType, Value>
{
    // just a common tag
}