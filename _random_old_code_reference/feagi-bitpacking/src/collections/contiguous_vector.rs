use crate::bitpacking_backends::BitPacked;



/// Contains a boolean bitpacked vector of certain length, using some quantization of bitpacking.
/// Bits are stored in a Packable, which is also referred to as a "Word"
pub struct BitPackedContiguousBoolVector<Packable: BitPacked> {
    words: Vec<Packable>,
}

impl<Packable: BitPacked> BitPackedContiguousBoolVector<Packable> {
    pub fn new(number_bools_to_store: usize) -> BitPackedContiguousBoolVector<Packable> {
        // TODO error if 0?

        let number_words: usize;
        if (number_bools_to_store % (Packable::BIT_PACKING_BACKEND as usize)) == 0
        {
            number_words = number_bools_to_store / (Packable::BIT_PACKING_BACKEND as usize);
        } else {
            number_words = 1 + (number_bools_to_store / (Packable::BIT_PACKING_BACKEND as usize));
        }

        let words: Vec<Packable> = vec![Packable::new(); number_words];
        BitPackedContiguousBoolVector { words }
    }



    pub fn get_word(&self, word_index: usize) -> Option<&Packable> {
        self.words.get(word_index)
    }

    pub fn get_word_unchecked(&self, word_index: usize) -> Packable {
        self.words[word_index]
    }

    pub fn get_bool(&self, bool_index: usize) -> Option<bool> {
        let word = self.get_word(bool_index / Packable::NUMBER_BITS)?;
        word.get_at((bool_index % Packable::NUMBER_BITS) as u8)
    }



    pub fn get_word_mut(&mut self, word_index: usize) -> Option<&mut Packable> {
        self.words.get_mut(word_index)
    }




    pub fn set_word(&mut self, word_index: usize, word: Packable) -> Option<()> {
        let mut_word =self.get_word_mut(word_index)?;
        *mut_word = word;
        Some(())
    }



    pub fn set_word_unchecked(&mut self, word_index: usize, word: Packable) {
        self.words[word_index] = word;
    }


    pub fn set_bool(&mut self, bool_index: usize, boolean: bool) -> Option<()> {
        let word = self.get_word_mut(bool_index / Packable::NUMBER_BITS)?;
        word.set_at((bool_index % Packable::NUMBER_BITS) as u8, boolean)
    }

    pub fn make_range_from_word_index(&self, word_index: usize, number_bools_stored: usize) -> core::ops::Range<usize> {
        core::cmp::min( word_index * Packable::NUMBER_BITS, number_bools_stored)
            .. core::cmp::min( (word_index + 1) * Packable::NUMBER_BITS, number_bools_stored)
    }

}



