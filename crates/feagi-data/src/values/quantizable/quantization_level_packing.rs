/// Common trait for the enums describing the quantization level which establishes a standard
/// way to read / write the data from a byte. Enforces that the bits encoding this are on the X
/// first (right side) of a byte, with X being configurable on implementation. This allows the
/// encoding byte to also represent other data
pub trait QuantizationLevelPacking: Into<u8> + TryFrom<u8> + Clone + Copy + Sized {
    /// Defines the number of bits on the right to define for the bitmask
    const NUMBER_BITS: usize;
    const BASE_QUANT_MASK: u8 = 255 >> (8 - Self::NUMBER_BITS);

    /// Applies the mask to the given byte to get just the bits to convert to this enum
    fn apply_mask(byte: &mut u8) {
        *byte &= Self::BASE_QUANT_MASK;
    }

    /// Implementation should use "core::mem::transmute(byte)" to directly a (properly masked) byte
    /// to the enum. While fast, this function is unsafe in that if there is no corresponding bits,
    /// we will get undefined behavior!
    unsafe fn from_packed_byte(byte: u8) -> Self;
}
