use core::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a brain region, based on UUID v7.
///
/// This struct provides type safety and ensures global uniqueness for brain region IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionID {
    uuid: Uuid,
}

impl RegionID {

    #[cfg(feature = "std")]
    pub fn new_current_timestamp() -> Self {
        Self { uuid: Uuid::now_v7() }
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {uuid: Uuid::from_bytes(bytes)}
    }

}

impl Display for RegionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid.to_string())
    }
}