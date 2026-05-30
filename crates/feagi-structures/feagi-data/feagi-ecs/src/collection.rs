//! A collection is the top level data holder, denotes that all of its contained data is on a
//! specific device and independent. These are the structures that FeagiECSSystems will operate
//! upon, to ensure maximum performance via parallelization. They can contain elements directly
//! or element sets

use crate::element::FeagiECSElement;
use crate::tag_device::FeagiECSTagGenericDevice;

/// Shared trait among all Feagi ECS Device traits. While only those that extend FeagiECSTagCPU
/// can have their internal data accessible and workable from standard Rust CPU systems, it is
/// possible for structs implementing other devices to still have "Metadata" that allows the CPU
/// to at least have some context of what is being held
pub trait FeagiECSCollection<Element>: FeagiECSTagGenericDevice 
where
    Element: FeagiECSElement

{
    
}
