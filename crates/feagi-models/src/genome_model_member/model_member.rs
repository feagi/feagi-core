

pub struct ModelDecimalMember<const DEFAULT_BITS: u64, const MIN_BITS: u64, const MAX_BITS: u64, const MEMBER_INDEX: u8> {}

impl<const DEFAULT_BITS: u64, const MIN_BITS: u64, const MAX_BITS: u64, const MEMBER_INDEX: u8>
    ModelDecimalMember<DEFAULT_BITS, MIN_BITS, MAX_BITS, MEMBER_INDEX>
{
    pub const DEFAULT: f64 = f64::from_bits(DEFAULT_BITS); // WTF RUST TOOLCHAIN
    pub const MIN: f64 = f64::from_bits(MIN_BITS);
    pub const MAX: f64 = f64::from_bits(MAX_BITS);
}
