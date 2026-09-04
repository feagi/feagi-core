use crate::generic_collections::generic_par_data::par_data_error::{ParDataError, ParDataInvalidRange};
use crate::values::quantizable::QuantizedUnsignedIntegerTrait;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};

macro_rules! impl_par_data_range_read {
    ($self_ty:ty, $qi:ty, $d:ty, [$($generics:tt)*]) => {
        impl<$($generics)*> Index<Range<$qi>> for $self_ty {
            type Output = [$d];
            fn index(&self, range: Range<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeInclusive<$qi>> for $self_ty {
            type Output = [$d];
            fn index(&self, range: core::ops::RangeInclusive<$qi>) -> &Self::Output {
                &self.data[range.start().quant_to_usize()..=range.end().quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFrom<$qi>> for $self_ty {
            type Output = [$d];
            fn index(&self, range: core::ops::RangeFrom<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeTo<$qi>> for $self_ty {
            type Output = [$d];
            fn index(&self, range: core::ops::RangeTo<$qi>) -> &Self::Output {
                &self.data[..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFull> for $self_ty {
            type Output = [$d];
            fn index(&self, _range: core::ops::RangeFull) -> &Self::Output {
                &self.data[..]
            }
        }
    };
}

macro_rules! impl_par_data_range_read_write {
    ($self_ty:ty, $qi:ty, $d:ty, [$($generics:tt)*]) => {
        impl_par_data_range_read!($self_ty, $qi, $d, [$($generics)*]);

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

/// Shared read behaviour for quantized-indexed collections backed by contiguous
/// generic data.
pub trait ParData<QI: QuantizedUnsignedIntegerTrait, D: Clone>: Index<QI, Output = D> + Index<Range<QI>, Output = [D]> {
    /// Borrows the backing storage as a regular shared slice.
    fn as_slice(&self) -> &[D];

    //region Default impls

    /// Number of elements in this collection.
    fn len(&self) -> QI {
        QI::quant_from_usize_unchecked(self.as_slice().len())
    }

    /// Returns `true` if there are no elements.
    fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// Copies out the element at `index`, or `None` if out of bounds.
    fn get(&self, index: QI) -> Option<D> {
        self.as_slice().get(index.quant_to_usize()).cloned()
    }

    /// Iterates over shared references to the elements.
    fn iter(&self) -> core::slice::Iter<'_, D> {
        self.as_slice().iter()
    }

    /// Raw pointer to the first element. Valid for reads of [`Self::len`] elements
    /// for as long as `self` is borrowed.
    fn as_ptr(&self) -> *const D {
        self.as_slice().as_ptr()
    }

    /// Returns a shared reference to the element at `index`, without bounds checking.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.len()`).
    unsafe fn get_par(&self, index: QI) -> &D {
        &*self.as_ptr().add(index.quant_to_usize())
    }

    /// Rayon parallel iterator over shared references to the elements.
    #[cfg(feature = "expose_rayon")]
    fn par_iter(&self) -> rayon::slice::Iter<'_, D>
    where
        D: Send + Sync,
    {
        use rayon::iter::IntoParallelRefIterator;
        self.as_slice().par_iter()
    }

    /// Borrows the whole collection as a [`ParDataSlice`] view.
    fn as_data_slice(&self) -> ParDataSlice<'_, QI, D> {
        ParDataSlice::new(self.as_slice())
    }

    /// Borrows a half-open element sub-range as a [`ParDataSlice`] view.
    ///
    /// Returns [`ParDataInvalidRange`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice(&self, range: Range<QI>) -> Result<ParDataSlice<'_, QI, D>, ParDataError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.as_slice().get(start..end) {
            Some(slice) => Ok(ParDataSlice::new(slice)),
            None => Err(ParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }

    /// Copies the internal elements to a new owned vector structure.
    fn clone_to_owned(&self) -> ParDataVector<QI, D> {
        ParDataVector::from_vec(self.as_slice().to_vec())
    }

    //endregion
}

/// Shared mutable behaviour for quantized-indexed collections that own or
/// exclusively borrow their storage ([`ParDataVector`], [`ParDataSliceMut`], and
/// [`ParDataArray`]).
pub trait ParDataMut<QI: QuantizedUnsignedIntegerTrait, D: Clone>:
    ParData<QI, D> + IndexMut<QI, Output = D> + IndexMut<Range<QI>, Output = [D]>
{
    /// Mutably borrows the backing storage as a regular slice.
    fn as_mut_slice(&mut self) -> &mut [D];

    //region Default impls

    /// Mutably borrows the element at `index`, or `None` if out of bounds.
    fn get_mut(&mut self, index: QI) -> Option<&mut D> {
        self.as_mut_slice().get_mut(index.quant_to_usize())
    }

    /// Overwrites the element at `index`, returning the previous value if the
    /// index was in bounds (otherwise leaves the collection untouched).
    fn set(&mut self, index: QI, value: D) -> Option<D> {
        match self.as_mut_slice().get_mut(index.quant_to_usize()) {
            Some(slot) => {
                let previous = slot.clone();
                *slot = value;
                Some(previous)
            }
            None => None,
        }
    }

    /// Iterates over mutable references to the elements.
    fn iter_mut(&mut self) -> core::slice::IterMut<'_, D> {
        self.as_mut_slice().iter_mut()
    }

    /// Rayon parallel iterator over mutable references to the elements.
    #[cfg(feature = "expose_rayon")]
    fn par_iter_mut(&mut self) -> rayon::slice::IterMut<'_, D>
    where
        D: Send + Sync,
    {
        use rayon::iter::IntoParallelRefMutIterator;
        self.as_mut_slice().par_iter_mut()
    }

    /// Raw mutable pointer to the first element, derived from a shared `&self`.
    ///
    /// # Safety
    /// The returned pointer aliases the collection's storage; writes through it
    /// must only target indices not simultaneously accessed elsewhere.
    unsafe fn as_mut_ptr_par(&self) -> *mut D {
        self.as_ptr() as *mut D
    }

    /// Returns a mutable reference to the element at `index` through a shared
    /// `&self`, enabling parallel mutation of disjoint indices.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.len()`).
    /// - No other reference (shared or mutable) to the *same* element may exist
    ///   for the duration of the returned borrow. Concurrent callers must only ever
    ///   target disjoint indices.
    unsafe fn get_mut_par(&self, index: QI) -> &mut D {
        &mut *self.as_mut_ptr_par().add(index.quant_to_usize())
    }

    /// Mutably borrows a half-open element sub-range as a [`ParDataSliceMut`] view.
    fn subslice_mut(&mut self, range: Range<QI>) -> Result<ParDataSliceMut<'_, QI, D>, ParDataError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.as_mut_slice().get_mut(start..end) {
            Some(slice) => Ok(ParDataSliceMut::new(slice)),
            None => Err(ParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }

    //endregion
}

//region Implementations

//region Vector

/// An owned, heap-allocated run of generic elements indexed by `QI`.
pub struct ParDataVector<QI: QuantizedUnsignedIntegerTrait, D: Clone> {
    pub(crate) data: Vec<D>,
    pub(crate) _marker: PhantomData<QI>,
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> Clone for ParDataVector<QI, D> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            _marker: PhantomData,
        }
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> ParDataVector<QI, D> {
    /// Builds a vector of `number_elements` entries, every one initialised to
    /// `initial_value`.
    pub fn new_uniform(number_elements: QI, initial_value: D) -> ParDataVector<QI, D> {
        Self {
            data: vec![initial_value; number_elements.quant_to_usize()],
            _marker: PhantomData,
        }
    }

    /// Wraps an existing `Vec` without copying.
    pub fn from_vec(data: Vec<D>) -> ParDataVector<QI, D> {
        Self { data, _marker: PhantomData }
    }

    /// Consumes the wrapper, returning the backing `Vec`.
    pub fn into_vec(self) -> Vec<D> {
        self.data
    }

    /// Appends `number_elements` new entries to the end of the vector, each
    /// initialised to `value`.
    pub fn append(&mut self, number_elements: QI, value: D) {
        let additional = number_elements.quant_to_usize();
        if additional == 0 {
            return;
        }
        self.data.extend(core::iter::repeat_n(value, additional));
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> ParData<QI, D> for ParDataVector<QI, D> {
    fn as_slice(&self) -> &[D] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> ParDataMut<QI, D> for ParDataVector<QI, D> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> Index<QI> for ParDataVector<QI, D> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> IndexMut<QI> for ParDataVector<QI, D> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> Default for ParDataVector<QI, D> {
    fn default() -> Self {
        Self {
            data: vec![],
            _marker: Default::default(),
        }
    }
}

impl_par_data_range_read_write!(
    ParDataVector<QI, D>,
    QI,
    D,
    [QI: QuantizedUnsignedIntegerTrait, D: Clone]
);

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> From<Vec<D>> for ParDataVector<QI, D> {
    fn from(value: Vec<D>) -> Self {
        Self::from_vec(value)
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone> From<ParDataVector<QI, D>> for Vec<D> {
    fn from(value: ParDataVector<QI, D>) -> Self {
        value.data
    }
}

//endregion

//region Slice

/// A borrowed, read-only view over a run of generic elements indexed by `QI`.
#[derive(Clone, Copy)]
pub struct ParDataSlice<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> {
    pub(crate) data: &'a [D],
    pub(crate) _marker: PhantomData<QI>,
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> ParDataSlice<'a, QI, D> {
    /// Wraps an existing shared slice.
    pub fn new(data: &'a [D]) -> ParDataSlice<'a, QI, D> {
        Self { data, _marker: PhantomData }
    }

    /// Alias for [`Self::new`].
    pub fn from_slice(data: &'a [D]) -> ParDataSlice<'a, QI, D> {
        Self::new(data)
    }

    /// Returns the underlying shared slice, keeping the original lifetime.
    pub fn into_slice(self) -> &'a [D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> ParData<QI, D> for ParDataSlice<'a, QI, D> {
    fn as_slice(&self) -> &[D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> Index<QI> for ParDataSlice<'a, QI, D> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl_par_data_range_read!(
    ParDataSlice<'a, QI, D>,
    QI,
    D,
    ['a, QI: QuantizedUnsignedIntegerTrait, D: Clone]
);

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> From<&'a [D]> for ParDataSlice<'a, QI, D> {
    fn from(value: &'a [D]) -> Self {
        Self::from_slice(value)
    }
}

//endregion

//region Mut Slice

/// A borrowed, mutable view over a run of generic elements indexed by `QI`.
pub struct ParDataSliceMut<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> {
    pub(crate) data: &'a mut [D],
    pub(crate) _marker: PhantomData<QI>,
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> ParDataSliceMut<'a, QI, D> {
    /// Wraps an existing mutable slice.
    pub fn new(data: &'a mut [D]) -> ParDataSliceMut<'a, QI, D> {
        Self { data, _marker: PhantomData }
    }

    /// Alias for [`Self::new`].
    pub fn from_slice_mut(data: &'a mut [D]) -> ParDataSliceMut<'a, QI, D> {
        Self::new(data)
    }

    /// Returns the underlying mutable slice, keeping the original lifetime.
    pub fn into_slice_mut(self) -> &'a mut [D] {
        self.data
    }

    /// Creates a shorter-lived, exclusive re-borrow of this view.
    pub fn reborrow(&mut self) -> ParDataSliceMut<'_, QI, D> {
        ParDataSliceMut::new(self.data)
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> ParData<QI, D> for ParDataSliceMut<'a, QI, D> {
    fn as_slice(&self) -> &[D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> ParDataMut<QI, D> for ParDataSliceMut<'a, QI, D> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> Index<QI> for ParDataSliceMut<'a, QI, D> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> IndexMut<QI> for ParDataSliceMut<'a, QI, D> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_par_data_range_read_write!(
    ParDataSliceMut<'a, QI, D>,
    QI,
    D,
    ['a, QI: QuantizedUnsignedIntegerTrait, D: Clone]
);

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: Clone> From<&'a mut [D]> for ParDataSliceMut<'a, QI, D> {
    fn from(value: &'a mut [D]) -> Self {
        Self::from_slice_mut(value)
    }
}

//endregion

//region Array

/// An owned, stack-allocated run of generic elements backed by exactly `N`
/// entries.
#[derive(Clone)]
pub struct ParDataArray<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> {
    pub(crate) data: [D; N],
    pub(crate) _marker: PhantomData<QI>,
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> ParDataArray<QI, D, N> {
    /// Builds an array of `N` entries, every one initialised to `initial_value`.
    pub fn new_uniform(initial_value: D) -> ParDataArray<QI, D, N> {
        Self {
            data: [(); N].map(|_| initial_value.clone()),
            _marker: PhantomData,
        }
    }

    /// Wraps an existing array.
    pub fn from_array(data: [D; N]) -> ParDataArray<QI, D, N> {
        Self { data, _marker: PhantomData }
    }

    /// Consumes the wrapper, returning the backing array.
    pub fn into_array(self) -> [D; N] {
        self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> ParData<QI, D> for ParDataArray<QI, D, N> {
    fn as_slice(&self) -> &[D] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> ParDataMut<QI, D> for ParDataArray<QI, D, N> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> Index<QI> for ParDataArray<QI, D, N> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> IndexMut<QI> for ParDataArray<QI, D, N> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl_par_data_range_read_write!(
    ParDataArray<QI, D, N>,
    QI,
    D,
    [QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize]
);

impl<QI: QuantizedUnsignedIntegerTrait, D: Clone, const N: usize> From<[D; N]> for ParDataArray<QI, D, N> {
    fn from(value: [D; N]) -> Self {
        Self::from_array(value)
    }
}

//endregion

//endregion
