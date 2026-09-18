use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};
use serde::{Deserialize, Serialize};
use feagi_basis_quantization::prelude::*;
use crate::feagi_collection_error::{FeagiDataCollectionError, ParDataInvalidRange};

/// Shared read behaviour for quantized-indexed collections backed by contiguous
/// generic data.
pub trait GenericParData<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug>: Index<QI, Output=D> {
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
    fn get(&self, index: QI) -> Option<&D> {
        self.as_slice().get(index.quant_to_usize())
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
    fn rayon_iter(&self) -> rayon::slice::Iter<'_, D>
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
    fn subslice(&self, range: Range<QI>) -> Result<ParDataSlice<'_, QI, D>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.as_slice().get(start..end) {
            Some(slice) => Ok(ParDataSlice::new(slice)),
            None => Err(ParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }

    #[cfg(feature = "alloc")]
    /// Copies the internal elements to a new owned vector structure.
    fn clone_to_vector(&self) -> ParDataVector<QI, D>
    where
        D: Clone,
    {
        ParDataVector::from_vec(self.as_slice().to_vec())
    }

    //endregion
}

/// Shared mutable behaviour for quantized-indexed collections that own or
/// exclusively borrow their storage ([`ParDataVector`], [`ParDataSliceMut`], and
/// [`ParDataArray`]).
pub trait GenericParDataMut<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug>:
GenericParData<QI, D> + IndexMut<QI, Output=D>
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
        match self.get_mut(index) {
            Some(slot) => {
                let returned = core::mem::replace(slot, value);
                Some(returned)
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
    fn rayon_iter_mut(&mut self) -> rayon::slice::IterMut<'_, D>
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
    fn subslice_mut(&mut self, range: Range<QI>) -> Result<ParDataSliceMut<'_, QI, D>, FeagiDataCollectionError> {
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
#[cfg(feature = "alloc")]
#[derive(Debug, Serialize, Deserialize)]
pub struct ParDataVector<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> {
    pub(crate) data: Vec<D>,
    pub(crate) _marker: PhantomData<QI>,
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> ParDataVector<QI, D> {
    /// Builds a vector of `number_elements` entries, every one initialised to
    /// `initial_value`.
    pub fn new_uniform(number_elements: QI, initial_value: D) -> ParDataVector<QI, D>
    where
        D: Clone,
    {
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
    pub fn append(&mut self, number_elements: QI, value: D)
    where
        D: Clone,
    {
        let additional = number_elements.quant_to_usize();
        if additional == 0 {
            return;
        }
        self.data.extend(core::iter::repeat_n(value, additional));
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> GenericParData<QI, D> for ParDataVector<QI, D> {
    fn as_slice(&self) -> &[D] {
        &self.data
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> GenericParDataMut<QI, D> for ParDataVector<QI, D> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        &mut self.data
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> Index<QI> for ParDataVector<QI, D> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> IndexMut<QI> for ParDataVector<QI, D> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> Default for ParDataVector<QI, D> {
    fn default() -> Self {
        Self {
            data: vec![],
            _marker: Default::default(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> From<Vec<D>> for ParDataVector<QI, D> {
    fn from(value: Vec<D>) -> Self {
        Self::from_vec(value)
    }
}

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> From<ParDataVector<QI, D>> for Vec<D> {
    fn from(value: ParDataVector<QI, D>) -> Self {
        value.data
    }
}

//endregion

//region Slice

/// A borrowed, read-only view over a run of generic elements indexed by `QI`.
#[derive(Debug, Serialize)]
pub struct ParDataSlice<'a, QI: QuantizedUnsignedIntegerTrait, D> {
    pub(crate) data: &'a [D],
    pub(crate) _marker: PhantomData<QI>,
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> ParDataSlice<'a, QI, D> {
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

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> GenericParData<QI, D> for ParDataSlice<'a, QI, D> {
    fn as_slice(&self) -> &[D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> Index<QI> for ParDataSlice<'a, QI, D> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> From<&'a [D]> for ParDataSlice<'a, QI, D> {
    fn from(value: &'a [D]) -> Self {
        Self::from_slice(value)
    }
}

//endregion

//region Mut Slice

/// A borrowed, mutable view over a run of generic elements indexed by `QI`.
#[derive(Debug, Serialize)]
pub struct ParDataSliceMut<'a, QI: QuantizedUnsignedIntegerTrait, D> {
    pub(crate) data: &'a mut [D],
    pub(crate) _marker: PhantomData<QI>,
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> ParDataSliceMut<'a, QI, D> {
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

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> GenericParData<QI, D> for ParDataSliceMut<'a, QI, D> {
    fn as_slice(&self) -> &[D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> GenericParDataMut<QI, D> for ParDataSliceMut<'a, QI, D> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> Index<QI> for ParDataSliceMut<'a, QI, D> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> IndexMut<QI> for ParDataSliceMut<'a, QI, D> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug> From<&'a mut [D]> for ParDataSliceMut<'a, QI, D> {
    fn from(value: &'a mut [D]) -> Self {
        Self::from_slice_mut(value)
    }
}

//endregion

//region Array

/// An owned, stack-allocated run of generic elements backed by exactly `N`
/// entries.
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "[D; N]: ::serde::Serialize",
    deserialize = "[D; N]: ::serde::Deserialize<'de>"
))]
pub struct ParDataArray<QI: QuantizedUnsignedIntegerTrait, D, const N: usize> {
    pub(crate) data: [D; N],
    pub(crate) _marker: PhantomData<QI>,
}

impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> ParDataArray<QI, D, N> {
    /// Builds an array of `N` entries, every one initialised to `initial_value`.
    pub fn new_uniform(initial_value: D) -> ParDataArray<QI, D, N>
    where
        D: Clone,
    {
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

impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> GenericParData<QI, D> for ParDataArray<QI, D, N> {
    fn as_slice(&self) -> &[D] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> GenericParDataMut<QI, D> for ParDataArray<QI, D, N> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> Index<QI> for ParDataArray<QI, D, N> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> IndexMut<QI> for ParDataArray<QI, D, N> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> From<[D; N]> for ParDataArray<QI, D, N> {
    fn from(value: [D; N]) -> Self {
        Self::from_array(value)
    }
}

//endregion

//region Heapless Vector

#[cfg(feature = "heapless")]
/// An owned, stack-allocated run of generic elements backed by exactly `N`
/// entries.
#[derive(Debug, Serialize, Deserialize)]
/*
#[serde(bound(
    serialize = "[D; N]: ::serde::Serialize",
    deserialize = "[D; N]: ::serde::Deserialize<'de>"
))]

 */
pub struct ParDataHeaplessVec<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> {
    pub(crate) data: ::heapless::Vec<D, N>,
    pub(crate) _marker: PhantomData<QI>,
}

#[cfg(feature = "heapless")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> ParDataHeaplessVec<QI, D, N> {
    /// Builds an array of `N` entries, every one initialised to `initial_value`.
    pub fn new_uniform(initial_value: D) -> ParDataHeaplessVec<QI, D, N>
    where
        D: Clone,
    {
        Self {
            data: ::heapless::Vec::from_array([(); N].map(|_| initial_value.clone())),
            _marker: PhantomData,
        }
    }

    /// Wraps an existing array.
    pub fn from_array(data: [D; N]) -> ParDataHeaplessVec<QI, D, N> {
        Self { data: ::heapless::Vec::from_array(data), _marker: PhantomData }
    }

    /// Consumes the wrapper, returning the backing array.
    pub fn into_array(self) -> [D; N]
    {
        self.data.into_array().unwrap()
    }
}

#[cfg(feature = "heapless")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> GenericParData<QI, D> for ParDataHeaplessVec<QI, D, N> {
    fn as_slice(&self) -> &[D] {
        &self.data
    }
}

#[cfg(feature = "heapless")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> GenericParDataMut<QI, D> for ParDataHeaplessVec<QI, D, N> {
    fn as_mut_slice(&mut self) -> &mut [D] {
        &mut self.data
    }
}

#[cfg(feature = "heapless")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> Index<QI> for ParDataHeaplessVec<QI, D, N> {
    type Output = D;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

#[cfg(feature = "heapless")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> IndexMut<QI> for ParDataHeaplessVec<QI, D, N> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

#[cfg(feature = "heapless")]
impl<QI: QuantizedUnsignedIntegerTrait, D: core::fmt::Debug, const N: usize> From<[D; N]> for ParDataHeaplessVec<QI, D, N> {
    fn from(value: [D; N]) -> Self {
        Self::from_array(value)
    }
}

//endregion

//endregion
