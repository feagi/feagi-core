

//region Mapping ID

use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;

/// Defines a directional mapping between 2 cortical areas (or to itself)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct CorticalMappingID {
    pub source: CorticalIdentifier,
    pub destination: CorticalIdentifier,
}

impl CorticalMappingID {
    pub const fn new(source: CorticalIdentifier, destination: CorticalIdentifier) -> Self {
        Self { source, destination }
    }

    pub fn get_source(&self) -> &CorticalIdentifier {
        &self.source
    }

    pub fn get_destination(&self) -> &CorticalIdentifier {
        &self.destination
    }

    /// If the mapping points to itself
    pub fn is_circular(&self) -> bool {
        self.source == self.destination
    }
}

