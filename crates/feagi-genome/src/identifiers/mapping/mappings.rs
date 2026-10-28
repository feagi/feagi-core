use crate::basis::cortical_area::cortical_id::{CorticalID, FauxCorticalID};
use crate::identifiers::genome_identifier_error::GenomeIdentifierError;
use feagi_basis::feagi_id::feagi_id;

feagi_id! {
    /// Specifies a unique synapse mapping rule that describes the pattern of how synapses are
    /// connected between cortical areas in a mapping entry
    pub ConnectivityRuleID: 8,
    random,
    base64 = Base64EncodedConnectivityRuleID,
}

feagi_id! {
    /// Defines a specific mapping entry within the mapping between 2 cortical areas
    pub MappingEntryID: 8,
    random,
    base64 = Base64EncodedMappingEntryID,
}

//region Mapping ID

/// Defines a directional mapping between 2 cortical areas (or to itself)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct MappingID {
    pub source: CorticalID,
    pub destination: CorticalID,
}

impl MappingID {
    pub const fn new(source: CorticalID, destination: CorticalID) -> Self {
        Self { source, destination }
    }

    pub fn get_source(&self) -> &CorticalID {
        &self.source
    }

    pub fn get_destination(&self) -> &CorticalID {
        &self.destination
    }

    /// If the mapping points to itself
    pub fn is_circular(&self) -> bool {
        self.source == self.destination
    }
}

/// During amalgamation, denotes a connection between 2 incoming areas via their faux cortical id,
/// or an open ended one on either end. This struct cannot be open on both ends
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
pub struct FauxMappingID {
    source: Option<FauxCorticalID>,
    destination: Option<FauxCorticalID>,
}

impl FauxMappingID {
    pub fn new(source: Option<FauxCorticalID>, destination: Option<FauxCorticalID>) -> Result<Self, GenomeIdentifierError> {
        if source.is_none() && destination.is_none() {
            return Err(GenomeIdentifierError::InvalidFauxStruct("Source and Destination Cortical IDs cannot both be null!"));
        }
        Ok(Self { source, destination })
    }

    pub fn get_source(&self) -> &Option<FauxCorticalID> {
        &self.source
    }

    pub fn get_destination(&self) -> &Option<FauxCorticalID> {
        &self.destination
    }

    /// If the mapping points to itself
    pub fn is_circular(&self) -> bool {
        self.source == self.destination
    }
}

//endregion