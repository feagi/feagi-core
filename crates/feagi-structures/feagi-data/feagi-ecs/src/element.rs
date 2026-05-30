//! An element is the "lowest" form of data in an ECS system. Lives entirely on one device with
//! no promised access to the data from the CPU. You do not operate on them individually, but
//! through collections (or indirectly via element sets)
//! Its probably best to not use any advanced (wrapped) types in these elements, as other
//! device implementations may not be able to use Rust traits and implementations. Stick to
//! the most basic data types like ints, uints, and floats

use crate::tag_device::FeagiECSTagGenericDevice;

/// Root trait for ECS Elements, simply states that the element data lives on some device. As such,
/// anything inheriting only this trait cannot have any data access methods of any sort. This
/// is essentially just a tag
pub trait FeagiECSElement: FeagiECSTagGenericDevice {}


