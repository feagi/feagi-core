use ahash::AHashMap;
use crate::quantization::QuantizableUIntType;
use crate::FeagiStructuresError;
// TODO move some checks to work only under debug

/// Tracks the next best usable index, should a user nullify an earlier one
pub struct IndexTracker<IndexType: QuantizableUIntType> {
    skipped_indexes: Vec<IndexType>,
    index_length: IndexType
}

impl<IndexType: QuantizableUIntType> IndexTracker<IndexType> {
    pub fn new() -> Self {
        IndexTracker {
            skipped_indexes: Vec::new(),
            index_length: IndexType::ZERO,
        }
    }

    /// Returns number of indexes skipped internally
    pub fn get_number_skipped_indexes(&self) -> usize {
        self.skipped_indexes.len()
    }

    /// Returns number of unique indexes that should be in circulation
    pub fn get_number_generated_indexes(&self) -> usize {
        self.index_length.to_usize() - self.skipped_indexes.len()
    }

    /// Grabs an Index that is unique
    pub fn get_unique_index(&mut self) -> IndexType {
        let latest_skipped = self.skipped_indexes.pop();
        match latest_skipped {
            Some(index) => index,
            None => {
                self.index_length += IndexType::ONE;
                self.index_length - IndexType::ONE
            }
        }
    }

    /// Return an index to be able to be used later
    pub fn return_index(&mut self, returning_index: IndexType) -> Result<(), FeagiStructuresError> {

        if returning_index >= self.index_length {
            // Not possible
            return Err(FeagiStructuresError::InvalidValue { context: "returned unique Index is larger than largest possible checked out!" })
        }

        // User tried checking in index 0 when nothing was checked out
        if self.index_length == IndexType::ZERO {
            return Err(FeagiStructuresError::InvalidValue { context: "Tried returning an index when no indexes were checked out!" })
        }


        if returning_index == self.index_length - IndexType::ONE {
            self.index_length -= IndexType::ONE;
        }
        else {
            // TODO DEBUG ONLY CHECK: Make sure we arent returning the same index multiple times!
            self.skipped_indexes.push(returning_index);
        }

        Ok(())
    }

    /// Free any spare memory from the skipped indexes vector
    pub fn shrink_to_fit(&mut self) {
        self.skipped_indexes.shrink_to_fit();
    }

    /// Moves forward elements over any "skipped" spaces to ensure all elements are contiguous,
    ///and  returns a mapping table key'd by the initial index, mapped to the new index
    pub fn defragment_and_collapse(&mut self) -> Result<AHashMap<IndexType, IndexType>, FeagiStructuresError> {
        todo!()
    }

    /// Invalidates all indexes, without freeing capacity
    pub fn invalidate_all(&mut self) {
        self.skipped_indexes.clear();
        self.index_length = IndexType::ZERO;
    }
}

impl<IndexType: QuantizableUIntType> Default for IndexTracker<IndexType> {
    fn default() -> Self {
        Self::new()
    }
}

// TODO Iterators that skip invalids