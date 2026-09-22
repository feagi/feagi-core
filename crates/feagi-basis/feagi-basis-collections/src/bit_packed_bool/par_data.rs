use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};
use serde::{Deserialize, Serialize};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::bit_packed_bool::bit_batch_word::BitBatchWord;
use crate::generic_data::par_data::{ParDataStore, ParDataStoreMut};
use crate::feagi_collection_error::{BitBatchParDataInvalidRange, FeagiDataCollectionError};

/// Bit-packed bools held as words of `S`, addressed by word index `QWord` or bool index `QBit`.
///
/// `valid_bools` tracks how many trailing bits are padding rather than real data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitBatchParData<QWord, QBit, S> {
    pub(crate) words: S,
    pub(crate) valid_bools: QBit,
    pub(crate) _marker: PhantomData<QWord>,
}

#[cfg(feature = "alloc")]
pub type BitBatchParDataVector<QWord, QBit, Word> = BitBatchParData<QWord, QBit, Vec<Word>>;

pub type BitBatchParDataArray<QWord, QBit, Word, const N: usize> = BitBatchParData<QWord, QBit, [Word; N]>;

pub type BitBatchParDataSlice<'a, QWord, QBit, Word> = BitBatchParData<QWord, QBit, &'a [Word]>;

pub type BitBatchParDataSliceMut<'a, QWord, QBit, Word> = BitBatchParData<QWord, QBit, &'a mut [Word]>;

#[cfg(feature = "heapless")]
pub type BitBatchParDataHeaplessVec<QWord, QBit, Word, const N: usize> = BitBatchParData<QWord, QBit, ::heapless::Vec<Word, N>>;

//region Store-agnostic behaviour

impl<QWord, QBit, S> BitBatchParData<QWord, QBit, S> {
    /// Wraps a backing store and its valid-bool count, tagging it with `QWord`.
    pub const fn from_store(words: S, number_valid_bools: QBit) -> Self {
        Self { words, valid_bools: number_valid_bools, _marker: PhantomData }
    }

    /// Consumes the wrapper, returning the backing store.
    pub fn into_store(self) -> S {
        self.words
    }

    /// Borrows the backing store.
    pub fn store(&self) -> &S {
        &self.words
    }

    /// Mutably borrows the backing store.
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.words
    }
}

/// Shared behaviour
impl<QWord, QBit, S> BitBatchParData<QWord, QBit, S>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    S: ParDataStore,
    S::Elem: BitBatchWord,
{
    /// How many bools that are valid (IE not extra from padding) are included?
    pub fn number_valid_bools(&self) -> QBit {
        self.valid_bools
    }

    /// How many words are contained?
    pub fn number_words(&self) -> QWord {
        QWord::quant_from_usize_unchecked(self.words.store_as_slice().len())
    }

    /// Number of words in this collection.
    pub fn len(&self) -> QWord {
        self.number_words()
    }

    /// Returns `true` when this collection holds no words.
    pub fn is_empty(&self) -> bool {
        self.words.store_as_slice().is_empty()
    }

    /// Tries to get a word at a given index if it exists. Otherwise, returns none.
    pub fn get_word(&self, word_index: QWord) -> Option<&S::Elem> {
        self.words.store_as_slice().get(word_index.quant_to_usize())
    }

    /// Tries to get the holding word index if it exists (and is within padding margins).
    /// Otherwise, returns None
    pub fn get_word_index_from_bool_index(&self, bool_index: QBit) -> Option<QWord> {
        if bool_index > self.number_valid_bools() {
            return None;
        }
        Some(QWord::quant_from_usize_unchecked(
            bool_index.quant_to_usize() >> <S::Elem as BitBatchWord>::BIT_SHIFT_DISTANCE
        ))
    }

    /// From the collection bool index, get the index of the bool relevant to its word.
    /// Is unchecked.
    pub fn get_word_bit_index_from_bool_index(&self, bool_index: QBit) -> usize {
        bool_index.quant_to_usize() & (<S::Elem as BitBatchWord>::LOWER_BYTE_COUNT_BIT_MASK as usize)
    }

    /// Tries to get a bool / bit if it exists (and is within padding margins). Otherwise, returns
    /// none.
    pub fn get_bool(&self, bool_index: QBit) -> Option<bool> {
        let word_index = self.get_word_index_from_bool_index(bool_index)?;
        let local_bit_index = self.get_word_bit_index_from_bool_index(bool_index);
        let word = self.get_word(word_index)?;
        Some(word.get_bit(local_bit_index))
    }

    /// Borrows the whole collection as a [`BitBatchParDataSlice`] view.
    pub fn as_words_slice(&self) -> BitBatchParDataSlice<'_, QWord, QBit, S::Elem> {
        BitBatchParData::from_store(self.words.store_as_slice(), self.number_valid_bools())
    }

    /// Borrows a half-open word sub-range as a [`BitBatchParDataSlice`] view.
    pub fn subslice(&self, range: Range<QWord>) -> Result<BitBatchParDataSlice<'_, QWord, QBit, S::Elem>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.words.store_as_slice().get(start..end) {
            Some(slice) => {
                let range_valid_bools = self.number_valid_bools_for_word_range(start, end);
                Ok(BitBatchParData::from_store(slice, range_valid_bools))
            }
            None => Err(BitBatchParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }

    /// Calculates the number of words needed to hold a given number of bits
    /// while also ensuring a byte alignment (32, 64, 128, etc.).
    pub fn word_length_with_padding(number_bools: QBit) -> QWord {
        let needs_padding = (
            number_bools.quant_to_usize() & <S::Elem as BitBatchWord>::LOWER_BYTE_COUNT_BIT_MASK.quant_to_usize()) > 0;
        let word_count = number_bools.quant_to_usize() >> <S::Elem as BitBatchWord>::BIT_SHIFT_DISTANCE as usize;
        if needs_padding {
            QWord::quant_from_usize_unchecked(word_count + 1)
        } else {
            QWord::quant_from_usize_unchecked(word_count)
        }
    }

    /// Iterates over shared references to the words.
    pub fn iter_words(&self) -> core::slice::Iter<'_, S::Elem> {
        self.words.store_as_slice().iter()
    }

    /// Rayon parallel iterator over the words.
    #[cfg(feature = "expose_rayon")]
    pub fn par_iter_words(&self) -> rayon::slice::Iter<'_, S::Elem> {
        use rayon::iter::IntoParallelRefIterator;
        self.words.store_as_slice().par_iter()
    }

    /// Raw pointer to the first word.
    pub fn as_word_ptr(&self) -> *const S::Elem {
        self.words.store_as_slice().as_ptr()
    }

    /// Returns a shared reference to the word at `word_index` without bounds checks.
    pub unsafe fn get_word_par(&self, word_index: QWord) -> &S::Elem {
        debug_assert!(word_index.quant_to_usize() < self.words.store_as_slice().len());
        &*self.as_word_ptr().add(word_index.quant_to_usize())
    }

    /// Given the word index, gets the word without checking for bounds.
    pub unsafe fn get_word_unchecked(&self, word_index: QWord) -> &S::Elem {
        self.get_word_par(word_index)
    }

    /// Tries to get the holding word index, ignoring padding or internal data sizes
    pub unsafe fn get_word_index_from_bool_index_unchecked(&self, bool_index: QBit) -> QWord {
        debug_assert!(bool_index < self.number_valid_bools());
        QWord::quant_from_usize_unchecked(
            bool_index.quant_to_usize() >> <S::Elem as BitBatchWord>::BIT_SHIFT_DISTANCE)
    }

    /// Gets the bool without checking any bounds or padding
    pub unsafe fn get_bool_unchecked(&self, bool_index: QBit) -> bool {
        let word_index = self.get_word_index_from_bool_index_unchecked(bool_index);
        let local_bit_index = self.get_word_bit_index_from_bool_index(bool_index);
        let word = self.get_word_unchecked(word_index);
        word.get_bit(local_bit_index)
    }

    /// Calculates how many valid bools fall into a provided word-range.
    pub fn number_valid_bools_for_word_range(&self, start_word: usize, end_word: usize) -> QBit {
        let total_valid_bools = self.number_valid_bools().quant_to_usize();
        let bits_per_word = <S::Elem as BitBatchWord>::NUMBER_BITS as usize;
        let range_start_bool = start_word.saturating_mul(bits_per_word);
        let range_word_count = end_word.saturating_sub(start_word);
        let range_capacity_bools = range_word_count.saturating_mul(bits_per_word);
        let remaining_valid_bools = total_valid_bools.saturating_sub(range_start_bool);
        QBit::quant_from_usize_unchecked(core::cmp::min(remaining_valid_bools, range_capacity_bools))
    }
}

/// mutable
impl<QWord, QBit, S> BitBatchParData<QWord, QBit, S>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    S: ParDataStoreMut,
    S::Elem: BitBatchWord,
{
    /// Tries to get a mutable word at a given index if it exists. Otherwise, returns none.
    pub fn get_word_mut(&mut self, word_index: QWord) -> Option<&mut S::Elem> {
        self.words.store_as_mut_slice().get_mut(word_index.quant_to_usize())
    }

    /// Overwrites a word and returns the prior value when `word_index` is in bounds.
    pub fn set_word(&mut self, word_index: QWord, value: S::Elem) -> Option<S::Elem> {
        match self.get_word_mut(word_index) {
            Some(slot) => Some(core::mem::replace(slot, value)),
            None => None,
        }
    }

    /// Tries to get the mutable holding word for a bool index if it exists.
    pub fn get_word_from_bool_index_mut(&mut self, bool_index: QBit) -> Option<&mut S::Elem> {
        let word_index = self.get_word_index_from_bool_index(bool_index)?;
        self.get_word_mut(word_index)
    }

    /// Raw mutable pointer to the first word, derived from shared `&self`. Writes MUST target
    /// disjoint words that are not otherwise borrowed.
    pub unsafe fn as_word_mut_ptr_par(&self) -> *mut S::Elem {
        self.as_word_ptr() as *mut S::Elem
    }

    /// Returns a mutable reference to the word at `word_index` through shared `&self`.
    /// MUST NOT WRITE TO THE SAME WORD IN PARALLEL!
    pub unsafe fn get_word_mut_par(&self, word_index: QWord) -> &mut S::Elem {
        &mut *self.as_word_mut_ptr_par().add(word_index.quant_to_usize())
    }

    /// Gets a mutable word without checking for bounds.
    pub unsafe fn get_word_mut_unchecked(&mut self, word_index: QWord) -> &mut S::Elem {
        debug_assert!(word_index.quant_to_usize() < self.words.store_as_mut_slice().len());
        self.words.store_as_mut_slice().get_unchecked_mut(word_index.quant_to_usize())
    }

    /// Gets the mutable holding word for a bool index without any bounds checks.
    pub unsafe fn get_word_from_bool_index_mut_unchecked(&mut self, bool_index: QBit) -> &mut S::Elem {
        let word_index = self.get_word_index_from_bool_index_unchecked(bool_index);
        self.get_word_mut_unchecked(word_index)
    }

    /// Mutable iterator over all words.
    pub fn iter_words_mut(&mut self) -> core::slice::IterMut<'_, S::Elem> {
        self.words.store_as_mut_slice().iter_mut()
    }

    /// Rayon parallel mutable iterator over the words.
    #[cfg(feature = "expose_rayon")]
    pub fn par_iter_words_mut(&mut self) -> rayon::slice::IterMut<'_, S::Elem> {
        use rayon::iter::IntoParallelRefMutIterator;
        self.words.store_as_mut_slice().par_iter_mut()
    }

    /// Mutably borrows a half-open word sub-range as a [`BitBatchParDataSliceMut`] view.
    pub fn subslice_mut(&mut self, range: Range<QWord>) -> Result<BitBatchParDataSliceMut<'_, QWord, QBit, S::Elem>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        let range_valid_bools = self.number_valid_bools_for_word_range(start, end);
        match self.words.store_as_mut_slice().get_mut(start..end) {
            Some(slice) => Ok(BitBatchParData::from_store(slice, range_valid_bools)),
            None => Err(BitBatchParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }
}

impl<QWord, QBit, S> Index<QWord> for BitBatchParData<QWord, QBit, S>
where
    QWord: QuantizedUnsignedIntegerTrait,
    S: ParDataStore,
{
    type Output = S::Elem;

    fn index(&self, index: QWord) -> &Self::Output {
        &self.words.store_as_slice()[index.quant_to_usize()]
    }
}

impl<QWord, QBit, S> Index<Range<QWord>> for BitBatchParData<QWord, QBit, S>
where
    QWord: QuantizedUnsignedIntegerTrait,
    S: ParDataStore,
{
    type Output = [S::Elem];

    fn index(&self, range: Range<QWord>) -> &Self::Output {
        &self.words.store_as_slice()[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

impl<QWord, QBit, S> IndexMut<QWord> for BitBatchParData<QWord, QBit, S>
where
    QWord: QuantizedUnsignedIntegerTrait,
    S: ParDataStoreMut,
{
    fn index_mut(&mut self, index: QWord) -> &mut Self::Output {
        &mut self.words.store_as_mut_slice()[index.quant_to_usize()]
    }
}

impl<QWord, QBit, S> IndexMut<Range<QWord>> for BitBatchParData<QWord, QBit, S>
where
    QWord: QuantizedUnsignedIntegerTrait,
    S: ParDataStoreMut,
{
    fn index_mut(&mut self, range: Range<QWord>) -> &mut Self::Output {
        &mut self.words.store_as_mut_slice()[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

//endregion

//region Vector

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> BitBatchParData<QWord, QBit, Vec<Word>>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Builds a padded word vector for `number_valid_bools`, with every word set to `initial_word`.
    pub fn new_uniform(number_valid_bools: QBit, initial_word: Word) -> Self {
        let word_count = number_valid_bools.quant_to_usize().div_ceil(Word::NUMBER_BITS as usize);
        Self::from_store(vec![initial_word; word_count], number_valid_bools)
    }

    /// Wraps an existing word vector and valid-bool count.
    pub fn from_vec(words: Vec<Word>, number_valid_bools: QBit) -> Self {
        Self::from_store(words, number_valid_bools)
    }

    /// Consumes this wrapper and returns the backing vector.
    pub fn into_vec(self) -> Vec<Word> {
        self.words
    }
}

//endregion

//region Slice

impl<'a, QWord, QBit, Word> BitBatchParData<QWord, QBit, &'a [Word]>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Wraps an existing shared bit-packed slice and valid-bool count.
    pub const fn new(words: &'a [Word], number_valid_bools: QBit) -> Self {
        Self::from_store(words, number_valid_bools)
    }

    /// Returns the underlying shared slice.
    pub fn into_slice(self) -> &'a [Word] {
        self.words
    }
}

//endregion

//region Mut Slice

impl<'a, QWord, QBit, Word> BitBatchParData<QWord, QBit, &'a mut [Word]>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Wraps an existing mutable bit-packed slice and valid-bool count.
    pub const fn new(words: &'a mut [Word], number_valid_bools: QBit) -> Self {
        Self::from_store(words, number_valid_bools)
    }

    /// Returns the underlying mutable slice.
    pub fn into_slice_mut(self) -> &'a mut [Word] {
        self.words
    }

    /// Creates a shorter-lived exclusive reborrow for nested mutation flows.
    pub fn reborrow(&mut self) -> BitBatchParDataSliceMut<'_, QWord, QBit, Word> {
        BitBatchParData::from_store(&mut *self.words, self.valid_bools)
    }
}

//endregion

//region Array

impl<QWord, QBit, Word, const N: usize> BitBatchParData<QWord, QBit, [Word; N]>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Builds a fixed-size array where each word is `initial_word`.
    pub fn new_uniform(initial_word: Word, number_valid_bools: QBit) -> Self {
        Self::from_store([initial_word; N], number_valid_bools)
    }

    /// Wraps an existing word array and valid-bool count.
    pub fn from_array(words: [Word; N], number_valid_bools: QBit) -> Self {
        Self::from_store(words, number_valid_bools)
    }

    /// Consumes this wrapper and returns the backing array.
    pub fn into_array(self) -> [Word; N] {
        self.words
    }
}

//endregion

//region Heapless Vector

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> BitBatchParData<QWord, QBit, ::heapless::Vec<Word, N>>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType=QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Builds a capacity-`N` heapless vector with each word set to `initial_word`.
    pub fn new_uniform(initial_word: Word, number_valid_bools: QBit) -> Self {
        Self::from_store(::heapless::Vec::from_array([initial_word; N]), number_valid_bools)
    }

    /// Wraps an existing array into the backing heapless vector.
    pub fn from_array(words: [Word; N], number_valid_bools: QBit) -> Self {
        Self::from_store(::heapless::Vec::from_array(words), number_valid_bools)
    }
}

//endregion
