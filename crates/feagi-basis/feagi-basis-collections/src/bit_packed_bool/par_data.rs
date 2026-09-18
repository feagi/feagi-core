use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Range};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::bit_packed_bool::bit_batch_word::BitBatchWord;
use crate::feagi_collection_error::{BitBatchParDataInvalidRange, FeagiDataCollectionError};

pub trait BitBatchParData<QWord, QBit, Word>:
Index<QWord, Output = Word> + Index<Range<QWord>, Output = [Word]>
+ core::fmt::Debug
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Borrows the backing storage as a regular shared bit packed slice.
    fn as_words(&self) -> &[Word];

    /// How many bools that are valid (IE not extra from padding) are included?
    fn number_valid_bools(&self) -> QBit;

    //region Default Impls
    /// How many words are contained?
    fn number_words(&self) -> QWord {
        QWord::quant_from_usize_unchecked(self.as_words().len())
    }

    /// Number of words in this collection.
    fn len(&self) -> QWord {
        self.number_words()
    }

    /// Returns `true` when this collection holds no words.
    fn is_empty(&self) -> bool {
        self.as_words().is_empty()
    }

    /// Tries to get a word at a given index if it exists. Otherwise, returns none.
    fn get_word(&self, word_index: QWord) -> Option<&Word> {
        self.as_words().get(word_index.quant_to_usize())
    }

    /// Tries to get the holding word index if it exists (and is within padding margins).
    /// Otherwise, returns None
    fn get_word_index_from_bool_index(&self, bool_index: QBit) -> Option<QWord> {
        if bool_index > self.number_valid_bools() {
            return None;
        }
        Some(QWord::quant_from_usize_unchecked(
            bool_index.quant_to_usize() >> Word::BIT_SHIFT_DISTANCE
        ))
    }
    
    /// From the collection bool index, get the index of the bool relevant to its word. 
    /// Is unchecked.
    fn get_word_bit_index_from_bool_index(&self, bool_index: QBit) -> usize {
        bool_index.quant_to_usize() & (Word::LOWER_BYTE_COUNT_BIT_MASK as usize)
    }

    /// Tries to get a bool / bit if it exists (and is within padding margins). Otherwise, returns
    /// none.
    fn get_bool(&self, bool_index: QBit) -> Option<bool> {
        let word_index = self.get_word_index_from_bool_index(bool_index)?;
        let local_bit_index = self.get_word_bit_index_from_bool_index(bool_index);
        let word = self.get_word(word_index)?;
        Some(word.get_bit(local_bit_index))
    }

    /// Borrows the whole collection as a [`BitBatchParDataSlice`] view.
    fn as_words_slice(&self) -> BitBatchParDataSlice<'_, QWord, QBit, Word> {
        BitBatchParDataSlice::new(self.as_words(), self.number_valid_bools())
    }

    /// Borrows a half-open word sub-range as a [`BitBatchParDataSlice`] view.
    ///
    /// Returns [`BitBatchParDataInvalidRange`] if `range` is out of bounds.
    fn subslice(&self, range: Range<QWord>) -> Result<BitBatchParDataSlice<'_, QWord, QBit, Word>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        match self.as_words().get(start..end) {
            Some(slice) => {
                let range_valid_bools = self.number_valid_bools_for_word_range(start, end);
                Ok(BitBatchParDataSlice::new(slice, range_valid_bools))
            }
            None => Err(BitBatchParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }

    /// Calculates the number of words needed to hold a given number of bits
    /// while also ensuring a byte alignment (32, 64, 128, etc.).
    fn word_length_with_padding(number_bools: QBit) -> QWord {
        let needs_padding = (
            number_bools.quant_to_usize() & Word::LOWER_BYTE_COUNT_BIT_MASK.quant_to_usize()) > 0;
        let word_count = number_bools.quant_to_usize() >> Word::BIT_SHIFT_DISTANCE as usize;
        if needs_padding {
            QWord::quant_from_usize_unchecked(word_count)
        } else {
            QWord::quant_from_usize_unchecked(word_count + 1)
        }
    }
    
    fn iter_words(&self) -> core::slice::Iter<'_, Word> {
        self.as_words().iter()
    }

    #[cfg(feature = "expose_rayon")]
    fn par_iter_words(&self) -> rayon::slice::Iter<'_, Word> {
        use rayon::iter::IntoParallelRefIterator;
        self.as_words().par_iter()
    }
    
    /// Raw pointer to the first word.
    fn as_word_ptr(&self) -> *const Word {
        self.as_words().as_ptr()
    }

    /// Returns a shared reference to the word at `word_index` without bounds checks.
    ///
    /// # Safety
    /// - `word_index` must be in bounds (`word_index < self.number_words()`).
    unsafe fn get_word_par(&self, word_index: QWord) -> &Word {
        debug_assert!(word_index.quant_to_usize() < self.as_words().len());
        &*self.as_word_ptr().add(word_index.quant_to_usize())
    }
    
    /// Given the word index, gets the word without checking for bounds.
    unsafe fn get_word_unchecked(&self, word_index: QWord) -> &Word {
        self.get_word_par(word_index)
    }

    /// Tries to get the holding word index, ignoring padding or internal data sizes
    unsafe fn get_word_index_from_bool_index_unchecked(&self, bool_index: QBit) -> QWord {
        debug_assert!(bool_index < self.number_valid_bools());
        QWord::quant_from_usize_unchecked(
            bool_index.quant_to_usize() >> Word::BIT_SHIFT_DISTANCE)
    }

    /// Gets the bool without checking any bounds or padding
    unsafe fn get_bool_unchecked(&self, bool_index: QBit) -> bool {
        let word_index = self.get_word_index_from_bool_index_unchecked(bool_index);
        let local_bit_index = self.get_word_bit_index_from_bool_index(bool_index);
        let word = self.get_word_unchecked(word_index);
        word.get_bit(local_bit_index)
    }

    /// Calculates how many valid bools fall into a provided word-range.
    fn number_valid_bools_for_word_range(&self, start_word: usize, end_word: usize) -> QBit {
        let total_valid_bools = self.number_valid_bools().quant_to_usize();
        let bits_per_word = Word::NUMBER_BITS as usize;
        let range_start_bool = start_word.saturating_mul(bits_per_word);
        let range_word_count = end_word.saturating_sub(start_word);
        let range_capacity_bools = range_word_count.saturating_mul(bits_per_word);
        let remaining_valid_bools = total_valid_bools.saturating_sub(range_start_bool);
        QBit::quant_from_usize_unchecked(core::cmp::min(remaining_valid_bools, range_capacity_bools))
    }

    //endregion
}

pub trait BitBatchParDataMut<QWord, QBit, Word>:
BitBatchParData<QWord, QBit, Word>
+ IndexMut<QWord, Output = Word>
+ IndexMut<Range<QWord>, Output = [Word]>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Borrows the backing storage as a regular shared bit packed mut slice.
    fn as_words_mut(&mut self) -> &mut [Word];

    //region Default impls
    /// Tries to get a mutable word at a given index if it exists. Otherwise, returns none.
    fn get_word_mut(&mut self, word_index: QWord) -> Option<&mut Word> {
        self.as_words_mut().get_mut(word_index.quant_to_usize())
    }

    /// Overwrites a word and returns the prior value when `word_index` is in bounds.
    fn set_word(&mut self, word_index: QWord, value: Word) -> Option<Word> {
        match self.get_word_mut(word_index) {
            Some(slot) => Some(core::mem::replace(slot, value)),
            None => None,
        }
    }

    /// Tries to get the mutable holding word for a bool index if it exists.
    fn get_word_from_bool_index_mut(&mut self, bool_index: QBit) -> Option<&mut Word> {
        let word_index = self.get_word_index_from_bool_index(bool_index)?;
        self.get_word_mut(word_index)
    }

    /// Raw mutable pointer to the first word, derived from shared `&self`.
    ///
    /// # Safety
    /// Writes through the returned pointer must target disjoint words that are
    /// not otherwise borrowed for the duration of use.
    unsafe fn as_word_mut_ptr_par(&self) -> *mut Word {
        self.as_word_ptr() as *mut Word
    }

    /// Returns a mutable reference to the word at `word_index` through shared `&self`.
    ///
    /// # Safety
    /// - `word_index` must be in bounds (`word_index < self.number_words()`).
    /// - No other reference to the same word may exist for the returned borrow.
    unsafe fn get_word_mut_par(&self, word_index: QWord) -> &mut Word {
        &mut *self.as_word_mut_ptr_par().add(word_index.quant_to_usize())
    }

    /// Gets a mutable word without checking for bounds.
    unsafe fn get_word_mut_unchecked(&mut self, word_index: QWord) -> &mut Word {
        debug_assert!(word_index.quant_to_usize() < self.as_words_mut().len());
        self.as_words_mut().get_unchecked_mut(word_index.quant_to_usize())
    }

    /// Gets the mutable holding word for a bool index without any bounds checks.
    unsafe fn get_word_from_bool_index_mut_unchecked(&mut self, bool_index: QBit) -> &mut Word {
        let word_index = self.get_word_index_from_bool_index_unchecked(bool_index);
        self.get_word_mut_unchecked(word_index)
    }

    /// Mutable iterator over all words.
    fn iter_words_mut(&mut self) -> core::slice::IterMut<'_, Word> {
        self.as_words_mut().iter_mut()
    }

    #[cfg(feature = "expose_rayon")]
    fn par_iter_words_mut(&mut self) -> rayon::slice::IterMut<'_, Word> {
        use rayon::iter::IntoParallelRefMutIterator;
        self.as_words_mut().par_iter_mut()
    }

    /// Mutably borrows a half-open word sub-range as a [`BitBatchParDataSliceMut`] view.
    fn subslice_mut(&mut self, range: Range<QWord>) -> Result<BitBatchParDataSliceMut<'_, QWord, QBit, Word>, FeagiDataCollectionError> {
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        let range_valid_bools = self.number_valid_bools_for_word_range(start, end);
        match self.as_words_mut().get_mut(start..end) {
            Some(slice) => Ok(BitBatchParDataSliceMut::new(slice, range_valid_bools)),
            None => Err(BitBatchParDataInvalidRange::new("subslice range is out of bounds", start, end).into()),
        }
    }
    
    //endregion
}

//region Implementations

//region Vector

/// Owned, heap-allocated bit-packed words paired with explicit valid-bool count.
#[cfg(feature = "alloc")]
#[derive(Debug, serde::Serialize)]
pub struct BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    pub(crate) words: Vec<Word>,
    pub(crate) valid_bools: QBit,
    pub(crate) _marker: PhantomData<QWord>,
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Builds a padded word vector for `number_valid_bools`, with every word set to `initial_word`.
    pub fn new_uniform(number_valid_bools: QBit, initial_word: Word) -> Self {
        let word_count = number_valid_bools.quant_to_usize().div_ceil(Word::NUMBER_BITS as usize);
        Self {
            words: vec![initial_word; word_count],
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Wraps an existing word vector and valid-bool count.
    pub fn from_vec(words: Vec<Word>, number_valid_bools: QBit) -> Self {
        Self {
            words,
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Consumes this wrapper and returns the backing vector.
    pub fn into_vec(self) -> Vec<Word> {
        self.words
    }
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> BitBatchParData<QWord, QBit, Word> for BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words(&self) -> &[Word] {
        &self.words
    }

    fn number_valid_bools(&self) -> QBit {
        self.valid_bools
    }
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> BitBatchParDataMut<QWord, QBit, Word> for BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words_mut(&mut self) -> &mut [Word] {
        &mut self.words
    }
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> Index<QWord> for BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = Word;

    fn index(&self, index: QWord) -> &Self::Output {
        &self.words[index.quant_to_usize()]
    }
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> Index<Range<QWord>> for BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = [Word];

    fn index(&self, range: Range<QWord>) -> &Self::Output {
        &self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> IndexMut<QWord> for BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, index: QWord) -> &mut Self::Output {
        &mut self.words[index.quant_to_usize()]
    }
}

#[cfg(feature = "alloc")]
impl<QWord, QBit, Word> IndexMut<Range<QWord>> for BitBatchParDataVector<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, range: Range<QWord>) -> &mut Self::Output {
        &mut self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

//endregion

//region Slice

/// Shared view over bit-packed words with explicit valid-bool count.
#[derive(Debug, serde::Serialize)]
pub struct BitBatchParDataSlice<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    pub(crate) words: &'a [Word],
    pub(crate) valid_bools: QBit,
    pub(crate) _marker: PhantomData<QWord>,
}

impl<'a, QWord, QBit, Word> BitBatchParDataSlice<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Wraps an existing shared bit-packed slice and valid-bool count.
    pub fn new(words: &'a [Word], number_valid_bools: QBit) -> Self {
        Self {
            words,
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying shared slice.
    pub fn into_slice(self) -> &'a [Word] {
        self.words
    }
}

impl<'a, QWord, QBit, Word> BitBatchParData<QWord, QBit, Word> for BitBatchParDataSlice<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words(&self) -> &[Word] {
        self.words
    }

    fn number_valid_bools(&self) -> QBit {
        self.valid_bools
    }
}

impl<'a, QWord, QBit, Word> Index<QWord> for BitBatchParDataSlice<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = Word;

    fn index(&self, index: QWord) -> &Self::Output {
        &self.words[index.quant_to_usize()]
    }
}

impl<'a, QWord, QBit, Word> Index<Range<QWord>> for BitBatchParDataSlice<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = [Word];

    fn index(&self, range: Range<QWord>) -> &Self::Output {
        &self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

//endregion

//region Mut Slice

/// Mutable view over bit-packed words with explicit valid-bool count.
#[derive(Debug, serde::Serialize)]
pub struct BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    pub(crate) words: &'a mut [Word],
    pub(crate) valid_bools: QBit,
    pub(crate) _marker: PhantomData<QWord>,
}

impl<'a, QWord, QBit, Word> BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Wraps an existing mutable bit-packed slice and valid-bool count.
    pub fn new(words: &'a mut [Word], number_valid_bools: QBit) -> Self {
        Self {
            words,
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying mutable slice.
    pub fn into_slice_mut(self) -> &'a mut [Word] {
        self.words
    }

    /// Creates a shorter-lived exclusive reborrow for nested mutation flows.
    pub fn reborrow(&mut self) -> BitBatchParDataSliceMut<'_, QWord, QBit, Word> {
        BitBatchParDataSliceMut::new(self.words, self.valid_bools)
    }
}

impl<'a, QWord, QBit, Word> BitBatchParData<QWord, QBit, Word> for BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words(&self) -> &[Word] {
        self.words
    }

    fn number_valid_bools(&self) -> QBit {
        self.valid_bools
    }
}

impl<'a, QWord, QBit, Word> BitBatchParDataMut<QWord, QBit, Word> for BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words_mut(&mut self) -> &mut [Word] {
        self.words
    }
}

impl<'a, QWord, QBit, Word> Index<QWord> for BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = Word;

    fn index(&self, index: QWord) -> &Self::Output {
        &self.words[index.quant_to_usize()]
    }
}

impl<'a, QWord, QBit, Word> Index<Range<QWord>> for BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = [Word];

    fn index(&self, range: Range<QWord>) -> &Self::Output {
        &self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

impl<'a, QWord, QBit, Word> IndexMut<QWord> for BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, index: QWord) -> &mut Self::Output {
        &mut self.words[index.quant_to_usize()]
    }
}

impl<'a, QWord, QBit, Word> IndexMut<Range<QWord>> for BitBatchParDataSliceMut<'a, QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, range: Range<QWord>) -> &mut Self::Output {
        &mut self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

//endregion

//region Array

/// Owned, stack-allocated bit-packed words with explicit valid-bool count.
#[derive(Debug, serde::Serialize)]
#[serde(bound(serialize = "[Word; N]: ::serde::Serialize"))]
pub struct BitBatchParDataArray<QWord, QBit, Word, const N: usize>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    pub(crate) words: [Word; N],
    pub(crate) valid_bools: QBit,
    pub(crate) _marker: PhantomData<QWord>,
}

impl<QWord, QBit, Word, const N: usize> BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Builds a fixed-size array where each word is `initial_word`.
    pub fn new_uniform(initial_word: Word, number_valid_bools: QBit) -> Self {
        Self {
            words: [initial_word; N],
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Wraps an existing word array and valid-bool count.
    pub fn from_array(words: [Word; N], number_valid_bools: QBit) -> Self {
        Self {
            words,
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Consumes this wrapper and returns the backing array.
    pub fn into_array(self) -> [Word; N] {
        self.words
    }
}

impl<QWord, QBit, Word, const N: usize> BitBatchParData<QWord, QBit, Word> for BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words(&self) -> &[Word] {
        &self.words
    }

    fn number_valid_bools(&self) -> QBit {
        self.valid_bools
    }
}

impl<QWord, QBit, Word, const N: usize> BitBatchParDataMut<QWord, QBit, Word> for BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words_mut(&mut self) -> &mut [Word] {
        &mut self.words
    }
}

impl<QWord, QBit, Word, const N: usize> Index<QWord> for BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = Word;

    fn index(&self, index: QWord) -> &Self::Output {
        &self.words[index.quant_to_usize()]
    }
}

impl<QWord, QBit, Word, const N: usize> Index<Range<QWord>> for BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = [Word];

    fn index(&self, range: Range<QWord>) -> &Self::Output {
        &self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

impl<QWord, QBit, Word, const N: usize> IndexMut<QWord> for BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, index: QWord) -> &mut Self::Output {
        &mut self.words[index.quant_to_usize()]
    }
}

impl<QWord, QBit, Word, const N: usize> IndexMut<Range<QWord>> for BitBatchParDataArray<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, range: Range<QWord>) -> &mut Self::Output {
        &mut self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

//endregion

//region Heapless Vector

#[cfg(feature = "heapless")]
/// Owned, stack-backed word vector with fixed capacity.
#[derive(Debug, serde::Serialize)]
pub struct BitBatchParDataHeaplessVec<QWord, QBit, Word, const N: usize>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    pub(crate) words: ::heapless::Vec<Word, N>,
    pub(crate) valid_bools: QBit,
    pub(crate) _marker: PhantomData<QWord>,
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Builds a capacity-`N` heapless vector with each word set to `initial_word`.
    pub fn new_uniform(initial_word: Word, number_valid_bools: QBit) -> Self {
        Self {
            words: ::heapless::Vec::from_array([initial_word; N]),
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }

    /// Wraps an existing array into the backing heapless vector.
    pub fn from_array(words: [Word; N], number_valid_bools: QBit) -> Self {
        Self {
            words: ::heapless::Vec::from_array(words),
            valid_bools: number_valid_bools,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> BitBatchParData<QWord, QBit, Word>
for BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words(&self) -> &[Word] {
        &self.words
    }

    fn number_valid_bools(&self) -> QBit {
        self.valid_bools
    }
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> BitBatchParDataMut<QWord, QBit, Word>
for BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn as_words_mut(&mut self) -> &mut [Word] {
        &mut self.words
    }
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> Index<QWord> for BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = Word;

    fn index(&self, index: QWord) -> &Self::Output {
        &self.words[index.quant_to_usize()]
    }
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> Index<Range<QWord>> for BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    type Output = [Word];

    fn index(&self, range: Range<QWord>) -> &Self::Output {
        &self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> IndexMut<QWord> for BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, index: QWord) -> &mut Self::Output {
        &mut self.words[index.quant_to_usize()]
    }
}

#[cfg(feature = "heapless")]
impl<QWord, QBit, Word, const N: usize> IndexMut<Range<QWord>> for BitBatchParDataHeaplessVec<QWord, QBit, Word, N>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    fn index_mut(&mut self, range: Range<QWord>) -> &mut Self::Output {
        &mut self.words[range.start.quant_to_usize()..range.end.quant_to_usize()]
    }
}

//endregion

//endregion