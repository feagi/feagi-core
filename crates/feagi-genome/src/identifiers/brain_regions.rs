use feagi_basis::create_wrapped_uuid;

create_wrapped_uuid!(
    /// The ID of a region. This is created fresh with the current timestamp whenever a Brain Region
    /// is created, regardless of its new or if its from an almagamation
    BrainRegionID,
    v7);

create_wrapped_uuid!(
    /// The "Original" ID of a region. A new one is created only on the first creation of a
    /// brain region. On almagamation, this is preserved.
    BrainRegionAncestryID,
    v7);