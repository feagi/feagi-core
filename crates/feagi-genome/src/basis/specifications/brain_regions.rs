use std::collections::HashSet;
use feagi_basis::feagi_id::feagi_id;
use crate::basis::specifications::cortical_area::cortical_id::CorticalID;

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

// TODO comparisons / logic between brain ancestry and brain ID

impl From<BrainRegionID> for BrainRegionAncestryID {
    fn from(value: BrainRegionID) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone)]
pub struct BrainRegionDesignerContainer {
    parent_id: BrainRegionID,
    ancestry_id: BrainRegionAncestryID,


    cache_containing_areas: HashSet<CorticalID>,
    cache_brain_regions: HashSet<BrainRegionID>
}

impl BrainRegionDesignerContainer {

    fn new_now(ancestor: Option<BrainRegionAncestryID>) -> Self {
        let new_id = BrainRegionID::new_now();
        let ancestor = if let Some(a) = ancestor {
            a
        } else {
            BrainRegionAncestryID::from(new_id.clone())
        };

        Self {
            parent_id: new_id,
            ancestry_id: ancestor,
            cache_containing_areas: Default::default(),
            cache_brain_regions: Default::default(),
        }
    }




}