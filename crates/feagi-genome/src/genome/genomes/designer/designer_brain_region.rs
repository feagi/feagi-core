use crate::basis::cortical_area::cortical_id_legacy::CorticalIDLegacy;
use crate::basis::specifications::brain_regions::{BrainRegionAncestryID, BrainRegionID};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct BrainRegionDesignerContainer {
    parent_id: BrainRegionID,
    ancestry_id: BrainRegionAncestryID,


    cache_containing_areas: HashSet<CorticalIDLegacy>,
    cache_brain_regions: HashSet<BrainRegionID>
}

impl BrainRegionDesignerContainer {

    pub fn new_now(ancestor: Option<BrainRegionAncestryID>) -> Self {
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