use crate::cortical_area::CorticalID;

/// Identifies a set of mapping entries between two cortical areas in a directional matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorticalMappingID {
    source: CorticalID,
    destination: CorticalID,
}

impl CorticalMappingID {
    pub fn new(source: CorticalID, destination: CorticalID) -> Self {
        Self { source, destination }
    }
}
