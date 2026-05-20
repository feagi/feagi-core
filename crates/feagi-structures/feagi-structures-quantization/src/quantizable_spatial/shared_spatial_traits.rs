use crate::quantizable_base::index_count::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};

/// Common base for all coordinate / dimension spatial data
pub trait SpatialBaseXDTrait<QuantIndex: QuantizedIndexCountTrait>:
Copy
+ Clone
+ Send
+ Sync
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::cmp::PartialOrd

+ 'static
{

}

/// Common base for all unsigned coordinate / dimension spatial data
pub trait SpatialUnsignedBaseXDTrait<QuantIndex: QuantizedIndexCountTrait>:
Copy
+ Clone
+ Send
+ Sync
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign

+ core::fmt::Debug
+ core::fmt::Display
+ 'static
{
    type LinearIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>;
}
