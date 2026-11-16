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


impl From<BrainRegionID> for BrainRegionAncestryID {
    fn from(value: BrainRegionID) -> Self {
        Self(value.0)
    }
}

/// Allows definition of a pair of brain region and brain region ancestry IDs easily in constructors
pub enum BrainRegionIdAndAncestryConstructor {
    /// Self ID and Ancestry ID will be an identical uuidv7 based on current time
    IdenticalNow(),
    /// Self ID and Ancestry ID will be an identical uuidv7 that was given
    IdenticalSpecific(BrainRegionID),
    /// Self ID and Ancestry ID will be specific values
    Unique(BrainRegionID, BrainRegionAncestryID)
}

impl BrainRegionIdAndAncestryConstructor {
    pub fn into_pair(self) -> (BrainRegionID, BrainRegionAncestryID) {
        match self {
            BrainRegionIdAndAncestryConstructor::IdenticalNow() => {
                let id = BrainRegionID::new_now();
                (id.clone(), id.into())
            }
            BrainRegionIdAndAncestryConstructor::IdenticalSpecific(id) => {
                (id.clone(), id.into())
            }
            BrainRegionIdAndAncestryConstructor::Unique(id, aid) => {
                (id, aid)
            }
        }
    }
}