use feagi_basis::feagi_basis_proc_common::feagi_id;
use feagi_basis::feagi_id::uuid::FeagiUuidID;
use crate::basis::specifications::brain_regions::BrainRegionID;

feagi_id! {
    /// The ID of a whole genome
    pub GenomeID: 16,
    uuid = v7,
    base64 = Base64EncodedGenomeID,
}

// TODO TEMP for current BV compat

impl From<GenomeID> for BrainRegionID {
    fn from(value: GenomeID) -> Self {
        BrainRegionID::try_from_uuid(value.as_uuid()).unwrap()
    }
}

// TODO TEMP for current BV compat

impl From<BrainRegionID> for GenomeID {
    fn from(value: BrainRegionID) -> Self {
        GenomeID::try_from_uuid(value.as_uuid()).unwrap()
    }
}