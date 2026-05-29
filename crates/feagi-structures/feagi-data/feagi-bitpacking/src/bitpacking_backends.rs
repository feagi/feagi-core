
// TODO feature gate u64?

#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub enum BitPackingBackend {
    U8 = 8,
    U16 = 16,
    U32 = 32,
    U64 = 64
}


#[repr(transparent)]
pub struct BitPackedU8(u8);

#[repr(transparent)]
pub struct BitPackedU16(u16);

#[repr(transparent)]
pub struct BitPackedU32(u32);

#[repr(transparent)]
pub struct BitPackedU64(u64);


pub trait BitPacked: Sized {
    const NUMBER_BITS: usize = size_of::<Self>() * 8;
}

impl BitPacked for BitPackedU8 {}

impl BitPacked for BitPackedU16 {}

impl BitPacked for BitPackedU32 {}

impl BitPacked for BitPackedU64 {}






