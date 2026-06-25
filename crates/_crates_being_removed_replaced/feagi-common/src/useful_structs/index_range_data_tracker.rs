use core::ops::Range;
use feagi_common_quantizable::QuantizedIndexCountTrait;
use crate::FeagiCommonError;

// TODO swap from options to Errors!


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeInsertability{
    CanInsert,
    DestinationMustBeDefragmented,
    DestinationMustBeGrown
}

impl RangeInsertability{
    pub fn can_fit(&self) -> bool {
        match &self {
            RangeInsertability::CanInsert => {true}
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct RangeInsertionResult<IndexType: QuantizedIndexCountTrait> {
    pub range_index: IndexType,
    pub range: Range<IndexType>,
    pub shifted_downstream_indexes: bool,
}


/// Tracks indexed ranges inside a fixed-size linear address space.
///
/// The tracker only stores range metadata. Deleted ranges become `None` entries so their region
/// indexes can be reused when a later insert belongs at the same ordered position.
#[derive(Debug, Clone)]
pub struct IndexRangeDataTracker<IndexType: QuantizedIndexCountTrait> {
    total_data_length: IndexType,
    used_data_length: IndexType,
    ranges: Vec<Option<Range<IndexType>>>,
}

impl<IndexType: QuantizedIndexCountTrait> IndexRangeDataTracker<IndexType> {
    pub fn new(data_length: IndexType) -> Self {
        Self {
            total_data_length: data_length,
            used_data_length: IndexType::QUANT_ZERO,
            ranges: Vec::new(),
        }
    }

    pub fn total_data_length(&self) -> IndexType {
        self.total_data_length
    }

    pub fn used_data_length(&self) -> IndexType { self.used_data_length }

    pub fn unused_sparse_data_length(&self) -> IndexType { self.total_data_length - self.used_data_length }

    pub fn get_range(&self, range_index: IndexType) -> Option<&Range<IndexType>> {
        self.ranges.get(range_index.to_usize()).and_then(|range| range.as_ref())
    }

    pub fn get_insertability(&self, range_length: IndexType) -> RangeInsertability {

        if self.unused_sparse_data_length() < range_length {
            return RangeInsertability::DestinationMustBeGrown
        }

        if self.find_smallest_fitting_gap(range_length).is_some() {
            return RangeInsertability::CanInsert;
        }

        RangeInsertability::DestinationMustBeDefragmented
    }

    pub fn get_number_allocated_ranges(&self) -> usize {
        self.ranges.iter().filter(|range| range.is_some()).count()
    }

    pub fn get_number_reusable_range_indexes(&self) -> usize {
        self.ranges.iter().filter(|range| range.is_none()).count()
    }

    pub fn set_data_length(&mut self, data_length: IndexType) -> Result<(), FeagiCommonError>{
        // TODO Check to make sure we arent shrinking too far!
        self.total_data_length = data_length;
        Ok(())
    }

    pub fn delete_range_by_index(&mut self, range_index: IndexType) -> Result<(), FeagiCommonError> {
        let range = self.ranges.get_mut(range_index.to_usize()).ok_or(
            FeagiCommonError::InvalidValue { context: "Range index is outside the tracker!" }
        )?;

        if range.is_none() {
            return Err(FeagiCommonError::InvalidValue { context: "Range index is already empty!" });
        } else {
            // TODO shrink used_data_length
        }
        *range = None;
        Ok(())
    }

    pub fn add_range_and_get_index(&mut self, range_length: IndexType) -> Result<RangeInsertionResult<IndexType>, RangeInsertability> {
        let best_gap = match self.find_smallest_fitting_gap(range_length) {
            Some(best_gap) => best_gap,
            None => return Err(self.get_insertability(range_length)),
        };

        let range = best_gap.start..(best_gap.start + range_length);
        let (range_index, shifted_downstream_indexes) = self.insert_range_at_ordered_index(best_gap.insert_at, range.clone());

        self.used_data_length += range_length;
        Ok(RangeInsertionResult {
            range_index: IndexType::from_usize_unchecked(range_index),
            range,
            shifted_downstream_indexes,
        })
    }

    pub fn invalidate_all(&mut self) {
        self.used_data_length = IndexType::QUANT_ZERO;
        self.ranges.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Range<IndexType>> {
        self.ranges.iter().filter_map(|range| range.as_ref())
    }



    fn insert_range_at_ordered_index(&mut self, insert_at: usize, range: Range<IndexType>) -> (usize, bool) {
        let reusable_index = self.find_empty_index_for_ordered_insert(insert_at);

        match reusable_index {
            Some(index) => {
                self.ranges[index] = Some(range);
                (index, false)
            }
            None => {
                self.ranges.insert(insert_at, Some(range));
                (insert_at, true)
            }
        }
    }

    fn find_empty_index_for_ordered_insert(&self, insert_at: usize) -> Option<usize> {
        if self.ranges.get(insert_at).is_some_and(|range| range.is_none()) {
            return Some(insert_at);
        }

        let previous_full_index = self.ranges[..insert_at].iter().rposition(|range| range.is_some());
        let search_start = previous_full_index.map_or(0, |index| index + 1);

        self.ranges[search_start..insert_at]
            .iter()
            .position(|range| range.is_none())
            .map(|index| search_start + index)
    }

    fn find_smallest_fitting_gap(&self, range_length: IndexType) -> Option<FreeRangeGap<IndexType>> {
        let mut previous_range_end = IndexType::QUANT_ZERO;
        let mut best_gap: Option<FreeRangeGap<IndexType>> = None;

        for (range_index, range) in self.ranges.iter().enumerate() {
            let Some(range) = range.as_ref() else {
                continue;
            };

            Self::record_gap_if_better(
                &mut best_gap,
                previous_range_end,
                range.start,
                range_index,
                range_length,
            );
            previous_range_end = range.end;
        }

        Self::record_gap_if_better(
            &mut best_gap,
            previous_range_end,
            self.total_data_length,
            self.ranges.len(),
            range_length,
        );

        best_gap
    }

    fn record_gap_if_better(
        best_gap: &mut Option<FreeRangeGap<IndexType>>,
        start: IndexType,
        end: IndexType,
        insert_at: usize,
        range_length: IndexType,
    ) {
        let gap_length = end - start;
        if gap_length < range_length {
            return;
        }

        let should_replace = best_gap.as_ref().map_or(true, |best_gap| gap_length < best_gap.length);
        if should_replace {
            *best_gap = Some(FreeRangeGap {
                start,
                length: gap_length,
                insert_at,
            });
        }
    }



}

impl<IndexType: QuantizedIndexCountTrait> Default for IndexRangeDataTracker<IndexType> {
    fn default() -> Self {
        Self::new(IndexType::QUANT_ZERO)
    }
}




#[derive(Debug, Clone)]
struct FreeRangeGap<IndexType: QuantizedIndexCountTrait> {
    start: IndexType,
    length: IndexType,
    insert_at: usize,
}