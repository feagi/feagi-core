use base64::prelude::*

pub const CORTICAL_ID_BYTE_COUNT: usize = core::mem::size_of::<u64>(); // 8 bytes -> 64 bit
pub const CORTICAL_ID_BASE_64_BYTE_COUNT: usize = 4 * (CORTICAL_ID_BYTE_COUNT + 3); // enforces rounding up


/// A unique identifier for a cortical area in a genome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CorticalID(u64);

impl CorticalID {
    
    
    
}


impl From<u64> for CorticalID {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl Into<u64> for CorticalID {
    fn into(self) -> u64 {
        self.0
    }
}

impl

