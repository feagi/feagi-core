use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::identifiers::cortical_id::CorticalID;
use crate::identifiers::identifiers::{BrainGraphIdentifier, GenomeIdentifier};

/// Identifies a set of mapping entries between two cortical areas in a directional matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorticalMappingID {
    source: CorticalID,
    destination: CorticalID,
}

impl CorticalMappingID {
    pub fn new(source: CorticalID, destination: CorticalID) -> Self {
        Self { source, destination }
    }
    
    pub fn source_cortical_id(&self) -> &CorticalID {
        &self.source
    }

    pub fn destination_cortical_id(&self) -> &CorticalID {
        &self.destination
    }
}

impl BrainGraphIdentifier for CorticalMappingID {}

impl Display for CorticalMappingID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FromStr for CorticalMappingID {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl GenomeIdentifier for CorticalMappingID {}