/// Allows for communication of quantization levels at runtime. Numerical value represents
/// number of bytes used.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum QuantizationLevel {
    Bit8 = 1,
    Bit16 = 2,
    Bit32 = 4,
    Bit64 = 8, // NOTE: Always include this regardless of 64 bit feature flag, so we can have better error handling
}

impl QuantizationLevel {
    pub const fn minimum_quantization_needed_for_usize(u: usize) -> Self {
        const U8M: usize = u8::MAX as usize;
        const U16M: usize = u16::MAX as usize;
        const U32M: usize = u32::MAX as usize;

        if u < U8M {
            return QuantizationLevel::Bit8;
        }
        if u < U16M {
            return QuantizationLevel::Bit16;
        }
        if u < U32M {
            return QuantizationLevel::Bit32;
        }
        // Don't bother checking the difference between u64 and u128. If you get here you did
        // something wrong or are trying to summon more neurons than collectively exists in all
        // mankind and instantiating a such a super-intelligence is probably against Neuraville
        // TOS or something idk
        QuantizationLevel::Bit64
    }
}
