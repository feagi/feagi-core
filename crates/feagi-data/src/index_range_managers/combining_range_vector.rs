use crate::index_range_managers::feagi_index_range_manager_error::{FeagiIndexRangeManagerError, FeagiIndexRangeVectorFailedMerge};
use crate::values::quantizable::QuantizedUnsignedIntegerTrait;
use core::ops::Range;

/// Contains a vector of incrementing (by start / end index) ranges that are not overlapping, as well as indexes to those
/// ranges in order of decrementing lengths of each range. Inserting a range will attempt to insert
/// a range maintaining order, but if the range touches any neighbors, those neighbors are merged
/// to produce a contiguous range instead
pub struct CombiningRangeVector<Q: QuantizedUnsignedIntegerTrait> {
    ranges: Vec<Range<Q>>,
    indexes_sorted_by_length: Vec<Q>,
    total_value: Q,
}

impl<Q: QuantizedUnsignedIntegerTrait> CombiningRangeVector<Q> {
    pub fn new() -> Self {
        Self {
            ranges: vec![],
            indexes_sorted_by_length: vec![],
            total_value: Q::QUANT_ZERO,
        }
    }

    /// Returns the number of contained ranges
    pub fn get_number_ranges(&self) -> Q {
        Q::quant_from_usize_unchecked(self.ranges.len())
    }

    /// Returns the total value of all lengths of all ranges summed together
    pub fn get_all_ranges_sum(&self) -> Q {
        self.total_value
    }

    /// Does a reverse search through the internal ranges to find the smallest range that can
    /// contain the given length. If one is found, a new range of length starting at the found
    /// range is returned after that given range had the part being removed cut out, with
    /// reordering of internal index sorting updated as well. If no suitable range is found,
    /// returns None.
    pub fn try_pop_range_containing_length(&mut self, length: Q) -> Option<Range<Q>> {
        // NOTE: Since we are looking through a list of ranges in decrementing order, use the
        // internal sorted length vector to quickly jump to the index of each range!
        let mut found_range_index: Option<usize> = None;
        for sorted_position in (0..self.indexes_sorted_by_length.len()) {
            let range_index = self.indexes_sorted_by_length[sorted_position].quant_to_usize();
            let range = &self.ranges[range_index];
            if range.end - range.start >= length {
                found_range_index = Some(range_index);
                break;
            }
        }

        let range_index = found_range_index?;
        let range = &mut self.ranges[range_index];
        let popped_start = range.start;
        let popped = popped_start..(popped_start + length);

        // Cut the popped chunk out of the front of the found range.
        range.start = popped_start + length;
        if range.start == range.end {
            self.ranges.remove(range_index);
        }

        self.total_value -= length;
        self.rebuild_length_ordering();
        Some(popped)
    }

    /// Undoes the effects of `try_pop_range_containing_length` by returning the Range it produced,
    /// and restoring the inner state of this struct at that point. Note that this must be called
    /// immediately after `try_pop_range_containing_length` with the range it returned, otherwise
    /// this struct will be left in an invalid state.
    pub fn undo_pop_range_containing_length(&mut self, undoing_range: Range<Q>) {
        let length = undoing_range.end - undoing_range.start;

        // The pop cut `undoing_range` off the front of a range, leaving a range that now starts at
        // `undoing_range.end`. If that range still exists, simply grow its front back. Otherwise the
        // range was fully consumed and removed, so re-insert it at the correct ordered position.
        if let Some(position) = self.ranges.iter().position(|range| range.start == undoing_range.end) {
            self.ranges[position].start = undoing_range.start;
        } else {
            let insert_at = self.ranges.partition_point(|range| range.start < undoing_range.start);
            self.ranges.insert(insert_at, undoing_range);
        }

        self.total_value += length;
        self.rebuild_length_ordering();
    }

    /// Attempts to insert a range maintaining the order to the ranges internal vector. If
    /// the range shares a side with a neighbor, they are merged. The ordered indexes will also be
    /// updated as a result.
    ///  NOTE: It is assumed that any inserting range may not overlap into existing ranges!
    pub fn insert_range_merge(&mut self, range: Range<Q>) -> Result<(), FeagiIndexRangeManagerError> {
        if range.start > range.end {
            return Err(FeagiIndexRangeVectorFailedMerge::new("Inserting range has a start greater than its end.").into());
        }

        let added_length = range.end - range.start;
        let insert_at = self.ranges.partition_point(|existing| existing.start < range.start);

        // Reject overlap with either neighbor (touching, i.e. equal bounds, is allowed and merges).
        if insert_at > 0 && self.ranges[insert_at - 1].end > range.start {
            return Err(FeagiIndexRangeVectorFailedMerge::new("Inserting range overlaps an existing range.").into());
        }
        if insert_at < self.ranges.len() && range.end > self.ranges[insert_at].start {
            return Err(FeagiIndexRangeVectorFailedMerge::new("Inserting range overlaps an existing range.").into());
        }

        let merge_previous = insert_at > 0 && self.ranges[insert_at - 1].end == range.start;
        let merge_next = insert_at < self.ranges.len() && self.ranges[insert_at].start == range.end;

        match (merge_previous, merge_next) {
            (true, true) => {
                let joined_end = self.ranges[insert_at].end;
                self.ranges[insert_at - 1].end = joined_end;
                self.ranges.remove(insert_at);
            }
            (true, false) => {
                self.ranges[insert_at - 1].end = range.end;
            }
            (false, true) => {
                self.ranges[insert_at].start = range.start;
            }
            (false, false) => {
                self.ranges.insert(insert_at, range);
            }
        }

        self.total_value += added_length;
        self.rebuild_length_ordering();
        Ok(())
    }

    /// Undoes the effects of `insert_range_merge` by removing the range generated by that function,
    /// recutting any existing ranges that may have been previously merged as well.
    /// Note this must be called immediately after `try_pop_range_containing_length` otherwise
    /// this struct may be left in an invalid state
    pub fn undo_insert_range_merge(&mut self, undoing_range: Range<Q>) {
        // Locate the (possibly merged) range that now contains the inserted range and cut it back out.
        let position = self
            .ranges
            .iter()
            .position(|range| range.start <= undoing_range.start && undoing_range.end <= range.end);
        let Some(position) = position else {
            return;
        };

        let containing_start = self.ranges[position].start;
        let containing_end = self.ranges[position].end;
        let removed_length = undoing_range.end - undoing_range.start;

        let leftover_on_left = containing_start != undoing_range.start;
        let leftover_on_right = containing_end != undoing_range.end;

        match (leftover_on_left, leftover_on_right) {
            (false, false) => {
                self.ranges.remove(position);
            }
            (false, true) => {
                self.ranges[position].start = undoing_range.end;
            }
            (true, false) => {
                self.ranges[position].end = undoing_range.start;
            }
            (true, true) => {
                self.ranges[position].end = undoing_range.start;
                self.ranges.insert(position + 1, undoing_range.end..containing_end);
            }
        }

        self.total_value -= removed_length;
        self.rebuild_length_ordering();
    }

    /// Clears all values in the internal vectors without deallocating memory
    pub fn clear(&mut self) {
        self.ranges.clear();
        self.indexes_sorted_by_length.clear();
        self.total_value = Q::QUANT_ZERO;
    }

    /// Deallocates any unused memory at the end of the vectors
    pub fn shrink_to_fit(&mut self) {
        self.ranges.shrink_to_fit();
        self.indexes_sorted_by_length.shrink_to_fit();
    }

    /// Rebuilds `indexes_sorted_by_length` so it holds every index into `ranges` ordered by
    /// decrementing range length. Called after any mutation that changes range lengths or the
    /// layout of `ranges`.
    fn rebuild_length_ordering(&mut self) {
        self.indexes_sorted_by_length.clear();
        self.indexes_sorted_by_length.reserve(self.ranges.len());
        for i in 0..self.ranges.len() {
            self.indexes_sorted_by_length.push(Q::quant_from_usize_unchecked(i));
        }

        let ranges = &self.ranges;
        self.indexes_sorted_by_length.sort_by(|a, b| {
            let length_a = {
                let range = &ranges[a.quant_to_usize()];
                range.end - range.start
            };
            let length_b = {
                let range = &ranges[b.quant_to_usize()];
                range.end - range.start
            };
            // Decrementing order: longest ranges first.
            length_b.partial_cmp(&length_a).expect("range lengths are always comparable")
        });
    }
}
