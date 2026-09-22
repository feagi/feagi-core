use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};
use serde::{Deserialize, Serialize};
use feagi_basis_quantization::prelude::*;
use crate::feagi_collection_error::{FeagiDataCollectionError, ParDataInvalidRange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParData<QI, S> {
    pub(crate) data: S,
    pub(crate) _marker: PhantomData<QI>,
}

#[cfg(feature = "alloc")]
pub type ParDataVector<QI, D> = ParData<QI, Vec<D>>;

pub type ParDataArray<QI, D, const N: usize> = ParData<QI, [D; N]>;

pub type ParDataSlice<'a, QI, D> = ParData<QI, &'a [D]>;

pub type ParDataSliceMut<'a, QI, D> = ParData<QI, &'a mut [D]>;

#[cfg(feature = "heapless")]
pub type ParDataHeaplessVec<QI, D, const N: usize> = ParData<QI, ::heapless::Vec<D, N>>;

//region Backing Stores

/// Links a contiguous backing store to the element type it holds.
pub trait ParDataStore {
    /// The element type
    type Elem;

    /// Borrows the whole store as slice.
    fn store_as_slice(&self) -> &[Self::Elem];

    fn len(&self) -> usize {
        self.store_as_slice().len()
    }
}

/// A `ParDataStore` whose elements can also be mutated in place.
pub trait ParDataStoreMut: ParDataStore {
    /// Mutably borrows the whole store as a regular slice.
    fn store_as_mut_slice(&mut self) -> &mut [Self::Elem];
}

/// A `ParDataStoreMut` that can be resized.
pub trait ParDataStoreResizable: ParDataStoreMut {
    fn new_from_element_default(number_elements: usize) -> Self
    where Self::Elem: Default;

    fn new_from_element_clonable(number_elements: usize, source: Self::Elem) -> Self
    where Self::Elem: Clone;

}

#[cfg(feature = "alloc")]
impl<D> ParDataStore for Vec<D> {
    type Elem = D;
    fn store_as_slice(&self) -> &[D] {
        self
    }
}

#[cfg(feature = "alloc")]
impl<D> ParDataStoreMut for Vec<D> {
    fn store_as_mut_slice(&mut self) -> &mut [D] {
        self
    }
}

#[cfg(feature = "alloc")]
impl<D> ParDataStoreResizable for Vec<D> {
    fn new_from_element_default(number_elements: usize) -> Self
    where
        Self::Elem: Default
    {
        let mut out = Vec::with_capacity(number_elements);
        out.resize_with(number_elements, Self::Elem::default);
        out
    }

    fn new_from_element_clonable(number_elements: usize, source: Self::Elem) -> Self
    where
        Self::Elem: Clone
    {
        vec![source; number_elements]
    }
}


impl<D, const N: usize> ParDataStore for [D; N] {
    type Elem = D;
    fn store_as_slice(&self) -> &[D] {
        self
    }
}

impl<D, const N: usize> ParDataStoreMut for [D; N] {
    fn store_as_mut_slice(&mut self) -> &mut [D] {
        self
    }
}

impl<'a, D> ParDataStore for &'a [D] {
    type Elem = D;
    fn store_as_slice(&self) -> &[D] {
        self
    }
}

impl<'a, D> ParDataStore for &'a mut [D] {
    type Elem = D;
    fn store_as_slice(&self) -> &[D] {
        self
    }
}

impl<'a, D> ParDataStoreMut for &'a mut [D] {
    fn store_as_mut_slice(&mut self) -> &mut [D] {
        self
    }
}

#[cfg(feature = "heapless")]
impl<D, const N: usize> ParDataStore for ::heapless::Vec<D, N> {
    type Elem = D;
    fn store_as_slice(&self) -> &[D] {
        self
    }
}

#[cfg(feature = "heapless")]
impl<D, const N: usize> ParDataStoreMut for ::heapless::Vec<D, N> {
    fn store_as_mut_slice(&mut self) -> &mut [D] {
        self
    }
}

//endregion

//region Store-agnostic behaviour

impl<QI, S> ParData<QI, S> {
    /// Wraps a backing store, tagging it with the index type `QI`.
    pub const fn from_store(data: S) -> Self {
        Self { data, _marker: PhantomData } // TODO verify that the length doesnt exceed QI abilities
    }

    /// Consumes the wrapper, returning the backing store.
    pub fn into_store(self) -> S {
        self.data
    }

    /// Borrows the backing store.
    pub fn store(&self) -> &S {
        &self.data
    }

    /// Mutably borrows the backing store.
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.data
    }
}

impl<QI, S: Default> Default for ParData<QI, S> {
    fn default() -> Self {
        Self::from_store(S::default())
    }
}

/// Shared behaviour
impl<QI: QuantizedUnsignedIntegerTrait, S: ParDataStore> ParData<QI, S> {

    /// Number of elements
    pub fn len(&self) -> usize {
        self.data.store_as_slice().len()
    }

    /// Returns `true` if there are no elements.
    pub fn is_empty(&self) -> bool {
        self.data.store_as_slice().is_empty()
    }

    /// Gets the element at `index`, or `None` if out of bounds.
    pub fn get(&self, index: QI) -> Option<&S::Elem> {
        self.data.store_as_slice().get(index.quant_to_usize())
    }

    /// Iterates over shared references to the elements.
    pub fn iter(&self) -> core::slice::Iter<'_, S::Elem> {
        self.data.store_as_slice().iter()
    }

    /// Raw pointer to the first element
    pub fn as_ptr(&self) -> *const S::Elem {
        self.data.store_as_slice().as_ptr()
    }

    /// Returns a shared reference to the element at `index`, without bounds checking.
    pub unsafe fn get_par(&self, index: QI) -> &S::Elem {
        &*self.as_ptr().add(index.quant_to_usize())
    }

    /// Rayon parallel iterator
    #[cfg(feature = "expose_rayon")]
    pub fn rayon_iter(&self) -> rayon::slice::Iter<'_, S::Elem>
    where
        S::Elem: Send + Sync,
    {
        use rayon::iter::IntoParallelRefIterator;
        self.data.store_as_slice().par_iter()
    }

    /// Borrows the whole collection as a [`ParDataSlice`] view.
    pub fn as_data_slice(&self) -> ParDataSlice<'_, QI, S::Elem> {
        ParData::from_store(self.data.store_as_slice())
    }

    /// Borrows a half-open element sub-range as a [`ParDataSlice`] view.
    pub fn subslice(&self, range: Range<QI>) -> Result<ParDataSlice<'_, QI, S::Elem>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.data.store_as_slice().get(start..end) {
            Some(slice) => Ok(ParData::from_store(slice)),
            None => Err(ParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }

    /// Copies the internal elements to a new owned vector structure.
    #[cfg(feature = "alloc")]
    pub fn clone_to_vector(&self) -> ParDataVector<QI, S::Elem>
    where
        S::Elem: Clone,
    {
        ParData::from_store(self.data.store_as_slice().to_vec())
    }
}

/// mutable
impl<QI: QuantizedUnsignedIntegerTrait, S: ParDataStoreMut> ParData<QI, S> {

    /// Mutably borrows the element at `index`, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: QI) -> Option<&mut S::Elem> {
        self.data.store_as_mut_slice().get_mut(index.quant_to_usize())
    }

    /// Overwrites the element at `index`, returning the previous value if the
    /// index was in bounds (otherwise leaves the collection untouched).
    pub fn set(&mut self, index: QI, value: S::Elem) -> Option<S::Elem> {
        match self.get_mut(index) {
            Some(slot) => {
                let returned = core::mem::replace(slot, value);
                Some(returned)
            }
            None => None,
        }
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, S::Elem> {
        self.data.store_as_mut_slice().iter_mut()
    }

    #[cfg(feature = "expose_rayon")]
    pub fn rayon_iter_mut(&mut self) -> rayon::slice::IterMut<'_, S::Elem>
    where
        S::Elem: Send + Sync,
    {
        use rayon::iter::IntoParallelRefMutIterator;
        self.data.store_as_mut_slice().par_iter_mut()
    }

    /// Raw mutable pointer to the first element
    pub unsafe fn as_mut_ptr_par(&self) -> *mut S::Elem {
        self.as_ptr() as *mut S::Elem
    }

    /// Returns a mutable reference to the element at `index` through a shared
    /// `&self`, enabling parallel mutation of multiple indices. MUST NOT WRITE TO THE SAME
    /// INDEX IN PARALLEL!
    pub unsafe fn get_mut_par(&self, index: QI) -> &mut S::Elem {
        &mut *self.as_mut_ptr_par().add(index.quant_to_usize())
    }

    /// Mutably borrows a half-open element sub-range as a [`ParDataSliceMut`] view.
    pub fn subslice_mut(&mut self, range: Range<QI>) -> Result<ParDataSliceMut<'_, QI, S::Elem>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.data.store_as_mut_slice().get_mut(start..end) {
            Some(slice) => Ok(ParData::from_store(slice)),
            None => Err(ParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }
}

impl<QI, S> Index<QI> for ParData<QI, S>
where
    QI: QuantizedUnsignedIntegerTrait,
    S: ParDataStore,
{
    type Output = S::Elem;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data.store_as_slice()[index.quant_to_usize()]
    }
}

impl<QI, S> IndexMut<QI> for ParData<QI, S>
where
    QI: QuantizedUnsignedIntegerTrait,
    S: ParDataStoreMut,
{
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data.store_as_mut_slice()[index.quant_to_usize()]
    }
}

//endregion

//region Vector

#[cfg(feature = "alloc")]
impl<QI: QuantizedUnsignedIntegerTrait, D> ParData<QI, Vec<D>> {
    /// Builds a vector of `number_elements` entries, every one initialised to
    /// `initial_value`.
    pub fn new_uniform(number_elements: QI, initial_value: D) -> Self
    where
        D: Clone,
    {
        Self::from_store(vec![initial_value; number_elements.quant_to_usize()])
    }

    /// Wraps an existing `Vec` without copying.
    pub fn from_vec(data: Vec<D>) -> Self {
        Self::from_store(data)
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
impl<QI, D> From<Vec<D>> for ParData<QI, Vec<D>> {
    fn from(value: Vec<D>) -> Self {
        Self::from_store(value)
    }
}

#[cfg(feature = "alloc")]
impl<QI, D> From<ParData<QI, Vec<D>>> for Vec<D> {
    fn from(value: ParData<QI, Vec<D>>) -> Self {
        value.data
    }
}

//endregion

//region Slice

impl<'a, QI, D> ParData<QI, &'a [D]> {
    /// Wraps an existing shared slice.
    pub const fn new(data: &'a [D]) -> Self {
        Self::from_store(data)
    }

    /// Alias for [`Self::new`].
    pub const fn from_slice(data: &'a [D]) -> Self {
        Self::from_store(data)
    }

    /// Returns the underlying shared slice, keeping the original lifetime.
    pub const fn into_slice(self) -> &'a [D] {
        self.data
    }
}

impl<'a, QI, D> From<&'a [D]> for ParData<QI, &'a [D]> {
    fn from(value: &'a [D]) -> Self {
        Self::from_store(value)
    }
}

//endregion

//region Mut Slice

impl<'a, QI, D> ParData<QI, &'a mut [D]> {
    /// Wraps an existing mutable slice.
    pub const fn new(data: &'a mut [D]) -> Self {
        Self::from_store(data)
    }

    /// Alias for [`Self::new`].
    pub const fn from_slice_mut(data: &'a mut [D]) -> Self {
        Self::from_store(data)
    }

    /// Returns the underlying mutable slice, keeping the original lifetime.
    pub fn into_slice_mut(self) -> &'a mut [D] {
        self.data
    }

    /// Creates a shorter-lived, exclusive re-borrow of this view.
    pub fn reborrow(&mut self) -> ParData<QI, &mut [D]> {
        ParData::from_store(&mut *self.data)
    }
}

impl<'a, QI, D> From<&'a mut [D]> for ParData<QI, &'a mut [D]> {
    fn from(value: &'a mut [D]) -> Self {
        Self::from_store(value)
    }
}

//endregion

//region Array

impl<QI, D, const N: usize> ParData<QI, [D; N]> {
    /// Builds an array of `N` entries, every one initialised to `initial_value`.
    pub fn new_uniform(initial_value: D) -> Self
    where
        D: Clone,
    {
        Self::from_store([(); N].map(|_| initial_value.clone()))
    }

    /// Wraps an existing array.
    pub const fn from_array(data: [D; N]) -> Self {
        Self::from_store(data)
    }

    /// Consumes the wrapper, returning the backing array.
    pub fn into_array(self) -> [D; N] {
        self.data
    }
}

impl<QI, D, const N: usize> From<[D; N]> for ParData<QI, [D; N]> {
    fn from(value: [D; N]) -> Self {
        Self::from_store(value)
    }
}

//endregion

//region Heapless Vector

#[cfg(feature = "heapless")]
impl<QI, D, const N: usize> ParData<QI, ::heapless::Vec<D, N>> {
    /// Builds a vector of `N` entries, every one initialised to `initial_value`.
    pub fn new_uniform(initial_value: D) -> Self
    where
        D: Clone,
    {
        Self::from_store(::heapless::Vec::from_array([(); N].map(|_| initial_value.clone())))
    }

    /// Wraps an existing array, filling the vector to capacity.
    pub fn from_array(data: [D; N]) -> Self {
        Self::from_store(::heapless::Vec::from_array(data))
    }

    /// Consumes the wrapper, returning the elements as a full array.
    ///
    /// Panics if the vector is not filled to capacity.
    pub fn into_array(self) -> [D; N]
    where
        D: core::fmt::Debug,
    {
        self.data.into_array().unwrap()
    }
}

#[cfg(feature = "heapless")]
impl<QI, D, const N: usize> From<[D; N]> for ParData<QI, ::heapless::Vec<D, N>> {
    fn from(value: [D; N]) -> Self {
        Self::from_store(::heapless::Vec::from_array(value))
    }
}

//endregion

