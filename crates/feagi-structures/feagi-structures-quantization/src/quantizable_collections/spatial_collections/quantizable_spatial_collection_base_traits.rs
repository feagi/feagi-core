use crate::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_collections::quantizable_collection_base_traits::QuantizableCollectionBaseTrait;

pub trait QuantizableSpatialCollectionBaseTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Value
>:
QuantizableCollectionBaseTrait<QuantIndex, WrappedLinearIndexCountType, Value>
{
    // just a common tag
}