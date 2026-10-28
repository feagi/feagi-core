use feagi_basis::feagi_id::feagi_id;

feagi_id! {
    /// The ID of a region. This is created fresh with the current timestamp whenever a Brain Region
    /// is created, regardless of its new or if its from an almagamation
    pub BrainRegionID: 16,
    uuid = v7,
    base64 = Base64EncodedBrainRegionID,
}

feagi_id! {
    /// The "Original" ID of a region. A new one is created only on the first creation of a
    /// brain region. On almagamation, this is preserved.
    pub BrainRegionAncestryID: 16,
    uuid = v7,
    base64 = Base64EncodedBrainRegionAncestryID,
}
