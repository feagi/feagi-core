use core::iter::Iterator;
use core::ops::{Index, IndexMut};
use ahash::AHashMap;
use crate::quantization::QuantizableUIntType;
use crate::FeagiStructuresError;

#[derive(Debug, Clone)]
pub struct IndexedDataTracker<DataType, IndexType: QuantizableUIntType> {
    data: Vec<Option<DataType>>,
    skipped_indexes: Vec<IndexType>,
    index_length: IndexType,
}

impl<DataType, IndexType: QuantizableUIntType> IndexedDataTracker<DataType, IndexType> {
    
    /// Creates a new empty IndexedDataTracker
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            skipped_indexes: Vec::new(),
            index_length: IndexType::ZERO,
        }
    }

    /// Creates a new empty IndexedDataTracker with preallocated capacity
    pub fn with_capacity(capacity: IndexType) -> Self {
        Self {
            data: Vec::with_capacity(capacity.to_usize()),
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

    /// Inserts an element, reusing invalid slots or extending the vector
    /// Returns the index where the element was inserted
    pub fn insert_data_and_get_unique_index(&mut self, value: DataType) -> IndexType {

        let latest_skipped = self.skipped_indexes.pop();
        match latest_skipped {
            Some(index) => {
                self.data[index.to_usize()] = Some(value);
                index
            }
            None => {
                self.index_length += IndexType::ONE;
                self.data.push(Some(value));
                self.index_length - IndexType::ONE
            }
        }
    }


    /// Return an index to be able to be used later, invalidating data at that index as well
    pub fn return_index_and_invalidate_data(&mut self, returning_index: IndexType) -> Result<(), FeagiStructuresError> {
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

        self.data[returning_index.to_usize()] = None;
        Ok(())
    }

    
    /// Shrinks the capacity of the vector to fit its current size
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.skipped_indexes.shrink_to_fit();
    }

    /// Moves forward elements over any "skipped" spaces to ensure all elements are contiguous,
    ///and  returns a mapping table key'd by the initial index, mapped to the new index
    pub fn defragment_and_collapse(&mut self) -> Result<AHashMap<IndexType, IndexType>, FeagiStructuresError> {
        todo!()
    }
    
    /// Gets a reference to the element at the specified index (if valid)
    pub fn get(&self, index: IndexType) -> Option<&DataType> {
        self.data.get(index.to_usize()).and_then(|opt| opt.as_ref())
    }

    /// Gets a mutable reference to the element at the specified index (if valid)
    pub fn get_mut(&mut self, index: IndexType) -> Option<&mut DataType> {
        self.data.get_mut(index.to_usize()).and_then(|opt| opt.as_mut())
    }

    /// Returns an iterator over all valid elements
    pub fn iter(&self) -> impl Iterator<Item = &DataType> {
        self.data.iter().filter_map(|opt| opt.as_ref())
    }

    /// Returns a mutable iterator over all valid elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DataType> {
        self.data.iter_mut().filter_map(|opt| opt.as_mut())
    }

    /// Invalidates all indexes and data, without freeing capacity
    pub fn invalidate_all(&mut self) {
        self.data.clear();
        self.skipped_indexes.clear();
        self.index_length = IndexType::ZERO;
    }
    
}

impl<T, IndexType: QuantizableUIntType> Default for IndexedDataTracker<T, IndexType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, IndexType: QuantizableUIntType> Index<IndexType> for IndexedDataTracker<T, IndexType> {
    type Output = T;

    fn index(&self, index: IndexType) -> &Self::Output {
        self.get(index).expect("Index out of bounds or element invalid")
    }
}

impl<T, IndexType: QuantizableUIntType> IndexMut<IndexType> for  IndexedDataTracker<T, IndexType> {
    fn index_mut(&mut self, index: IndexType) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bounds or element invalid")
    }
}

// TODO iterators that skip Nones