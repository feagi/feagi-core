use crate::values::quantizable::{QuantizedIndexCountTrait, WrappedQuantizedIndexCount};
use core::ops::Range;

// TODO rn very basic implementation just for adding stuff

pub struct IndexRangeMappingManager<HeaderIndex: WrappedQuantizedIndexCount, RangeIndex: WrappedQuantizedIndexCount> {
    header_indexed_used_ranges: Vec<Option<Range<RangeIndex>>>,
    empty_ranges_decreasing_order: Vec<(HeaderIndex, RangeIndex)>, // header to the current empty range, and the length of it
    skipped_headers: Vec<HeaderIndex>,
    /// The current max index referenced by a range (there may be some space after that is allocated but unused)
    current_max_range_index: RangeIndex,
    /// The amount allocated in the data vectors
    current_range_allocated: RangeIndex,
    /// The maximum allowed index
    max_allowed_range_index: RangeIndex,
    /// amount allocated range data that is not actually being used
    amount_unused_capacity: RangeIndex,
}

impl<HeaderIndex: WrappedQuantizedIndexCount, RangeIndex: WrappedQuantizedIndexCount> IndexRangeMappingManager<HeaderIndex, RangeIndex> {
    pub fn new_empty(max_range_allowed: RangeIndex) -> Self {
        Self {
            header_indexed_used_ranges: vec![],
            empty_ranges_decreasing_order: vec![],
            skipped_headers: vec![],
            current_max_range_index: RangeIndex::QUANT_ZERO,
            current_range_allocated: RangeIndex::QUANT_ZERO,
            max_allowed_range_index: max_range_allowed,
            amount_unused_capacity: RangeIndex::QUANT_ZERO,
        }
    }

    pub fn allocate_for_length(&mut self, needed_length: RangeIndex) -> Result<NewHeaderRangeStruct<HeaderIndex, RangeIndex>, ()> {
        // TODO check for space in fragmented space

        // Not enough contiguous space in fragmented free areas, append to the end
        if needed_length + self.current_max_range_index > self.max_allowed_range_index {
            return Err(()); // would overshoot allowed memory
        }

        let amount_to_allocate = needed_length - (self.current_range_allocated - self.current_max_range_index); // Make use of any free allocated space at the end
        let allocation_needed = if amount_to_allocate != RangeIndex::QUANT_ZERO {
            Some(amount_to_allocate)
        } else {
            None
        };

        let new_range = self.current_max_range_index..(self.current_max_range_index + needed_length);
        self.current_max_range_index = needed_length;
        self.current_range_allocated = new_range.end;
        let (header, range) = self.get_header_index_and_option_range();
        *range = Some(new_range.clone());
        Ok(NewHeaderRangeStruct {
            new_header_index: header,
            additional_allocation_needed: allocation_needed,
            range: new_range,
        })
    }

    /// Gets an unused header index and the mutable reference to the range it relates to (which will be a None)
    fn get_header_index_and_option_range(&mut self) -> (HeaderIndex, &mut Option<Range<RangeIndex>>) {
        if let Some(header_index) = self.skipped_headers.pop() {
            assert_eq!(
                self.header_indexed_used_ranges[header_index.quant_to_usize()],
                None,
                "Range was returned without clearing it!"
            );
            return (header_index, &mut self.header_indexed_used_ranges[header_index.quant_to_usize()]);
        }
        let header: HeaderIndex = HeaderIndex::quant_from_usize(self.header_indexed_used_ranges.len());
        self.header_indexed_used_ranges.push(None);
        (header, &mut self.header_indexed_used_ranges[header.quant_to_usize()])
    }
}

pub struct NewHeaderRangeStruct<HeaderIndex: WrappedQuantizedIndexCount, RangeIndex: WrappedQuantizedIndexCount> {
    pub new_header_index: HeaderIndex,
    pub additional_allocation_needed: Option<RangeIndex>,
    pub range: Range<RangeIndex>,
}
