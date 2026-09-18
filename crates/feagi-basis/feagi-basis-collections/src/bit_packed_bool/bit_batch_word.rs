use serde::{Deserialize, Serialize};

/// The uint used as the packing for each BitBatchCollection Struct for holding the bits
pub trait BitBatchWord: Sized + Clone + Copy + core::fmt::Debug + core::hash::Hash
+ Send + Sync
+ Serialize + Deserialize<'static> + sealed::BitBatchWordSealing {
    /// How many bits can this uint contain?
    const NUMBER_BITS: u8 = (size_of::<Self>() as u8) * 8u8;

    /// How many bytes make up this uint?
    const NUMBER_BYTES: u8 = size_of::<Self>() as u8;

    /// Enum Representing this trait
    const BIT_BATCH_WORD_SIZE: BitBatchWordSize =
        BitBatchWordSize::get_size_from_byte_count(Self::NUMBER_BYTES);

    /// Amount to bitshift to get byte count
    const BIT_SHIFT_DISTANCE: u8 = 2u8 + Self::NUMBER_BYTES;

    /// A pass mask of all bits that are not representing the number of bits of this impl
    const UPPER_BYTE_COUNT_BIT_MASK: u8 = 255u8 << (Self::BIT_SHIFT_DISTANCE);
    /// A pass mask of all bits that are representing the number of bits of this impl
    const LOWER_BYTE_COUNT_BIT_MASK: u8 = !Self::UPPER_BYTE_COUNT_BIT_MASK;

    fn from_usize(u: usize) -> Self;

    fn as_usize(&self) -> usize;

    /// Count the number of 'true' bits in this word
    fn count_bits(&self) -> u32; //NOTE: This is due to how rust handles count_ones

    fn local_bit_index_to_mask(local_index: usize) -> Self {
        debug_assert!(local_index < Self::NUMBER_BITS as usize);
        Self::from_usize(1usize << local_index)
    }

    fn get_bit(&self, local_index: usize) -> bool {
        let mask = Self::local_bit_index_to_mask(local_index);
        self.as_usize() & mask.as_usize() > 0
    }

    fn set_bit(&mut self, local_index: usize, value: bool) {
        let mask = Self::local_bit_index_to_mask(local_index).as_usize();
        if value {
            *self = Self::from_usize(self.as_usize() | mask)
        } else {
            let mask = !mask;
            *self = Self::from_usize(self.as_usize() & mask)
        }
    }
}


impl sealed::BitBatchWordSealing for u8 {}
impl BitBatchWord for u8 {
    fn from_usize(u: usize) -> Self {
        u as u8
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn count_bits(&self) -> u32 {
        self.count_ones()
    }
}
impl sealed::BitBatchWordSealing for u16 {}
impl BitBatchWord for u16 {
    fn from_usize(u: usize) -> Self {
        u as u16
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn count_bits(&self) -> u32 {
        self.count_ones()
    }
}
impl sealed::BitBatchWordSealing for u32 {}
impl BitBatchWord for u32 {
    fn from_usize(u: usize) -> Self {
        u as u32
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn count_bits(&self) -> u32 {
        self.count_ones()
    }
}
impl sealed::BitBatchWordSealing for u64 {}
impl BitBatchWord for u64 {
    fn from_usize(u: usize) -> Self {
        u as u64
    }

    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn count_bits(&self) -> u32 {
        self.count_ones()
    }
}


/// Defines the number of bits per word, and is used to determine byte padding distance
#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BitBatchWordSize {
    U8 = 8,
    U16 = 16,
    U32 = 32,
    U64 = 64,
    // Not doing 128 lol
}

impl BitBatchWordSize {
    pub const fn get_size_from_byte_count(byte_count: u8) -> Self {
        match byte_count {
            1 => BitBatchWordSize::U8,
            2 => BitBatchWordSize::U16,
            3 => BitBatchWordSize::U32,
            4 => BitBatchWordSize::U64,
            _ => panic!("Invalid byte count!")
        }
    }

    pub fn get_as_number_bytes(&self) -> u8 {
        match self {
            BitBatchWordSize::U8 => 1,
            BitBatchWordSize::U16 => 2,
            BitBatchWordSize::U32 => 3,
            BitBatchWordSize::U64 => 4,
        }
    }
}



mod sealed {
    /// Marker trait to prevent impl `BitBatchWord` to other structs
    pub trait BitBatchWordSealing {}
}