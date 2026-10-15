
/// Used during amalgamation for references to cortical areas during transition
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FauxCorticalID(u64);

pub trait CorticalAreaObjectIdentifier: core::hash::Hash + Clone + Copy + Eq {}








