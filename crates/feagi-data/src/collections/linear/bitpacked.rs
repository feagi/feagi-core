use crate::collections::feagi_data_collections_error::{FeagiDataCollectionError, FeagiFailCollectionInvalidIndex};
use crate::values::quantizable::QuantizedIndexCountTrait;
use core::ops::{Index, IndexMut, Range};

macro_rules! impl_bitpacked_range_read {
    ($self_ty:ty, $qi:ty, [$($generics:tt)*]) => {
        impl<$($generics)*> Index<Range<$qi>> for $self_ty {
            type Output = [u8];
            fn index(&self, range: Range<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeInclusive<$qi>> for $self_ty {
            type Output = [u8];
            fn index(&self, range: core::ops::RangeInclusive<$qi>) -> &Self::Output {
                &self.data[range.start().quant_to_usize()..=range.end().quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFrom<$qi>> for $self_ty {
            type Output = [u8];
            fn index(&self, range: core::ops::RangeFrom<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeTo<$qi>> for $self_ty {
            type Output = [u8];
            fn index(&self, range: core::ops::RangeTo<$qi>) -> &Self::Output {
                &self.data[..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFull> for $self_ty {
            type Output = [u8];
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

/// Returns the number of whole bytes required to hold `self` bits, i.e. the
/// ceiling division of the bit count by 8. `0` bits require `0` bytes.
fn number_bits_to_number_bytes(n: usize) -> usize {
    if n % 8 == 0 {
        n / 8
    } else {
        (n / 8) + 1
    }
}

pub trait BitPackedTrait<QI: QuantizedIndexCountTrait>: Index<QI, Output = u8> + Index<Range<QI>, Output = [u8]> {
    /// Borrows the backing storage as a regular shared byte slice.
    fn as_bytes(&self) -> &[u8];

    /// Total number of addressable bits (booleans) held by this collection. Note that some bits
    /// may not be accessible (dangling)
    fn number_addressable_bits(&self) -> QI;

    /// Number of bytes backing this collection.
    fn number_bytes(&self) -> QI {
        QI::quant_from_usize(self.as_bytes().len())
    }

    /// Number of unused ("dangling") bits in the final byte, i.e. the bits of
    /// the backing storage beyond [`Self::number_addressable_bits`].
    fn number_dangling_bits(&self) -> u8 {
        let capacity = self.as_bytes().len() * 8;
        (capacity - self.number_addressable_bits().quant_to_usize()) as u8
    }

    /// Returns `true` if there are no bits.
    fn is_empty(&self) -> bool {
        self.number_addressable_bits() == QI::QUANT_ZERO
    }

    /// Copies out the bit at `index`, or `None` if out of bounds.
    fn get_bit(&self, bit_index: QI) -> Option<bool> {
        let bit = bit_index.quant_to_usize();
        if bit >= self.number_addressable_bits().quant_to_usize() {
            return None;
        }
        // ' >> 3' is the same as ' / 8'
        let byte = self.as_bytes()[bit >> 3];
        // '& 0b00000111' is the same as  '% 8'
        Some((byte >> (bit & 0b00000111)) & 1 == 1)
    }

    /// Copies out the whole byte at `index`, or `None` if out of bounds.
    fn get_byte(&self, bool_index: QI) -> Option<u8> {
        self.as_bytes().get(bool_index.quant_to_usize()).copied()
    }

    /// Borrows the whole collection as a [`BitPackedSlice`] view.
    fn as_bit_packed_slice(&self) -> BitPackedSlice<'_, QI> {
        BitPackedSlice::new(self.as_bytes(), self.number_addressable_bits())
    }

    /// Borrows a half-open *byte* sub-range as a [`BitPackedSlice`] view. The
    /// resulting view treats every byte as full (its bit count is `bytes * 8`),
    /// so any dangling bits of the original collection are not carried over.
    ///
    /// Returns [`FeagiFailCollectionInvalidIndex`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice(&self, range: Range<QI>) -> Result<BitPackedSlice<'_, QI>, FeagiDataCollectionError> {
        match self.as_bytes().get(range.start.quant_to_usize()..range.end.quant_to_usize()) {
            Some(slice) => {
                let bits = QI::quant_from_usize(slice.len() * 8);
                Ok(BitPackedSlice::new(slice, bits))
            }
            None => Err(FeagiFailCollectionInvalidIndex::new("subslice byte range is out of bounds").into()),
        }
    }

    /// Copies the internal bytes and total length to a new owned vector structure
    fn clone_to_owned(&self) -> BitPackedVector<QI> {
        BitPackedVector::from_vec_with_bits(self.as_bytes().to_vec(), self.number_addressable_bits())
    }

    /// Iterates over shared references to the bytes.
    fn iter_bytes(&self) -> core::slice::Iter<'_, u8> {
        self.as_bytes().iter()
    }

    /// Given a byte index, gets the index of the first bit of that byte
    fn get_first_bit_index_from_byte_unchecked(&self, byte_index: QI) -> QI {
        QI::quant_from_usize((byte_index.quant_to_usize()) << 3)
    }

    /// If the bit packed array is holding data of length not divisible by 8, eventually the last
    /// byte will cover a range less than 8. This function will return less than 8 if thats the case
    fn get_number_bits_remaining_from_byte_unchecked(&self, byte_index: QI) -> u8 {
        let first_index = self.get_first_bit_index_from_byte_unchecked(byte_index);
        let bits_remaining = (self.number_addressable_bits() - first_index).quant_to_usize();
        bits_remaining.min(8) as u8
    }

    /// Rayon parallel iterator over shared references to the bytes.
    #[cfg(feature = "use-rayon")]
    fn par_iter_bytes(&self) -> rayon::slice::Iter<'_, u8> {
        use rayon::iter::IntoParallelRefIterator;
        self.as_bytes().par_iter()
    }
}

/// Shared, mutable behaviour for the bit-packed quantized collections that own
/// or exclusively borrow their storage (the [`BitPackedVector`],
/// [`BitPackedSliceMut`] view, and [`BitPackedArray`]).
///
/// Implementors only need to expose their backing storage via
/// [`Self::as_mut_bytes`].
pub trait BitPackedMutTrait<QI: QuantizedIndexCountTrait>:
    BitPackedTrait<QI> + IndexMut<QI, Output = u8> + IndexMut<Range<QI>, Output = [u8]>
{
    /// Mutably borrows the backing storage as a regular byte slice.
    fn as_mut_bytes(&mut self) -> &mut [u8];

    /// Sets the bit at `index`, returning the previous value if the index was in
    /// bounds (otherwise leaves the collection untouched).
    fn set_bit(&mut self, index: QI, value: bool) -> Option<bool> {
        let bit = index.quant_to_usize();
        if bit >= self.number_addressable_bits().quant_to_usize() {
            return None;
        }
        let byte = &mut self.as_mut_bytes()[bit / 8];
        let previous = (*byte >> (bit % 8)) & 1 == 1;
        if value {
            *byte |= 1 << (bit % 8);
        } else {
            *byte &= !(1 << (bit % 8));
        }
        Some(previous)
    }

    /// Mutably borrows the whole byte at `index`, or `None` if out of bounds.
    fn get_byte_mut(&mut self, index: QI) -> Option<&mut u8> {
        self.as_mut_bytes().get_mut(index.quant_to_usize())
    }

    /// Overwrites the whole byte at `index`, returning the previous value if the
    /// index was in bounds (otherwise leaves the collection untouched).
    fn set_byte(&mut self, index: QI, value: u8) -> Option<u8> {
        match self.as_mut_bytes().get_mut(index.quant_to_usize()) {
            Some(slot) => {
                let previous = *slot;
                *slot = value;
                Some(previous)
            }
            None => None,
        }
    }

    /// Mutably borrows a half-open *byte* sub-range as a [`BitPackedSliceMut`]
    /// view. The resulting view treats every byte as full (its bit count is
    /// `bytes * 8`).
    ///
    /// Returns [`FeagiFailCollectionInvalidIndex`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice_bytes_mut(&mut self, range: Range<QI>) -> Result<BitPackedSliceMut<'_, QI>, FeagiDataCollectionError> {
        match self.as_mut_bytes().get_mut(range.start.quant_to_usize()..range.end.quant_to_usize()) {
            Some(slice) => {
                let bits = QI::quant_from_usize(slice.len() * 8);
                Ok(BitPackedSliceMut::new(slice, bits))
            }
            None => Err(FeagiFailCollectionInvalidIndex::new("subslice byte range is out of bounds").into()),
        }
    }

    /// Iterates over mutable references to the bytes.
    fn iter_bytes_mut(&mut self) -> core::slice::IterMut<'_, u8> {
        self.as_mut_bytes().iter_mut()
    }

    /// Rayon parallel iterator over mutable references to the bytes.
    #[cfg(feature = "use-rayon")]
    fn par_iter_bytes_mut(&mut self) -> rayon::slice::IterMut<'_, u8> {
        use rayon::iter::IntoParallelRefMutIterator;
        self.as_mut_bytes().par_iter_mut()
    }
}

/// Unsafe, index-based parallel *read* access to the *bytes* of a bit-packed
/// quantized collection.
///
/// This lets callers grab shared references to many (possibly disjoint) byte
/// indices at once — e.g. from inside a rayon closure — while skipping repeated
/// bounds checks. See [`BitPackedParMutTrait`] for the mutable counterpart that
/// additionally allows disjoint parallel *writes* through a shared `&self`.
///
/// Only byte access is offered: individual bits within a byte are not
/// independently addressable, so there is deliberately no parallel bit access.
pub trait BitPackedParTrait<QI: QuantizedIndexCountTrait>: BitPackedTrait<QI> {
    /// Raw pointer to the first byte. Valid for reads of [`Self::number_bytes`]
    /// bytes for as long as `self` is borrowed.
    fn as_byte_ptr(&self) -> *const u8 {
        self.as_bytes().as_ptr()
    }

    /// Returns a shared reference to the byte at `index`, without bounds
    /// checking.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.number_bytes()`).
    unsafe fn get_byte_par(&self, index: QI) -> &u8 {
        &*self.as_byte_ptr().add(index.quant_to_usize())
    }
}

/// Unsafe, index-based parallel *mutable* access to the *bytes* of a bit-packed
/// quantized collection.
///
/// # Safety (implementors)
/// This trait must only be implemented for collections whose backing storage is
/// genuinely writable through a shared `&self` (i.e. the storage is owned, or is
/// exclusively borrowed like a mutable slice). It must **never** be implemented
/// for a shared/read-only view, since [`Self::get_byte_mut_par`] casts a
/// `*const u8` to `*mut u8`; writing through such a pointer that aliases a shared
/// borrow is undefined behaviour.
pub unsafe trait BitPackedParMutTrait<QI: QuantizedIndexCountTrait>: BitPackedParTrait<QI> + BitPackedMutTrait<QI> {
    /// Raw mutable pointer to the first byte, derived from a shared `&self`.
    ///
    /// # Safety
    /// The returned pointer aliases the collection's storage; writes through it
    /// must only target byte indices not simultaneously accessed elsewhere.
    unsafe fn as_mut_byte_ptr_par(&self) -> *mut u8 {
        self.as_byte_ptr() as *mut u8
    }

    /// Returns a mutable reference to the byte at `index` through a shared
    /// `&self`, enabling parallel mutation of disjoint bytes.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.number_bytes()`).
    /// - No other reference (shared or mutable) to the *same* byte may exist for
    ///   the duration of the returned borrow. Concurrent callers must only ever
    ///   target disjoint byte indices.
    unsafe fn get_byte_mut_par(&self, index: QI) -> &mut u8 {
        &mut *self.as_mut_byte_ptr_par().add(index.quant_to_usize())
    }
}

//region Vector

/// An owned, heap-allocated run of bit-packed booleans.
pub struct BitPackedVector<QI: QuantizedIndexCountTrait> {
    data: Vec<u8>, // length is number of bytes
    number_bits: QI,
}

impl<QI: QuantizedIndexCountTrait> Clone for BitPackedVector<QI> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            number_bits: self.number_bits,
        }
    }
}

impl<QI: QuantizedIndexCountTrait> BitPackedVector<QI> {
    /// Builds a vector holding `number_bits` booleans, every one initialised to
    /// `initial_state`. Any dangling bits in the final byte are kept zeroed.
    pub fn new_uniform(number_bits: QI, initial_state: bool) -> BitPackedVector<QI> {
        let number_bytes = QI::quant_from_usize(number_bits_to_number_bytes(number_bits.quant_to_usize()));
        let mut data: Vec<u8> = if initial_state {
            vec![0xFF; number_bytes.quant_to_usize()]
        } else {
            vec![0x00; number_bytes.quant_to_usize()]
        };

        Self { data, number_bits }
    }

    /// Wraps an existing `Vec` without copying, treating every byte as full
    /// (bit count is `data.len() * 8`, no dangling bits).
    pub fn from_vec(data: Vec<u8>) -> BitPackedVector<QI> {
        let number_bits = QI::quant_from_usize(data.len() * 8);
        Self { data, number_bits }
    }

    /// Wraps an existing `Vec` without copying, using an explicit bit count.
    /// `number_bits` must not exceed `data.len() * 8`.
    pub fn from_vec_with_bits(data: Vec<u8>, number_bits: QI) -> BitPackedVector<QI> {
        Self { data, number_bits }
    }

    /// Consumes the wrapper, returning the backing `Vec`.
    pub fn into_vec(self) -> Vec<u8> {
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
        let needed_bytes = number_bits_to_number_bytes(new_total_bits);

        if needed_bytes > self.data.len() {
            self.data.resize(needed_bytes, 0);
        }

        if initial_state {
            for bit_index in previous_bits..new_total_bits {
                let byte_index = bit_index >> 3;
                let bit_offset = bit_index & 0b00000111;
                self.data[byte_index] |= 1 << bit_offset;
            }
        }

        self.number_bits = QI::quant_from_usize(new_total_bits);
    }
}

impl<QI: QuantizedIndexCountTrait> BitPackedTrait<QI> for BitPackedVector<QI> {
    fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<QI: QuantizedIndexCountTrait> BitPackedMutTrait<QI> for BitPackedVector<QI> {
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl<QI: QuantizedIndexCountTrait> BitPackedParTrait<QI> for BitPackedVector<QI> {}

// SAFETY: the backing `Vec` is owned exclusively by this wrapper, so its storage
// is writable through a shared `&self` under the trait's disjoint-index contract.
unsafe impl<QI: QuantizedIndexCountTrait> BitPackedParMutTrait<QI> for BitPackedVector<QI> {}

impl<QI: QuantizedIndexCountTrait> Index<QI> for BitPackedVector<QI> {
    type Output = u8;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedIndexCountTrait> IndexMut<QI> for BitPackedVector<QI> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read_write!(
    BitPackedVector<QI>, QI,
    [QI: QuantizedIndexCountTrait]
);

impl<QI: QuantizedIndexCountTrait> From<Vec<u8>> for BitPackedVector<QI> {
    fn from(value: Vec<u8>) -> Self {
        Self::from_vec(value)
    }
}

impl<QI: QuantizedIndexCountTrait> From<BitPackedVector<QI>> for Vec<u8> {
    fn from(value: BitPackedVector<QI>) -> Self {
        value.data
    }
}

//endregion

//region Slice

/// A borrowed, read-only view over a run of bit-packed booleans.
#[derive(Clone, Copy)]
pub struct BitPackedSlice<'a, QI: QuantizedIndexCountTrait> {
    pub(crate) data: &'a [u8],
    number_bits: QI,
}

impl<'a, QI: QuantizedIndexCountTrait> BitPackedSlice<'a, QI> {
    /// Wraps an existing shared byte slice with an explicit bit count.
    /// `number_bits` must not exceed `data.len() * 8`.
    pub fn new(data: &'a [u8], number_bits: QI) -> BitPackedSlice<'a, QI> {
        Self { data, number_bits }
    }

    /// Wraps an existing shared byte slice, treating every byte as full
    /// (bit count is `data.len() * 8`, no dangling bits).
    pub fn from_bytes(data: &'a [u8]) -> BitPackedSlice<'a, QI> {
        let number_bits = QI::quant_from_usize(data.len() * 8);
        Self { data, number_bits }
    }

    /// Returns the underlying shared byte slice, keeping the original lifetime.
    pub fn into_bytes(self) -> &'a [u8] {
        self.data
    }
}

impl<'a, QI: QuantizedIndexCountTrait> BitPackedTrait<QI> for BitPackedSlice<'a, QI> {
    fn as_bytes(&self) -> &[u8] {
        self.data
    }

    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

// Read-only parallel access only: this view may alias a shared borrow, so the
// mutable `BitPackedParMutTrait` is intentionally NOT implemented.
impl<'a, QI: QuantizedIndexCountTrait> BitPackedParTrait<QI> for BitPackedSlice<'a, QI> {}

impl<'a, QI: QuantizedIndexCountTrait> Index<QI> for BitPackedSlice<'a, QI> {
    type Output = u8;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read!(
    BitPackedSlice<'a, QI>, QI,
    ['a, QI: QuantizedIndexCountTrait]
);

impl<'a, QI: QuantizedIndexCountTrait> From<&'a [u8]> for BitPackedSlice<'a, QI> {
    fn from(value: &'a [u8]) -> Self {
        Self::from_bytes(value)
    }
}

//endregion

//region Mut Slice

/// A borrowed, mutable view over a run of bit-packed booleans.
pub struct BitPackedSliceMut<'a, QI: QuantizedIndexCountTrait> {
    pub(crate) data: &'a mut [u8],
    number_bits: QI,
}

impl<'a, QI: QuantizedIndexCountTrait> BitPackedSliceMut<'a, QI> {
    /// Wraps an existing mutable byte slice with an explicit bit count.
    /// `number_bits` must not exceed `data.len() * 8`.
    pub fn new(data: &'a mut [u8], number_bits: QI) -> BitPackedSliceMut<'a, QI> {
        Self { data, number_bits }
    }

    /// Wraps an existing mutable byte slice, treating every byte as full
    /// (bit count is `data.len() * 8`, no dangling bits).
    pub fn from_bytes(data: &'a mut [u8]) -> BitPackedSliceMut<'a, QI> {
        let number_bits = QI::quant_from_usize(data.len() * 8);
        Self { data, number_bits }
    }

    /// Returns the underlying mutable byte slice, keeping the original lifetime.
    pub fn into_bytes_mut(self) -> &'a mut [u8] {
        self.data
    }

    /// Creates a shorter-lived, exclusive re-borrow of this view.
    pub fn reborrow(&mut self) -> BitPackedSliceMut<'_, QI> {
        BitPackedSliceMut::new(self.data, self.number_bits)
    }
}

impl<'a, QI: QuantizedIndexCountTrait> BitPackedTrait<QI> for BitPackedSliceMut<'a, QI> {
    fn as_bytes(&self) -> &[u8] {
        self.data
    }

    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<'a, QI: QuantizedIndexCountTrait> BitPackedMutTrait<QI> for BitPackedSliceMut<'a, QI> {
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.data
    }
}

impl<'a, QI: QuantizedIndexCountTrait> BitPackedParTrait<QI> for BitPackedSliceMut<'a, QI> {}

// SAFETY: the backing slice is exclusively borrowed, so its storage is writable
// through a shared `&self` under the trait's disjoint-index contract.
unsafe impl<'a, QI: QuantizedIndexCountTrait> BitPackedParMutTrait<QI> for BitPackedSliceMut<'a, QI> {}

impl<'a, QI: QuantizedIndexCountTrait> Index<QI> for BitPackedSliceMut<'a, QI> {
    type Output = u8;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedIndexCountTrait> IndexMut<QI> for BitPackedSliceMut<'a, QI> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read_write!(
    BitPackedSliceMut<'a, QI>, QI,
    ['a, QI: QuantizedIndexCountTrait]
);

impl<'a, QI: QuantizedIndexCountTrait> From<&'a mut [u8]> for BitPackedSliceMut<'a, QI> {
    fn from(value: &'a mut [u8]) -> Self {
        Self::from_bytes(value)
    }
}

//endregion

//region Array

/// An owned, stack-allocated run of bit-packed booleans backed by exactly `N`
/// bytes.
///
/// The compile-time length `N` is the *byte* count as a `usize` const generic
/// (Rust const generics must be an integer type, so `QI` is retained as the
/// associated index/count type used by the shared trait methods). The logical
/// bit count may be smaller than `N * 8` when there are dangling bits.
#[derive(Clone, Copy)]
pub struct BitPackedArray<QI: QuantizedIndexCountTrait, const N: usize> {
    pub(crate) data: [u8; N],
    number_bits: QI,
}

impl<QI: QuantizedIndexCountTrait, const N: usize> BitPackedArray<QI, N> {
    /// Builds an array holding `number_bits` booleans, every one initialised to
    /// `initial_state`. `number_bits` must not exceed `N * 8`. Any dangling bits
    /// in the final byte are kept zeroed.
    pub fn new_uniform(number_bits: QI, initial_state: bool) -> BitPackedArray<QI, N> {
        let mut data: [u8; N] = if initial_state { [0xFF; N] } else { [0x00; N] };

        Self { data, number_bits }
    }

    /// Wraps an existing array, treating every byte as full (bit count is
    /// `N * 8`, no dangling bits).
    pub fn from_array(data: [u8; N]) -> BitPackedArray<QI, N> {
        let number_bits = QI::quant_from_usize(N * 8);
        Self { data, number_bits }
    }

    /// Wraps an existing array with an explicit bit count. `number_bits` must not
    /// exceed `N * 8`.
    pub fn from_array_with_bits(data: [u8; N], number_bits: QI) -> BitPackedArray<QI, N> {
        Self { data, number_bits }
    }

    /// Consumes the wrapper, returning the backing array.
    pub fn into_array(self) -> [u8; N] {
        self.data
    }
}

impl<QI: QuantizedIndexCountTrait, const N: usize> BitPackedTrait<QI> for BitPackedArray<QI, N> {
    fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn number_addressable_bits(&self) -> QI {
        self.number_bits
    }
}

impl<QI: QuantizedIndexCountTrait, const N: usize> BitPackedMutTrait<QI> for BitPackedArray<QI, N> {
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl<QI: QuantizedIndexCountTrait, const N: usize> BitPackedParTrait<QI> for BitPackedArray<QI, N> {}

// SAFETY: the backing array is owned exclusively by this wrapper, so its storage
// is writable through a shared `&self` under the trait's disjoint-index contract.
unsafe impl<QI: QuantizedIndexCountTrait, const N: usize> BitPackedParMutTrait<QI> for BitPackedArray<QI, N> {}

impl<QI: QuantizedIndexCountTrait, const N: usize> Index<QI> for BitPackedArray<QI, N> {
    type Output = u8;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedIndexCountTrait, const N: usize> IndexMut<QI> for BitPackedArray<QI, N> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_bitpacked_range_read_write!(
    BitPackedArray<QI, N>, QI,
    [QI: QuantizedIndexCountTrait, const N: usize]
);

impl<QI: QuantizedIndexCountTrait, const N: usize> From<[u8; N]> for BitPackedArray<QI, N> {
    fn from(value: [u8; N]) -> Self {
        Self::from_array(value)
    }
}

//endregion
