use feagi_basis::feagi_id::feagi_id::FeagiID;

/// The given identifier has a faux identifier
pub trait HasFauxIdentifier<const ID_BYTE_COUNT: usize>: FeagiID<ID_BYTE_COUNT> {
    type FauxID: FauxIdentifier<ID_BYTE_COUNT, RealID=Self>;
    
}

/// Used when moving identifier structures with relationships between each other but
/// you do not want to risk collision
pub trait FauxIdentifier<const ID_BYTE_COUNT: usize>: FeagiID<ID_BYTE_COUNT> {
    type RealID: HasFauxIdentifier<ID_BYTE_COUNT, FauxID=Self>;
}