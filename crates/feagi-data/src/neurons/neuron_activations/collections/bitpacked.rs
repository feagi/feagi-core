use core::ops::{Index, IndexMut, Range};

use crate::neurons::neuron_activations::neuron_activation_error::{
    FeagiNeuronActivationError, FeagiNeuronActivationInvalidRange,
};
use crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait;

macro_rules! impl_bitpacked_range_read {
    ($self_ty:ty, $qi:ty, [$($generics:tt)*]) => {
        impl<$($generics)*> Index<Range<$qi>> for $self_ty {
            type Output = [u32];
            fn index(&self, range: Range<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeInclusive<$qi>> for $self_ty {
            type Output = [u32];
            fn index(&self, range: core::ops::RangeInclusive<$qi>) -> &Self::Output {
                &self.data[range.start().quant_to_usize()..=range.end().quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFrom<$qi>> for $self_ty {
            type Output = [u32];
            fn index(&self, range: core::ops::RangeFrom<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeTo<$qi>> for $self_ty {
            type Output = [u32];
            fn index(&self, range: core::ops::RangeTo<$qi>) -> &Self::Output {
                &self.data[..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFull> for $self_ty {
            type Output = [u32];
            fn index(&self, _range: core::ops::RangeFull) -> &Self::Output {
                &self.data[..]
            }
        }
    };
}

macro_rules! impl_bitpacked_range_read_write {
    ($self_ty:ty, $qi:ty, [$($generics:tt)*]) => {
        impl_bitpacked_range_read!($self_ty, $qi, [$($generics)*]);

        impl<$($generics)*> IndexMut<Range<$qi>> for $self_ty {
            fn index_mut(&mut self, range: Range<$qi>) -> &mut Self::Output {
                &mut self.data[range.start.quant_to_usize()..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> IndexMut<core::ops::RangeInclusive<$qi>> for $self_ty {
            fn index_mut(&mut self, range: core::ops::RangeInclusive<$qi>) -> &mut Self::Output {
                &mut self.data[range.start().quant_to_usize()..=range.end().quant_to_usize()]
            }
        }

        impl<$($generics)*> IndexMut<core::ops::RangeFrom<$qi>> for $self_ty {
            fn index_mut(&mut self, range: core::ops::RangeFrom<$qi>) -> &mut Self::Output {
                &mut self.data[range.start.quant_to_usize()..]
            }
        }

        impl<$($generics)*> IndexMut<core::ops::RangeTo<$qi>> for $self_ty {
            fn index_mut(&mut self, range: core::ops::RangeTo<$qi>) -> &mut Self::Output {
                &mut self.data[..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> IndexMut<core::ops::RangeFull> for $self_ty {
            fn index_mut(&mut self, _range: core::ops::RangeFull) -> &mut Self::Output {
                &mut self.data[..]
            }
        }
    };
}

/// Minimum byte alignment passed to [`BitPacked::u32_count_for_bits`] when no
/// larger alignment is required (equivalent to rounding up to whole u32 words).
const MIN_U32_BYTE_ALIGNMENT: usize = 4;

/// Defines a u32 bit-packed structure that you can read true/false from.
pub trait BitPacked<QI: QuantizedUnsignedIntegerUnwrappedTrait>:
    Index<QI, Output = u32> + Index<Range<QI>, Output = [u32]>
{
    /// Borrows the backing storage as a regular shared u32 slice.
    fn as_u32s(&self) -> &[u32];

    //region Default impls

    /// Calculates the number of u32 words needed to hold a given number of bits
    /// while also ensuring a byte alignment (32, 64, 128, etc.).
    fn u32_count_for_bits(number_elements: QI, byte_alignment: usize) -> usize {
        assert!(byte_alignment > 0 && byte_alignment % 4 == 0);

        let u32s_per_alignment = byte_alignment / 4;
        number_elements
            .quant_to_usize()
            .div_ceil(32)
            .div_ceil(u32s_per_alignment)
            * u32s_per_alignment
    }

    /// Number of u32 words backing this collection.
    fn number_u32s(&self) -> QI {
        QI::quant_from_usize_unchecked(self.as_u32s().len())
    }

    /// Copies out the bit at `index`, without checking if the value is in padding.
    fn get_bit_unchecked(&self, bit_index: QI) -> bool {
        let bit = bit_index.quant_to_usize();
        let word = self.as_u32s()[bit >> 5];
        (word >> (bit & 0x1F)) & 1 == 1
    }

    /// Copies out the whole u32 at `index`, or `None` if out of bounds.
    fn get_u32(&self, u32_index: QI) -> Option<u32> {
        self.as_u32s().get(u32_index.quant_to_usize()).copied()
    }

    /// Iterates over shared references to the u32 words.
    fn iter_u32s(&self) -> core::slice::Iter<'_, u32> {
        self.as_u32s().iter()
    }

    /// Given a u32 word index, gets the index of the first bit of that word.
    fn get_first_bit_index_from_u32_unchecked(&self, u32_index: QI) -> QI {
        QI::quant_from_usize_unchecked(u32_index.quant_to_usize() << 5)
    }

    /// Raw pointer to the first u32 word. Valid for reads of [`Self::number_u32s`]
    /// words for as long as `self` is borrowed.
    fn as_u32_ptr(&self) -> *const u32 {
        self.as_u32s().as_ptr()
    }

    /// Returns a shared reference to the u32 word at `index`, without bounds
    /// checking.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.number_u32s()`).
    unsafe fn get_u32_par(&self, index: QI) -> &u32 {
        &*self.as_u32_ptr().add(index.quant_to_usize())
    }

    /// Rayon parallel iterator over shared references to the u32 words.
    #[cfg(feature = "use-rayon")]
    fn par_iter_u32s(&self) -> rayon::slice::Iter<'_, u32> {
        use rayon::iter::IntoParallelRefIterator;
        self.as_u32s().par_iter()
    }

    //endregion
}

/// Behaviour for bit-packed collections that track how many bits are logically
/// in use (as opposed to padding in the final u32 word).
pub trait BitPackedAwareSize<QI: QuantizedUnsignedIntegerUnwrappedTrait>: BitPacked<QI> {
    /// Total number of addressable bits (booleans) held by this collection. Note
    /// that some bits may not be accessible (dangling).
    fn number_addressable_bits(&self) -> QI;

    //region Default impls

    /// Number of unused ("dangling") bits in the final u32 word, i.e. the bits of
    /// the backing storage beyond [`Self::number_addressable_bits`].
    fn number_dangling_bits(&self) -> u32 {
        let capacity = self.as_u32s().len() * 32;
        (capacity - self.number_addressable_bits().quant_to_usize()) as u32
    }

    /// Returns `true` if there are no addressable bits.
    fn is_empty(&self) -> bool {
        self.number_addressable_bits() == QI::QUANT_ZERO
    }

    /// Copies out the bit at `index`, or `None` if out of bounds / in padding.
    fn get_bit(&self, bit_index: QI) -> Option<bool> {
        if bit_index >= self.number_addressable_bits() {
            return None;
        }
        Some(self.get_bit_unchecked(bit_index))
    }

    /// Borrows the whole collection as a [`BitPackedSlice`] view.
    fn as_bit_packed_slice(&self) -> BitPackedSlice<'_, QI> {
        BitPackedSlice::new(self.as_u32s(), self.number_addressable_bits())
    }

    /// Borrows a half-open *u32-word* sub-range as a [`BitPackedSlice`] view. The
    /// resulting view treats every u32 as full (its bit count is `u32s * 32`),
    /// so any dangling bits of the original collection are not carried over.
    ///
    /// Returns [`FeagiNeuronActivationInvalidRange`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice(&self, range: Range<QI>) -> Result<BitPackedSlice<'_, QI>, FeagiNeuronActivationError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.as_u32s().get(start..end) {
            Some(slice) => {
                let bits = QI::quant_from_usize_unchecked(slice.len() * 32);
                Ok(BitPackedSlice::new(slice, bits))
            }
            None => Err(
                FeagiNeuronActivationInvalidRange::new(
                    "subslice u32 range is out of bounds",
                    start,
                    end,
                )
                .into(),
            ),
        }
    }

    /// Copies the internal u32 words and total length to a new owned vector structure.
    fn clone_to_owned(&self) -> BitPackedVector<QI> {
        BitPackedVector::from_vec_with_bits(self.as_u32s().to_vec(), self.number_addressable_bits())
    }

    /// If the bit-packed array is holding data whose length is not divisible by 32,
    /// eventually the last u32 word will cover a range less than 32. This function
    /// will return less than 32 if that's the case.
    fn get_number_bits_remaining_from_u32_unchecked(&self, u32_index: QI) -> u32 {
        let first_index = self.get_first_bit_index_from_u32_unchecked(u32_index);
        let bits_remaining = (self.number_addressable_bits() - first_index).quant_to_usize();
        bits_remaining.min(32) as u32
    }

    //endregion
}

/// Shared, mutable behaviour for u32 bit-packed collections that own or
/// exclusively borrow their storage.
///
/// # Safety (implementors)
/// The unsafe functions allow for multiple element mut access at once. Required for parallel processing,
/// but can easily put you in a fucked state if you arent careful!
pub trait BitPackedMut<QI: QuantizedUnsignedIntegerUnwrappedTrait>:
    BitPacked<QI> + IndexMut<QI, Output = u32> + IndexMut<Range<QI>, Output = [u32]>
{
    /// Mutably borrows the backing storage as a regular u32 slice.
    fn as_mut_u32s(&mut self) -> &mut [u32];

    //region Default impls

    /// Mutably borrows the whole u32 at `index`, or `None` if out of bounds.
    fn get_u32_mut(&mut self, index: QI) -> Option<&mut u32> {
        self.as_mut_u32s().get_mut(index.quant_to_usize())
    }

    /// Overwrites the whole u32 at `index`, returning the previous value if the
    /// index was in bounds (otherwise leaves the collection untouched).
    fn set_u32(&mut self, index: QI, value: u32) -> Option<u32> {
        match self.as_mut_u32s().get_mut(index.quant_to_usize()) {
            Some(slot) => {
                let previous = *slot;
                *slot = value;
                Some(previous)
            }
            None => None,
        }
    }

    /// Iterates over mutable references to the u32 words.
    fn iter_u32s_mut(&mut self) -> core::slice::IterMut<'_, u32> {
        self.as_mut_u32s().iter_mut()
    }

    /// Rayon parallel iterator over mutable references to the u32 words.
    #[cfg(feature = "use-rayon")]
    fn par_iter_u32s_mut(&mut self) -> rayon::slice::IterMut<'_, u32> {
        use rayon::iter::IntoParallelRefMutIterator;
        self.as_mut_u32s().par_iter_mut()
    }

    /// Raw mutable pointer to the first u32 word, derived from a shared `&self`.
    ///
    /// # Safety
    /// The returned pointer aliases the collection's storage; writes through it
    /// must only target u32 indices not simultaneously accessed elsewhere.
    unsafe fn as_mut_u32_ptr_par(&self) -> *mut u32 {
        self.as_u32_ptr() as *mut u32
    }

    /// Returns a mutable reference to the u32 word at `index` through a shared
    /// `&self`, enabling parallel mutation of disjoint u32 words.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.number_u32s()`).
    /// - No other reference (shared or mutable) to the *same* u32 word may exist
    ///   for the duration of the returned borrow. Concurrent callers must only ever
    ///   target disjoint u32 indices.
    unsafe fn get_u32_mut_par(&self, index: QI) -> &mut u32 {
        &mut *self.as_mut_u32_ptr_par().add(index.quant_to_usize())
    }

    //endregion
}

/// Mutable behaviour for bit-packed collections that track how many bits are
/// logically in use (as opposed to padding in the final u32 word).
pub trait BitPackedAwareSizeMut<QI: QuantizedUnsignedIntegerUnwrappedTrait>:
    BitPackedMut<QI> + BitPackedAwareSize<QI>
{
    //region Default impls

    /// Sets the bit at `index`, returning the previous value if the index was in
    /// bounds (otherwise leaves the collection untouched).
    fn set_bit(&mut self, index: QI, value: bool) -> Option<bool> {
        let bit = index.quant_to_usize();
        if bit >= self.number_addressable_bits().quant_to_usize() {
            return None;
        }
        let word = &mut self.as_mut_u32s()[bit >> 5];
        let previous = (*word >> (bit & 0x1F)) & 1 == 1;
        if value {
            *word |= 1 << (bit & 0x1F);
        } else {
            *word &= !(1 << (bit & 0x1F));
        }
        Some(previous)
    }

    /// Mutably borrows a half-open *u32-word* sub-range as a [`BitPackedSliceMut`]
    /// view. The resulting view treats every u32 as full (its bit count is
    /// `u32s * 32`).
    ///
    /// Returns [`FeagiNeuronActivationInvalidRange`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice_u32s_mut(
        &mut self,
        range: Range<QI>,
    ) -> Result<BitPackedSliceMut<'_, QI>, FeagiNeuronActivationError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.as_mut_u32s().get_mut(start..end) {
            Some(slice) => {
                let bits = QI::quant_from_usize_unchecked(slice.len() * 32);
                Ok(BitPackedSliceMut::new(slice, bits))
            }
            None => Err(
                FeagiNeuronActivationInvalidRange::new(
                    "subslice u32 range is out of bounds",
                    start,
                    end,
                )
                .into(),
            ),
        }
    }

    //endregion
}

//region Vector

/// An owned, heap-allocated run of u32 bit-packed neuron activation booleans.
pub struct BitPackedVector<QI: QuantizedUnsignedIntegerUnwrappedTrait> {
    pub(crate) data: Vec<u32>,
    pub(crate) number_bits: QI,
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> Clone for BitPackedVector<QI> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            number_bits: self.number_bits,
        }
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedVector<QI> {
    /// Builds a vector holding `number_bits` booleans, every one initialised to
    /// `initial_state`. Any dangling bits in the final u32 word are kept zeroed.
    pub fn new_uniform(number_bits: QI, initial_state: bool) -> BitPackedVector<QI> {
        let number_u32s = QI::quant_from_usize_unchecked(<Self as BitPacked<QI>>::u32_count_for_bits(
            number_bits,
            MIN_U32_BYTE_ALIGNMENT,
        ));
        let data: Vec<u32> = if initial_state {
            vec![0xFFFF_FFFF; number_u32s.quant_to_usize()]
        } else {
            vec![0; number_u32s.quant_to_usize()]
        };

        Self { data, number_bits }
    }

    /// Wraps an existing `Vec` without copying, treating every u32 as full
    /// (bit count is `data.len() * 32`, no dangling bits).
    pub fn from_vec(data: Vec<u32>) -> BitPackedVector<QI> {
        let number_bits = QI::quant_from_usize_unchecked(data.len() * 32);
        Self { data, number_bits }
    }

    /// Wraps an existing `Vec` without copying, using an explicit bit count.
    /// `number_bits` must not exceed `data.len() * 32`.
    pub fn from_vec_with_bits(data: Vec<u32>, number_bits: QI) -> BitPackedVector<QI> {
        Self { data, number_bits }
    }

    /// Consumes the wrapper, returning the backing `Vec`.
    pub fn into_vec(self) -> Vec<u32> {
        self.data
    }

    /// Appends `number_bits` new bits to the end of the vector, initializing all
    /// appended bits to `initial_state`.
    pub fn append_bits(&mut self, number_bits: QI, initial_state: bool) {
        let additional_bits = number_bits.quant_to_usize();
        if additional_bits == 0 {
            return;
        }

        let previous_bits = self.number_bits.quant_to_usize();
        let new_total_bits = previous_bits + additional_bits;
        let needed_u32s = <Self as BitPacked<QI>>::u32_count_for_bits(
            QI::quant_from_usize_unchecked(new_total_bits),
            MIN_U32_BYTE_ALIGNMENT,
        );

        if needed_u32s > self.data.len() {
            self.data.resize(needed_u32s, 0);
        }

        if initial_state {
            for bit_index in previous_bits..new_total_bits {
                let u32_index = bit_index >> 5;
                let bit_offset = bit_index & 0x1F;
                self.data[u32_index] |= 1 << bit_offset;
            }
        }

        self.number_bits = QI::quant_from_usize_unchecked(new_total_bits);
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPacked<QI> for BitPackedVector<QI> {
    fn as_u32s(&self) -> &[u32] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedAwareSize<QI> for BitPackedVector<QI> {
    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedMut<QI> for BitPackedVector<QI> {
    fn as_mut_u32s(&mut self) -> &mut [u32] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedAwareSizeMut<QI> for BitPackedVector<QI> {}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> Index<QI> for BitPackedVector<QI> {
    type Output = u32;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> IndexMut<QI> for BitPackedVector<QI> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read_write!(
    BitPackedVector<QI>,
    QI,
    [QI: QuantizedUnsignedIntegerUnwrappedTrait]
);

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> From<Vec<u32>> for BitPackedVector<QI> {
    fn from(value: Vec<u32>) -> Self {
        Self::from_vec(value)
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> From<BitPackedVector<QI>> for Vec<u32> {
    fn from(value: BitPackedVector<QI>) -> Self {
        value.data
    }
}

//endregion

//region Slice

/// A borrowed, read-only view over a run of u32 bit-packed neuron activation booleans.
#[derive(Clone, Copy)]
pub struct BitPackedSlice<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> {
    pub(crate) data: &'a [u32],
    pub(crate) number_bits: QI,
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedSlice<'a, QI> {
    /// Wraps an existing shared u32 slice with an explicit bit count.
    /// `number_bits` must not exceed `data.len() * 32`.
    pub fn new(data: &'a [u32], number_bits: QI) -> BitPackedSlice<'a, QI> {
        Self { data, number_bits }
    }

    /// Wraps an existing shared u32 slice, treating every u32 as full
    /// (bit count is `data.len() * 32`, no dangling bits).
    pub fn from_u32s(data: &'a [u32]) -> BitPackedSlice<'a, QI> {
        let number_bits = QI::quant_from_usize_unchecked(data.len() * 32);
        Self { data, number_bits }
    }

    /// Returns the underlying shared u32 slice, keeping the original lifetime.
    pub fn into_u32s(self) -> &'a [u32] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPacked<QI> for BitPackedSlice<'a, QI> {
    fn as_u32s(&self) -> &[u32] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedAwareSize<QI> for BitPackedSlice<'a, QI> {
    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> Index<QI> for BitPackedSlice<'a, QI> {
    type Output = u32;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read!(
    BitPackedSlice<'a, QI>,
    QI,
    ['a, QI: QuantizedUnsignedIntegerUnwrappedTrait]
);

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> From<&'a [u32]> for BitPackedSlice<'a, QI> {
    fn from(value: &'a [u32]) -> Self {
        Self::from_u32s(value)
    }
}

//endregion

//region Mut Slice

/// A borrowed, mutable view over a run of u32 bit-packed neuron activation booleans.
pub struct BitPackedSliceMut<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> {
    pub(crate) data: &'a mut [u32],
    pub(crate) number_bits: QI,
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedSliceMut<'a, QI> {
    /// Wraps an existing mutable u32 slice with an explicit bit count.
    /// `number_bits` must not exceed `data.len() * 32`.
    pub fn new(data: &'a mut [u32], number_bits: QI) -> BitPackedSliceMut<'a, QI> {
        Self { data, number_bits }
    }

    /// Wraps an existing mutable u32 slice, treating every u32 as full
    /// (bit count is `data.len() * 32`, no dangling bits).
    pub fn from_u32s(data: &'a mut [u32]) -> BitPackedSliceMut<'a, QI> {
        let number_bits = QI::quant_from_usize_unchecked(data.len() * 32);
        Self { data, number_bits }
    }

    /// Returns the underlying mutable u32 slice, keeping the original lifetime.
    pub fn into_u32s_mut(self) -> &'a mut [u32] {
        self.data
    }

    /// Creates a shorter-lived, exclusive re-borrow of this view.
    pub fn reborrow(&mut self) -> BitPackedSliceMut<'_, QI> {
        BitPackedSliceMut::new(self.data, self.number_bits)
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPacked<QI> for BitPackedSliceMut<'a, QI> {
    fn as_u32s(&self) -> &[u32] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedAwareSize<QI> for BitPackedSliceMut<'a, QI> {
    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedMut<QI> for BitPackedSliceMut<'a, QI> {
    fn as_mut_u32s(&mut self) -> &mut [u32] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> BitPackedAwareSizeMut<QI> for BitPackedSliceMut<'a, QI> {}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> Index<QI> for BitPackedSliceMut<'a, QI> {
    type Output = u32;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> IndexMut<QI> for BitPackedSliceMut<'a, QI> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read_write!(
    BitPackedSliceMut<'a, QI>,
    QI,
    ['a, QI: QuantizedUnsignedIntegerUnwrappedTrait]
);

impl<'a, QI: QuantizedUnsignedIntegerUnwrappedTrait> From<&'a mut [u32]> for BitPackedSliceMut<'a, QI> {
    fn from(value: &'a mut [u32]) -> Self {
        Self::from_u32s(value)
    }
}

//endregion

//region Array

/// An owned, stack-allocated run of u32 bit-packed neuron activation booleans
/// backed by exactly `N` u32 words.
///
/// The compile-time length `N` is the *u32-word* count as a `usize` const generic
/// (Rust const generics must be an integer type, so `QI` is retained as the
/// associated index/count type used by the shared trait methods). The logical
/// bit count may be smaller than `N * 32` when there are dangling bits.
#[derive(Clone, Copy)]
pub struct BitPackedArray<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> {
    pub(crate) data: [u32; N],
    pub(crate) number_bits: QI,
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> BitPackedArray<QI, N> {
    /// Builds an array holding `number_bits` booleans, every one initialised to
    /// `initial_state`. `number_bits` must not exceed `N * 32`. Any dangling bits
    /// in the final u32 word are kept zeroed.
    pub fn new_uniform(number_bits: QI, initial_state: bool) -> BitPackedArray<QI, N> {
        let data: [u32; N] = if initial_state {
            [0xFFFF_FFFF; N]
        } else {
            [0; N]
        };

        Self { data, number_bits }
    }

    /// Wraps an existing array, treating every u32 as full (bit count is
    /// `N * 32`, no dangling bits).
    pub fn from_array(data: [u32; N]) -> BitPackedArray<QI, N> {
        let number_bits = QI::quant_from_usize_unchecked(N * 32);
        Self { data, number_bits }
    }

    /// Wraps an existing array with an explicit bit count. `number_bits` must not
    /// exceed `N * 32`.
    pub fn from_array_with_bits(data: [u32; N], number_bits: QI) -> BitPackedArray<QI, N> {
        Self { data, number_bits }
    }

    /// Consumes the wrapper, returning the backing array.
    pub fn into_array(self) -> [u32; N] {
        self.data
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> BitPacked<QI> for BitPackedArray<QI, N> {
    fn as_u32s(&self) -> &[u32] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> BitPackedAwareSize<QI>
    for BitPackedArray<QI, N>
{
    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> BitPackedMut<QI> for BitPackedArray<QI, N> {
    fn as_mut_u32s(&mut self) -> &mut [u32] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> BitPackedAwareSizeMut<QI>
    for BitPackedArray<QI, N>
{
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> Index<QI> for BitPackedArray<QI, N> {
    type Output = u32;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> IndexMut<QI> for BitPackedArray<QI, N> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read_write!(
    BitPackedArray<QI, N>,
    QI,
    [QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize]
);

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const N: usize> From<[u32; N]> for BitPackedArray<QI, N> {
    fn from(value: [u32; N]) -> Self {
        Self::from_array(value)
    }
}

//endregion
