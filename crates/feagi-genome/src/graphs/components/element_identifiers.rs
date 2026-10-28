use core::hash::Hash;
use feagi_basis::feagi_id::feagi_id::FeagiID;
use crate::basis::cortical_area::cortical_id::{CorticalID, FauxCorticalID};


/// Defines what type of identifier is used for Graph Elements
pub trait GraphElementIdentifiers {
    type CorticalIdentifier: Hash;
    type MappingIdentifier: Hash;
    type MappingEntryIdentifier;
}

/// All identifiers are concrete, as in they are known unique for actual use
pub struct ConcreteGraphElementIdentifiers;

impl GraphElementIdentifiers for ConcreteGraphElementIdentifiers {
    type CorticalIdentifier = CorticalID;
    type MappingIdentifier = ();
    type MappingEntryIdentifier = ();
}


pub struct FauxGraphElementIdentifiers;

impl GraphElementIdentifiers for FauxGraphElementIdentifiers {
    type CorticalIdentifier = FauxCorticalID;
    type MappingIdentifier = ();
    type MappingEntryIdentifier = ();
}

// TODO mixed???