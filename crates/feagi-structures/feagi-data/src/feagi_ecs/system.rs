//! An ECS System effectively is design to operate on ECS components on mass parallel (on
//! supporting hardware at least). It does not store the data itself, but rather acts through
//! (mutable) slices of components.


// TODO Async vs Sync as tags, how would we handle the System though?

use crate::feagi_ecs::component::FECSComponentBase;

/// A tag to establish something is a FEAGI ECS System
pub trait FeagiECSSystemBase { }

// A common scenario where we need to be able to mass copy data from one Data source to another
pub trait FECSSystemCopyComp<
    Source: FECSComponentBase,
    Destination: FECSComponentBase>
{
    fn copy_data_from_to(source: &[Source], destination: &mut [Source]);
}