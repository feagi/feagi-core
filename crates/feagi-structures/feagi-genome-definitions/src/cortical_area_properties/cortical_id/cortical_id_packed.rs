



/// Represents a cortical area identifier's 8 bytes directly as such, in big endian order
pub struct CorticalIDPacked([u8; Self::BYTE_COUNT]);


impl CorticalIDPacked {
    /// The number of bytes that are needed to represent a Cortical ID
    pub const BYTE_COUNT: usize = core::mem::size_of::<u64>();
    
    /// Directly creates an instance of this type without any checks, const compatible
    pub(crate) const fn new_const_unchecked(bytes: [u8; Self::BYTE_COUNT]) -> CorticalIDPacked {
        CorticalIDPacked(bytes)
    }
    
    /// Converts one of this type to an u64 int, which is easy to store
    pub fn to_u64(self) -> u64 {
        u64::from_be_bytes(self.0)
    }
}