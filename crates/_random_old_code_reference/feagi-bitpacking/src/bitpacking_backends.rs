
// TODO feature gate u64?

use core::hash::{Hash, Hasher};

macro_rules! __impl_bitpacked {
    ($type_name:ty, $integer_type:ty, $backend:expr, $number_bits:expr) => {
        impl BitPacked for $type_name {
            const BIT_PACKING_BACKEND: BitPackingBackend = $backend;
            type BoolArray = [bool; $number_bits];

            fn new() -> Self {Self(0)}

            fn get(&self) -> Self::BoolArray {
                let mut bits = [false; $number_bits];
                for bit_index in 0..$number_bits {
                    bits[bit_index] = ((self.0 >> bit_index) & 1) != 0;
                }
                bits
            }

            fn set(&mut self, bits: Self::BoolArray) {
                let mut packed_value: $integer_type = 0;
                for bit_index in 0..$number_bits {
                    if bits[bit_index] {
                        packed_value |= (1 as $integer_type) << bit_index;
                    }
                }
                self.0 = packed_value;
            }

            fn count_ones(&self) -> u8 {
                self.0.count_ones() as u8
            }

            fn flip_all(&mut self) {
                self.0 = !self.0;
            }

            fn get_at(&self, pos: u8) -> Option<bool> {
                if pos >= $number_bits {
                    return None;
                }

                Some(((self.0 >> pos) & 1) != 0)
            }

            fn set_at(&mut self, pos: u8, value: bool) -> Option<()> {
                if pos >= $number_bits {
                    return None;
                }

                let mask = (1 as $integer_type) << pos;
                if value {
                    self.0 |= mask;
                } else {
                    self.0 &= !mask;
                }

                Some(())
            }

            fn toggle_at(&mut self, pos: u8) -> Option<()> {
                if pos >= $number_bits {
                    return None;
                }

                self.0 ^= (1 as $integer_type) << pos;
                Some(())
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Clone for $type_name {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl Copy for $type_name {}

        impl Hash for $type_name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl PartialEq for $type_name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for $type_name {}
    };
}

#[repr(u8)]
#[derive(Debug, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub enum BitPackingBackend {
    U8 = (size_of::<u8>() * 8) as u8,
    U16 = (size_of::<u16>() * 8) as u8,
    U32 = (size_of::<u32>() * 8) as u8,
    U64 = (size_of::<u64>() * 8) as u8
}


#[repr(transparent)]
pub struct BitPackedU8(u8);


#[repr(transparent)]
pub struct BitPackedU16(u16);


#[repr(transparent)]
pub struct BitPackedU32(u32);


#[repr(transparent)]
pub struct BitPackedU64(u64);


pub trait BitPacked:
Sized
+ Copy
+ Default
+ Hash
+ PartialEq
{
    const BIT_PACKING_BACKEND: BitPackingBackend;
    const NUMBER_BITS: usize = Self::BIT_PACKING_BACKEND as usize;
    type BoolArray;

    fn new() -> Self;
    fn get(&self) -> Self::BoolArray;
    fn set(&mut self, bits: Self::BoolArray);

    fn flip_all(&mut self);
    fn count_ones(&self) -> u8;

    fn get_at(&self, pos: u8) -> Option<bool>;

    fn set_at(&mut self, pos: u8, value: bool) -> Option<()>;  // Returns some if valid, otherwise returns None
    
    fn toggle_at(&mut self, pos: u8) -> Option<()>;
}



__impl_bitpacked!(BitPackedU8, u8, BitPackingBackend::U8, 8);
__impl_bitpacked!(BitPackedU16, u16, BitPackingBackend::U16, 16);
__impl_bitpacked!(BitPackedU32, u32, BitPackingBackend::U32, 32);
__impl_bitpacked!(BitPackedU64, u64, BitPackingBackend::U64, 64);

