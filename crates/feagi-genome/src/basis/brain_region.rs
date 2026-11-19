use std::hash::Hash;
use serde::{Deserialize, Serialize};
use feagi_basis::feagi_id::feagi_id;

pub trait BrainRegionIDType: Clone + Copy + core::hash::Hash + core::fmt::Debug
+ ::serde::Serialize + ::serde::de::DeserializeOwned
{}

feagi_id! {
    /// The ID of a region. This is created fresh with the current timestamp whenever a Brain Region
    /// is created, regardless of its new or if its from an almagamation
    pub BrainRegionID: 16,
    uuid = v7,
    base64 = Base64EncodedBrainRegionID,
}

impl BrainRegionID {

    /// Casts this as a Faux Brain Region ID
    pub fn into_faux_brain_region_id(self) -> FauxBrainRegionID {
        FauxBrainRegionID(self.0)
    }

    #[cfg(feature = "std")]
    /// Rerolls the ID, should effectively never be needed
    pub fn reroll_id(mut self) -> Self {
        self = Self::new_now();
        self
    }
}

impl Into<MaybeBrainRegionID> for BrainRegionID {
    fn into(self) -> MaybeBrainRegionID {
        MaybeBrainRegionID::Genuine(self)
    }
}

impl BrainRegionIDType for BrainRegionID {}



feagi_id! {
    /// A standin Brain Region ID, primarily used during edit requests, that avoids odds of collisions,
    /// but allows in edit requests genome objects to have a parent
    pub FauxBrainRegionID: 16,
    uuid = v7,
    base64 = Base64EncodedFauxBrainRegionID,
}

impl FauxBrainRegionID {
    pub(crate) fn into_brain_region_id(self) -> BrainRegionID {
        BrainRegionID(self.0)
    }
}

impl Into<MaybeBrainRegionID> for FauxBrainRegionID {
    fn into(self) -> MaybeBrainRegionID {
        MaybeBrainRegionID::Faux(self)
    }
}

impl BrainRegionIDType for FauxBrainRegionID {}


/// Allows specifying either a concrete (existing genuine) brain region or a faux (in transit) one
#[derive(Clone, Copy, Hash, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub enum MaybeBrainRegionID {
    Genuine(BrainRegionID),
    Faux(FauxBrainRegionID)
}

impl BrainRegionIDType for MaybeBrainRegionID {}


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

