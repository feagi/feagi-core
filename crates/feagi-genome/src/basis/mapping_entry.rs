use feagi_basis::feagi_basis_proc_common::feagi_id;

feagi_id! {
    /// Defines a specific mapping entry within the mapping between 2 cortical areas
    pub MappingEntryID: 8,
    random,
    base64 = Base64EncodedMappingEntryID,
}
