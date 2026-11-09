
/// Used in place of the `CorticalIdentifier` in transformation / addition functions to eliminate
/// odds of collision while still having something mappings can match against
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct FauxCorticalIdentifier(u64);

impl FauxCorticalIdentifier {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}



