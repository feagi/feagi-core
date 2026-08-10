use crate::index_range_managers::feagi_index_range_manager_error::{
    FeagiIndexManagerInvalid, FeagiIndexManagerInvalidIndex, FeagiIndexManagerLimit, FeagiIndexRangeManagerError,
};
use crate::values::quantizable::QuantizedIndexCountTrait;

pub struct IndexManager<Q: QuantizedIndexCountTrait> {
    minimum_index: Q,
    maximum_index: Q,
    next_index: Q,
    skipped_indexes: Vec<Q>,
}

impl<Q: QuantizedIndexCountTrait> IndexManager<Q> {
    pub fn new(minimum_index: Q, maximum_index: Q, initial_number_indexes: Q) -> Result<IndexManager<Q>, FeagiIndexRangeManagerError> {
        if minimum_index > maximum_index {
            return Err(FeagiIndexManagerInvalid::new("Minimum index is greater than maximum index.").into());
        }

        if maximum_index - minimum_index < initial_number_indexes {
            return Err(FeagiIndexManagerInvalid::new("Initial number of indexes exceeds what range allows!").into());
        }

        Ok(Self {
            minimum_index,
            maximum_index,
            next_index: minimum_index + initial_number_indexes,
            skipped_indexes: vec![],
        })
    }

    pub fn get_next_index(&mut self) -> Result<Q, FeagiIndexRangeManagerError> {
        if let Some(i) = self.skipped_indexes.pop() {
            return Ok(i);
        }

        if self.next_index == self.maximum_index {
            return Err(FeagiIndexManagerLimit::new("Reached maximum index").into());
        }

        let i: Q = self.next_index;
        self.next_index += Q::QUANT_ONE;
        Ok(i)
    }

    pub fn return_index(&mut self, index: Q) -> Result<(), FeagiIndexRangeManagerError> {
        if index == self.minimum_index - Q::QUANT_ONE {
            self.minimum_index -= Q::QUANT_ONE;
            return Ok(());
        }

        if let Some(index) = self.skipped_indexes.iter().rposition(|&item| item == index) {
            self.skipped_indexes.swap_remove(index);
        }

        Err(FeagiIndexManagerInvalidIndex::new("Index not found.", index.quant_to_usize()).into())
    }
}
