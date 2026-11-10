



/// The last 6 bytes of a cortical ID, which each cortical type gives its own meaning. Converting to
/// bytes and back must return the exact same bytes, or two different byte patterns could mean the
/// same cortical area
pub trait CorticalAreaInnerID: Copy + Eq + core::hash::Hash + core::fmt::Debug + Into<[u8; 6]> + TryFrom<[u8; 6]> {}

/// just 6 bytes counts as well
impl CorticalAreaInnerID for [u8; 6] {}




/// Used in place of the `CorticalIdentifier` in transformation / addition functions to eliminate
/// odds of collision while still having something mappings can match against
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct FauxCorticalIdentifier(u64);

impl FauxCorticalIdentifier {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}
