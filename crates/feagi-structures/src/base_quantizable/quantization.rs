

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum QuantizationLevel
{
    Bit8,
    Bit16,
    Bit32,
    Bit64, // NOTE: Always include this regardless of 64 bit feature flag, so we can have better error handling
}