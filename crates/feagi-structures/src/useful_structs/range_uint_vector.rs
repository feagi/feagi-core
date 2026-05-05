use core::ops::Range;
use core::marker::PhantomData;
use crate::quantization::QuantizableUIntType;

#[derive(Debug, Clone)]
pub struct RangeUintVector<RangeIndex, RangeCount> {
    ranges: Vec<Range<RangeIndex>>,
    _phantom: PhantomData<RangeCount>
}

impl<RangeIndex, RangeCount> RangeUintVector<RangeIndex, RangeCount>
where
    RangeIndex: QuantizableUIntType,
    RangeCount: QuantizableUIntType,
{
    pub fn new() -> Self {
        RangeUintVector { ranges: Vec::new(), _phantom: PhantomData }
    }

    /// Adds a range to the vector, merging with existing ranges if overlapping or adjacent
    pub fn add_range(&mut self, inserting_range: Range<RangeIndex>) {

        let mut insert_at: usize = 0;
        // Process existing ranges to find where to insert/merge
        for i in 0..self.ranges.len() {

            if inserting_range.start > self.ranges[i].end {
                insert_at += 1;
            }

            if self.ranges[i].end == inserting_range.start {
                // adjacent to previous range, merge
                self.ranges[i].end = inserting_range.end;

                if i == self.ranges.len() - 2 {
                    // nothing next
                    return;
                }
                if self.ranges[i].end == self.ranges[i + 1].start {
                    // Also adjacent to next range, merge that too
                    self.ranges[i].end = self.ranges[i + 1].end;
                    self.ranges.remove(i + 1);
                    return;
                }
                continue;
            }

            if inserting_range.end == self.ranges[i].start {
                // adjacent to next range
                self.ranges[i].start = inserting_range.start;
                return;
            }
        }
        self.ranges.insert(insert_at, inserting_range);
    }

    /// Finds a range big enough to contain the given length and takes from it
    /// Returns Some((start, length)) if found, None otherwise
    /// Updates internal ranges accordingly when allocation is made
    pub fn find_space(&mut self, length: RangeCount) -> Option<(Range<RangeIndex>)>
    {
        let length: RangeIndex =  RangeIndex::from_usize(length.to_usize());

        for i in 0..self.ranges.len() {
            let range_length = self.ranges[i].end - self.ranges[i].start;

            if range_length == length {
                let output = self.ranges.remove(i);
                return Some(output)
            }

            if range_length >= length {
                // Range can fit this length with spare
                let output = self.ranges[i].start..(self.ranges[i].start + length);
                self.ranges[i].start += length;
                return Some(output)
            }
        }
        None
    }

    /// Gets an iterator over all ranges
    pub fn iter(&self) -> core::slice::Iter<Range<RangeIndex>> {
        self.ranges.iter()
    }

    /// Returns the number of ranges
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Checks if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Clears all ranges
    pub fn clear(&mut self) {
        self.ranges.clear();
    }
}