use feagi_basis_quantization::values::quantizable::QuantizedUnsignedIntegerTrait;
use crate::generic_collections::feagi_index_organizer_error::IndexOrganizerError;

pub struct IndexManager<Q: QuantizedUnsignedIntegerTrait> {
    minimum_index: Q,
    maximum_index: Q,
    next_index: Q,
    skipped_indexes: Vec<Q>,
}

impl<Q: QuantizedUnsignedIntegerTrait> IndexManager<Q> {
    pub fn new(minimum_index: Q, maximum_index: Q, initial_number_indexes: Q) -> Result<IndexManager<Q>, IndexOrganizerError> {
        if minimum_index > maximum_index {
            return Err(IndexOrganizerError::IndexManagerError);
        }

        if maximum_index - minimum_index < initial_number_indexes {
            return Err(IndexOrganizerError::IndexManagerError);
        }

        Ok(Self {
            minimum_index,
            maximum_index,
            next_index: minimum_index + initial_number_indexes,
            skipped_indexes: vec![],
        })
    }

    pub fn get_next_index(&mut self) -> Result<Q, IndexOrganizerError> {
        if let Some(i) = self.skipped_indexes.pop() {
            return Ok(i);
        }

        if self.next_index == self.maximum_index {
            return Err(IndexOrganizerError::IndexManagerLimit);
        }

        let i: Q = self.next_index;
        self.next_index += Q::QUANT_ONE;
        Ok(i)
    }

    pub fn return_index(&mut self, index: Q) -> Result<(), IndexOrganizerError> {
        if index == self.minimum_index - Q::QUANT_ONE {
            self.minimum_index -= Q::QUANT_ONE;
            return Ok(());
        }

        if let Some(index) = self.skipped_indexes.iter().rposition(|&item| item == index) {
            self.skipped_indexes.swap_remove(index);
        }

        Err(IndexOrganizerError::IndexManagerIndex { index: index.quant_to_usize() })
    }
}
