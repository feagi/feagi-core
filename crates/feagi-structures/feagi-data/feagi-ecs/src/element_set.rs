//! Sometimes we have multiple levels of nesting within our structures before we reach the
//! collection level. An Element Set is simply an element that denotes that it groups other
//! elements within itself, and may have its own "header" data associated with it. Otherwise the
//! same use-case and restrictions from Elements continue to apply. It should be noted that
//! Feagi ECS Element Sets can be nested

use crate::element::{FeagiECSElement};

/// Root trait for ECS Element Sets, denotes an ECS Element but that it holds a collection
/// of elements (or other element sets) inside of it and potentially some header data to itself.
/// Otherwise is just an FeagiECSElement, so it cannot have any usable data access from the CPU.
/// Technically may contain multiple child elements, but just pick one to satisfy the trait, or 
/// unify them with a tuple
pub trait FeagiECSElementSet<ChildElement>: FeagiECSElement // Already extends FeagiECSTagGenericDevice
where
    ChildElement: FeagiECSElement
{}

// NOTE: Its probably good practice not to put the non-universal data types as generic parameters,
// as collections may vectorize different implementations