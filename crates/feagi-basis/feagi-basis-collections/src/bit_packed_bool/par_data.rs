use std::ops::{Index, Range};
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::bit_packed_bool::bit_batch_word::BitBatchWord;

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

    /// How many words are contained?
    fn number_words(&self) -> QWord {
        QWord::quant_from_usize_unchecked(self.as_words().len())
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
    fn iter_words_rayon(&self) -> rayon::slice::Iter<'_, Word> {
        use rayon::iter::IntoParallelRefIterator;
        self.as_words().par_iter()
    }
    
    /// Get raw pointer to first pointer
    fn get_first_word_ptr(&self) -> *const Word {
        self.as_words().as_ptr()
    }
    
    /// Given the word index, gets the word without checking for bounds
    unsafe fn get_word_unchecked(&self, word_index: QWord) -> &Word {
        debug_assert!(word_index.quant_to_usize() < self.as_words().len());
        self.as_words().get_unchecked(word_index.quant_to_usize())
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
    
    /// Raw mutable pointer to the first word
    unsafe fn get_first_word_mut_ptr(&self) -> *mut Word {
        self.get_first_word_ptr() as *mut Word
    }
    
    /// Get a parallelizable mut reference to a word. Make sure 2 things do not access this at once!
    unsafe fn get_par_mut_word(&self, word_index: QWord) -> &mut Word {
        &mut *self.get_first_word_mut_ptr().add(word_index.quant_to_usize())
    }
    
    // Can't do the same for bools as we cant give out single bits at a time!
    
}

pub trait BitBatchParDataMut<QWord, QBit, Word>: BitBatchParData<QWord, QBit, Word>
where
    QWord: QuantizedUnsignedIntegerTrait,
    QBit: QuantizedUnsignedIntegerTrait<QuantType = QWord::QuantType>,
    Word: BitBatchWord,
{
    /// Borrows the backing storage as a regular shared bit packed mut slice.
    fn as_words_mut(&mut self) -> &mut [Word];
    
    
}