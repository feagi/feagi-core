use crate::collections::feagi_data_collections_error::{FeagiDataCollectionError, FeagiFailCollectionInvalidIndex};
use crate::values::quantizable::{QuantizedUnsignedIntegerWrappedTrait};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};

/// Creates Vector, Array, Slice and SliceMut contiguous linear collections, which allow operations
/// similar to their named counterparts but more natively with quantization and with functions for
/// (unsafe) parallel mutation
#[macro_export]
macro_rules! create_quantized_contiguous_linear_collections {
    ($vis:vis, $base_name:ident, $index:ty, $count:ty, $data:ty) => {
        ::paste::paste! {
            $vis type [<$base_name Vector>]<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $crate::collections::linear::contiguous_data::QuantizedContiguousVector<$index<Q>, $count<Q>, $data>;
            $vis type [<$base_name Array>] <Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait, const SIZE: usize> = $crate::collections::linear::contiguous_data::QuantizedContiguousArray<$index<Q>, $count<Q>, $data, SIZE>;
            $vis type [<$base_name Slice>] <'a, Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $crate::collections::linear::contiguous_data::QuantizedContiguousSlice<'a, $index<Q>, $count<Q>, $data>;
            $vis type [<$base_name SliceMut>] <'a, Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $crate::collections::linear::contiguous_data::QuantizedContiguousSliceMut<'a, $index<Q>, $count<Q>, $data>;
        }
    };
}

macro_rules! impl_quantized_range_read {
    ($self_ty:ty, $qi:ty, $elem:ty, [$($generics:tt)*]) => {
        impl<$($generics)*> Index<Range<$qi>> for $self_ty {
            type Output = [$elem];
            fn index(&self, range: Range<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeInclusive<$qi>> for $self_ty {
            type Output = [$elem];
            fn index(&self, range: core::ops::RangeInclusive<$qi>) -> &Self::Output {
                &self.data[range.start().quant_to_usize()..=range.end().quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFrom<$qi>> for $self_ty {
            type Output = [$elem];
            fn index(&self, range: core::ops::RangeFrom<$qi>) -> &Self::Output {
                &self.data[range.start.quant_to_usize()..]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeTo<$qi>> for $self_ty {
            type Output = [$elem];
            fn index(&self, range: core::ops::RangeTo<$qi>) -> &Self::Output {
                &self.data[..range.end.quant_to_usize()]
            }
        }

        impl<$($generics)*> Index<core::ops::RangeFull> for $self_ty {
            type Output = [$elem];
            fn index(&self, _range: core::ops::RangeFull) -> &Self::Output {
                &self.data[..]
            }
        }
    };
}

macro_rules! impl_quantized_range_read_write {
    ($self_ty:ty, $qi:ty, $elem:ty, [$($generics:tt)*]) => {
        impl_quantized_range_read!($self_ty, $qi, $elem, [$($generics)*]);

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

/// Shared, read-only behaviour for every contiguous quantized collection in this
/// module (the owned [`QuantizedContiguousVector`], the borrowed
/// [`QuantizedContiguousSlice`] / [`QuantizedContiguousSliceMut`] views, and the
/// fixed-size [`QuantizedContiguousArray`]).
///
/// Implementors only need to expose their backing storage via [`Self::as_slice`]
/// (plus the [`Index`] impls required by the supertrait bounds); everything else
/// (length, element access, sub-slicing, iteration) is provided as default
/// methods.
///
/// The [`Index<Range<QI>>`] supertrait lets callers index with a quantized
/// range directly (`collection[start..end] -> &[V]`) instead of converting to
/// `usize` at every call site.
pub trait QuantizedContiguousTrait<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone>: Index<QI, Output = V> + Index<Range<QI>, Output = [V]> {
    /// Borrows the backing storage as a regular shared slice.
    fn as_slice(&self) -> &[V];

    /// Number of elements, expressed in the quantized index/count type.
    fn len(&self) -> QI {
        QI::quant_from_usize_unchecked(self.as_slice().len())
    }

    /// Returns `true` if there are no elements.
    fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// Copies out the element at `index`, or `None` if out of bounds.
    fn get(&self, index: QI) -> Option<V> {
        self.as_slice().get(index.quant_to_usize()).cloned()
    }

    /// Borrows the whole collection as a [`QuantizedContiguousSlice`] view.
    fn as_quantized_slice(&self) -> QuantizedContiguousSlice<'_, QI, QIC, V> {
        QuantizedContiguousSlice::new(self.as_slice())
    }

    /// Borrows a half-open sub-range as a [`QuantizedContiguousSlice`] view.
    ///
    /// Returns [`FeagiFailCollectionInvalidIndex`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice(&self, range: Range<QI>) -> Result<QuantizedContiguousSlice<'_, QI, QIC, V>, FeagiDataCollectionError> {
        match self.as_slice().get(range.start.quant_to_usize()..range.end.quant_to_usize()) {
            Some(slice) => Ok(QuantizedContiguousSlice::new(slice)),
            None => Err(FeagiFailCollectionInvalidIndex::new("subslice range is out of bounds").into()),
        }
    }

    /// Copies the internal data to a new owned vector structure
    fn clone_to_owned(&self) -> QuantizedContiguousVector<QI, QIC, V> {
        QuantizedContiguousVector::from_vec(self.as_slice().to_vec())
    }

    /// Iterates over shared references to the elements.
    fn iter(&self) -> core::slice::Iter<'_, V> {
        self.as_slice().iter()
    }

    /// Raw pointer to the first element. Valid for reads of [`Self::len`]
    /// elements for as long as `self` is borrowed.
    fn as_ptr(&self) -> *const V {
        self.as_slice().as_ptr()
    }

    /// Returns a shared reference to the element at `index`, without bounds
    /// checking.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.len()`).
    unsafe fn get_par(&self, index: QI) -> &V {
        &*self.as_ptr().add(index.quant_to_usize())
    }

    /// Rayon parallel iterator over shared references to the elements.
    #[cfg(feature = "use-rayon")]
    fn par_iter(&self) -> rayon::slice::Iter<'_, V>
    where
        V: Sync,
    {
        use rayon::iter::IntoParallelRefIterator;
        self.as_slice().par_iter()
    }
}

/// Shared, mutable behaviour for the contiguous quantized collections that own
/// or exclusively borrow their storage (the [`QuantizedContiguousVector`],
/// [`QuantizedContiguousSliceMut`] view, and [`QuantizedContiguousArray`]).
///
/// Implementors only need to expose their backing storage via
/// [`Self::as_mut_slice`].
pub trait QuantizedContiguousMutTrait<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone>:
    QuantizedContiguousTrait<QI, QIC, V> + IndexMut<QI, Output = V> + IndexMut<Range<QI>, Output = [V]>
{
    /// Mutably borrows the backing storage as a regular slice.
    fn as_mut_slice(&mut self) -> &mut [V];

    /// Mutably borrows the element at `index`, or `None` if out of bounds.
    fn get_mut(&mut self, index: QI) -> Option<&mut V> {
        self.as_mut_slice().get_mut(index.quant_to_usize())
    }

    /// Overwrites the element at `index`, returning the previous value if the
    /// index was in bounds (otherwise leaves the collection untouched).
    fn set(&mut self, index: QI, value: V) -> Option<V> {
        match self.as_mut_slice().get_mut(index.quant_to_usize()) {
            Some(slot) => {
                let previous = slot.clone();
                *slot = value;
                Some(previous)
            }
            None => None,
        }
    }

    /// Mutably borrows a half-open sub-range as a
    /// [`QuantizedContiguousSliceMut`] view.
    ///
    /// Returns [`FeagiFailCollectionInvalidIndex`] if `range` is out of bounds or its
    /// start is greater than its end (rather than panicking like `self[range]`).
    fn subslice_mut(&mut self, range: Range<QI>) -> Result<QuantizedContiguousSliceMut<'_, QI, QIC, V>, FeagiDataCollectionError> {
        match self.as_mut_slice().get_mut(range.start.quant_to_usize()..range.end.quant_to_usize()) {
            Some(slice) => Ok(QuantizedContiguousSliceMut::new(slice)),
            None => Err(FeagiFailCollectionInvalidIndex::new("subslice range is out of bounds").into()),
        }
    }

    /// Iterates over mutable references to the elements.
    fn iter_mut(&mut self) -> core::slice::IterMut<'_, V> {
        self.as_mut_slice().iter_mut()
    }

    /// Raw mutable pointer to the first element, derived from a shared `&self`.
    ///
    /// # Safety
    /// The returned pointer aliases the collection's storage; writes through it
    /// must only target indices not simultaneously accessed elsewhere.
    unsafe fn as_mut_ptr_par(&self) -> *mut V {
        self.as_ptr() as *mut V
    }

    /// Returns a mutable reference to the element at `index` through a shared
    /// `&self`, enabling parallel mutation of disjoint indices.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.len()`).
    /// - No other reference (shared or mutable) to the *same* element may exist
    ///   for the duration of the returned borrow. Concurrent callers must only
    ///   ever target disjoint indices.
    unsafe fn get_mut_par(&self, index: QI) -> &mut V {
        &mut *self.as_mut_ptr_par().add(index.quant_to_usize())
    }

    /// Rayon parallel iterator over mutable references to the elements.
    #[cfg(feature = "use-rayon")]
    fn par_iter_mut(&mut self) -> rayon::slice::IterMut<'_, V>
    where
        V: Send,
    {
        use rayon::iter::IntoParallelRefMutIterator;
        self.as_mut_slice().par_iter_mut()
    }
}


//region Vector

/// An owned, contiguous, heap-allocated run of quantized values.
pub struct QuantizedContiguousVector<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> {
    data: Vec<V>,
    phantom_data: PhantomData<(QI, QIC)>,
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousVector<QI, QIC, V> {
    pub fn new_empty() -> Self {
        Self {
            data: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    pub fn new_uniform(number_values: QI, filling_value: V) -> QuantizedContiguousVector<QI, QIC, V> {
        // TODO ensure length isnt 0!

        let values = vec![filling_value; number_values.quant_to_usize()];
        Self {
            data: values,
            phantom_data: PhantomData,
        }
    }

    /// Wraps an existing `Vec` without copying.
    pub fn from_vec(data: Vec<V>) -> QuantizedContiguousVector<QI, QIC, V> {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }

    /// Consumes the wrapper, returning the backing `Vec`.
    pub fn into_vec(self) -> Vec<V> {
        self.data
    }

    pub fn extend(&mut self, number_elements_to_extend: QI, extend_with: V) {
        // TODO set capacity?
        // TODO not this
        for _ in 0..number_elements_to_extend.quant_to_usize() {
            self.data.push(extend_with.clone());
        }
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousTrait<QI, QIC, V> for QuantizedContiguousVector<QI, QIC, V> {
    fn as_slice(&self) -> &[V] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousMutTrait<QI, QIC, V> for QuantizedContiguousVector<QI, QIC, V> {
    fn as_mut_slice(&mut self) -> &mut [V] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> Clone for QuantizedContiguousVector<QI, QIC, V> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> Index<QI> for QuantizedContiguousVector<QI, QIC, V> {
    type Output = V;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> IndexMut<QI> for QuantizedContiguousVector<QI, QIC, V> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> From<Vec<V>> for QuantizedContiguousVector<QI, QIC, V> {
    fn from(value: Vec<V>) -> Self {
        Self::from_vec(value)
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> From<QuantizedContiguousVector<QI, QIC, V>> for Vec<V> {
    fn from(value: QuantizedContiguousVector<QI, QIC, V>) -> Self {
        value.data
    }
}

impl_quantized_range_read_write!(
    QuantizedContiguousVector<QI, QIC, V>, QI, V,
    [QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone ]
);



//endregion

//region Array

/// An owned, contiguous, stack-allocated run of exactly `N` quantized values.
///
/// The compile-time length `N` is a `usize` const generic (Rust const generics
/// must be an integer type, so `QI` is retained only as the associated
/// index/count type used by the shared trait methods).
#[derive(Clone, Copy)]
pub struct QuantizedContiguousArray<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> {
    pub(crate) data: [V; N],
    phantom_data: PhantomData<(QI, QIC)>,
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> QuantizedContiguousArray<QI, QIC, V, N> {
    /// Builds an array with every element set to `filling_value`.
    pub fn new_uniform(filling_value: V) -> QuantizedContiguousArray<QI, QIC, V, N> {
        Self {
            data: core::array::repeat(filling_value),
            phantom_data: PhantomData,
        }
    }

    /// Wraps an existing array.
    pub fn from_array(data: [V; N]) -> QuantizedContiguousArray<QI, QIC, V, N> {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }

    /// Consumes the wrapper, returning the backing array.
    pub fn into_array(self) -> [V; N] {
        self.data
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> QuantizedContiguousTrait<QI, QIC, V> for QuantizedContiguousArray<QI, QIC, V, N> {
    fn as_slice(&self) -> &[V] {
        &self.data
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> QuantizedContiguousMutTrait<QI, QIC, V> for QuantizedContiguousArray<QI, QIC, V, N> {
    fn as_mut_slice(&mut self) -> &mut [V] {
        &mut self.data
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> Index<QI> for QuantizedContiguousArray<QI, QIC, V, N> {
    type Output = V;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> IndexMut<QI> for QuantizedContiguousArray<QI, QIC, V, N> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> From<[V; N]> for QuantizedContiguousArray<QI, QIC, V, N> {
    fn from(value: [V; N]) -> Self {
        Self::from_array(value)
    }
}

impl<QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone, const N: usize> From<QuantizedContiguousArray<QI, QIC, V, N>> for [V; N] {
    fn from(value: QuantizedContiguousArray<QI, QIC, V, N>) -> Self {
        value.data
    }
}

impl_quantized_range_read_write!(
    QuantizedContiguousArray<QI, QIC, V, N>, QI, V,
    [QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone , const N: usize]
);


//endregion

//region Slice

/// A borrowed, read-only view over a contiguous run of quantized values.
#[derive(Clone, Copy)]
pub struct QuantizedContiguousSlice<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> {
    pub(crate) data: &'a [V],
    phantom_data: PhantomData<(QI, QIC)>,
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousSlice<'a, QI, QIC, V> {
    /// Wraps an existing shared slice.
    pub fn new(data: &'a [V]) -> QuantizedContiguousSlice<'a, QI, QIC, V> {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }

    /// Returns the underlying shared slice, keeping the original lifetime.
    pub fn into_slice(self) -> &'a [V] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousTrait<QI, QIC, V> for QuantizedContiguousSlice<'a, QI, QIC, V> {
    fn as_slice(&self) -> &[V] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> Index<QI> for QuantizedContiguousSlice<'a, QI, QIC, V> {
    type Output = V;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> From<&'a [V]> for QuantizedContiguousSlice<'a, QI, QIC, V> {
    fn from(value: &'a [V]) -> Self {
        Self::new(value)
    }
}

impl_quantized_range_read!(
    QuantizedContiguousSlice<'a, QI, QIC, V>, QI, V,
    ['a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone ]
);


//endregion

//region Mut Slice

/// A borrowed, mutable view over a contiguous run of quantized values.
pub struct QuantizedContiguousSliceMut<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> {
    pub(crate) data: &'a mut [V],
    phantom_data: PhantomData<(QI, QIC)>,
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousSliceMut<'a, QI, QIC, V> {
    /// Wraps an existing mutable slice.
    pub fn new(data: &'a mut [V]) -> QuantizedContiguousSliceMut<'a, QI, QIC, V> {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }

    /// Returns the underlying mutable slice, keeping the original lifetime.
    pub fn into_slice_mut(self) -> &'a mut [V] {
        self.data
    }

    /// Creates a shorter-lived, exclusive re-borrow of this view.
    pub fn reborrow(&mut self) -> QuantizedContiguousSliceMut<'_, QI, QIC, V> {
        QuantizedContiguousSliceMut::new(self.data)
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousTrait<QI, QIC, V> for QuantizedContiguousSliceMut<'a, QI, QIC, V> {
    fn as_slice(&self) -> &[V] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> QuantizedContiguousMutTrait<QI, QIC, V> for QuantizedContiguousSliceMut<'a, QI, QIC, V> {
    fn as_mut_slice(&mut self) -> &mut [V] {
        self.data
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> Index<QI> for QuantizedContiguousSliceMut<'a, QI, QIC, V> {
    type Output = V;
    fn index(&self, index: QI) -> &Self::Output {
        &self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> IndexMut<QI> for QuantizedContiguousSliceMut<'a, QI, QIC, V> {
    fn index_mut(&mut self, index: QI) -> &mut Self::Output {
        &mut self.data[index.quant_to_usize()]
    }
}

impl<'a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone> From<&'a mut [V]> for QuantizedContiguousSliceMut<'a, QI, QIC, V> {
    fn from(value: &'a mut [V]) -> Self {
        Self::new(value)
    }
}

impl_quantized_range_read_write!(
    QuantizedContiguousSliceMut<'a, QI, QIC, V>, QI, V,
    ['a, QI: QuantizedUnsignedIntegerWrappedTrait, QIC: QuantizedUnsignedIntegerWrappedTrait, V: Clone ]
);

//endregion


