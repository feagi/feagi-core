//! Many independent bit-packed runs sharing one contiguous byte buffer.
//!
//! The engine needs one bit per neuron per cortical area (currently: "did this neuron fire in
//! this burst"). Giving each area its own allocation would scatter those bits across the heap and
//! cost a pointer chase per area, so instead every area gets a byte-aligned slice of a single
//! buffer and is addressed by the id handed out at allocation time.
//!
//! Byte alignment is the load-bearing property, not an implementation detail. Because no two runs
//! ever share a byte, and because a byte holds exactly eight neurons, a parallel kernel can give
//! each worker one whole byte and let it write without atomics or locks. Bit-level packing across
//! run boundaries would make every write a read-modify-write race.

use core::marker::PhantomData;
use core::ops::Range;
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;

/// The shared byte buffer that every run is carved out of.
#[derive(Debug, Clone)]
pub struct MultiBitPackedVector<Q: QuantizedUnsignedIntegerTrait> {
    data: Vec<u8>,
    phantom_data: PhantomData<Q>,
}

impl<Q: QuantizedUnsignedIntegerTrait> MultiBitPackedVector<Q> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn number_bytes(&self) -> Q {
        Q::quant_from_usize_unchecked(self.data.len())
    }

    /// Grows the buffer by `additional_bytes` zeroed bytes and returns the byte index the new
    /// space starts at.
    ///
    /// This can reallocate, so any slice previously borrowed from this buffer is invalidated. That
    /// is enforced by the borrow checker: growth needs `&mut self` and slices are handed out from
    /// `&self`.
    fn append_zeroed_bytes(&mut self, additional_bytes: usize) -> usize {
        let first_new_byte = self.data.len();
        self.data.resize(first_new_byte + additional_bytes, 0);
        first_new_byte
    }
}

impl<Q: QuantizedUnsignedIntegerTrait> Default for MultiBitPackedVector<Q> {
    fn default() -> Self {
        Self::new()
    }
}

/// A borrowed view of one run's bytes.
pub struct MultiBitPackedSlice<'a, Q: QuantizedUnsignedIntegerTrait> {
    data: &'a [u8],
    phantom_data: PhantomData<Q>,
}

impl<'a, Q: QuantizedUnsignedIntegerTrait> MultiBitPackedSlice<'a, Q> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data
    }

    pub fn number_bytes(&self) -> Q {
        Q::quant_from_usize_unchecked(self.data.len())
    }

    /// Reads one bit, or `None` if the run does not extend that far.
    pub fn get_bit(&self, bit_index: Q) -> Option<bool> {
        let bit = bit_index.quant_to_usize();
        let byte = self.data.get(bit >> 3)?;
        Some((byte >> (bit & 0b0000_0111)) & 1 == 1)
    }

    /// Number of set bits across the whole run.
    ///
    /// Bits past the run's logical length are held at zero by the manager, so this counts only
    /// real members without needing the length passed in.
    pub fn count_set_bits(&self) -> usize {
        self.data.iter().map(|byte| byte.count_ones() as usize).sum()
    }

    /// Calls `visit` with the index of every set bit, in ascending order.
    ///
    /// Whole zero bytes are skipped, so the cost tracks the number of set bits rather than the
    /// size of the run whenever activity is sparse.
    pub fn for_each_set_bit<F: FnMut(Q)>(&self, mut visit: F) {
        for (byte_index, byte) in self.data.iter().enumerate() {
            let mut remaining = *byte;
            while remaining != 0 {
                let bit = remaining.trailing_zeros() as usize;
                visit(Q::quant_from_usize_unchecked((byte_index << 3) | bit));
                remaining &= remaining - 1;
            }
        }
    }

    /// Raw pointer to the first byte.
    ///
    /// # Safety
    /// The returned pointer is valid for reads for as long as this slice is
    /// alive.
    pub unsafe fn as_byte_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Returns a shared reference to a byte without bounds checks.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.number_bytes()`).
    pub unsafe fn get_byte_par(&self, index: Q) -> &u8 {
        &*self.as_byte_ptr().add(index.quant_to_usize())
    }

    /// Raw mutable pointer to the first byte, derived from shared `&self`.
    ///
    /// # Safety
    /// The caller must uphold exclusivity: writes through this pointer must not
    /// alias with any other references to the same bytes.
    pub unsafe fn as_mut_byte_ptr_par(&self) -> *mut u8 {
        self.data.as_ptr() as *mut u8
    }

    /// Returns a mutable reference to a byte through shared `&self`.
    ///
    /// # Safety
    /// - `index` must be in bounds (`index < self.number_bytes()`).
    /// - No other references (shared or mutable) to that same byte may exist
    ///   for the duration of the returned borrow.
    /// - Concurrent callers must only target disjoint byte indices.
    pub unsafe fn get_byte_mut_par(&self, index: Q) -> &mut u8 {
        &mut *self.as_mut_byte_ptr_par().add(index.quant_to_usize())
    }
}

/// Hands out byte-aligned bit runs from one shared buffer, addressed by allocation id.
///
/// `QI` indexes the runs (one per cortical area); `QV` indexes bits and bytes within a run.
pub struct MultiBitPackedVectorManager<QI: QuantizedUnsignedIntegerTrait, QV: QuantizedUnsignedIntegerTrait> {
    /// Every run's bytes, back to back.
    vector: MultiBitPackedVector<QV>,
    /// Per run: the byte range it owns and how many bits of that range are in use. `None` marks a
    /// run that has been released. Indexed by the id returned from [`Self::get_new_range`].
    used_ranges: Vec<Option<(Range<QV>, QV)>>,
    /// Byte ranges freed by released runs, available for reuse.
    ///
    /// Reserved for the area-removal path, which the engine does not implement yet, so nothing
    /// populates this and every allocation currently extends the buffer.
    #[allow(dead_code)]
    free_ranges: Vec<Range<QV>>,
    /// Ids of released runs, available for reuse. Reserved alongside `free_ranges`.
    #[allow(dead_code)]
    skipped_indexes: Vec<QI>,
}

impl<QI: QuantizedUnsignedIntegerTrait, QV: QuantizedUnsignedIntegerTrait> MultiBitPackedVectorManager<QI, QV> {
    pub fn new() -> Self {
        Self {
            vector: MultiBitPackedVector::new(),
            used_ranges: Vec::new(),
            free_ranges: Vec::new(),
            skipped_indexes: Vec::new(),
        }
    }

    /// Allocates a run of `number_bits`, all initialised to zero, and returns its id.
    ///
    /// The run is rounded up to a whole number of bytes so it cannot share a byte with its
    /// neighbours. Ids are handed out in allocation order and stay valid until the run is
    /// released.
    pub fn get_new_range(&mut self, number_bits: QV) -> QI {
        let number_bytes = number_bits_to_number_bytes(number_bits.quant_to_usize());
        let first_byte = self.vector.append_zeroed_bytes(number_bytes);

        let range = QV::quant_from_usize_unchecked(first_byte)..QV::quant_from_usize_unchecked(first_byte + number_bytes);
        let id_index = QI::quant_from_usize_unchecked(self.used_ranges.len());
        self.used_ranges.push(Some((range, number_bits)));

        id_index
    }

    /// Number of run ids handed out so far, including released ones.
    pub fn len(&self) -> QI {
        QI::quant_from_usize_unchecked(self.used_ranges.len())
    }

    pub fn is_empty(&self) -> bool {
        self.used_ranges.is_empty()
    }

    /// Number of bits the run holds, or `None` if the id is missing or released.
    pub fn number_bits(&self, index: QI) -> Option<QV> {
        let (_, number_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        Some(*number_bits)
    }

    /// From an id index, returns the assigned byte slice and number of dangling
    /// bits for that slice. Returns `None` if the id is missing or currently
    /// unassigned.
    ///
    /// Dangling bits are the bits of the final byte beyond the run's length. The manager holds
    /// them at zero so readers can treat the whole slice as members.
    pub fn get_slice_by_index(&self, index: QI) -> Option<(MultiBitPackedSlice<'_, QV>, u8)> {
        let (range, number_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        let data = self.vector.as_bytes().get(start..end)?;
        let dangling_bits = (data.len() * 8 - number_bits.quant_to_usize()) as u8;
        Some((MultiBitPackedSlice::new(data), dangling_bits))
    }

    /// Mutable counterpart of [`Self::get_slice_by_index`].
    pub fn get_slice_by_index_mut(&mut self, index: QI) -> Option<(MultiBitPackedSlice<'_, QV>, u8)> {
        let (range, number_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        let number_bits = number_bits.quant_to_usize();
        let data = self.vector.as_mut_bytes().get_mut(start..end)?;
        let dangling_bits = (data.len() * 8 - number_bits) as u8;
        Some((MultiBitPackedSlice::new(data), dangling_bits))
    }

    /// Reads one bit of one run, or `None` if the id or the bit is out of range.
    pub fn get_bit(&self, index: QI, bit_index: QV) -> Option<bool> {
        let (range, number_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        if bit_index >= *number_bits {
            return None;
        }
        let bit = bit_index.quant_to_usize();
        let byte = self.vector.as_bytes()[range.start.quant_to_usize() + (bit >> 3)];
        Some((byte >> (bit & 0b0000_0111)) & 1 == 1)
    }

    /// Writes one bit of one run, returning its previous value, or `None` if the id or the bit is
    /// out of range.
    pub fn set_bit(&mut self, index: QI, bit_index: QV, value: bool) -> Option<bool> {
        let (range, number_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        if bit_index >= *number_bits {
            return None;
        }
        let bit = bit_index.quant_to_usize();
        let byte_index = range.start.quant_to_usize() + (bit >> 3);
        let mask = 1u8 << (bit & 0b0000_0111);

        let byte = &mut self.vector.as_mut_bytes()[byte_index];
        let previous = (*byte & mask) != 0;
        if value {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
        Some(previous)
    }

    /// Zeroes every bit of one run.
    pub fn clear(&mut self, index: QI) {
        let Some(Some((range, _))) = self.used_ranges.get(index.quant_to_usize()).cloned() else {
            return;
        };
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        self.vector.as_mut_bytes()[start..end].fill(0);
    }

    /// Zeroes every bit of every run.
    pub fn clear_all(&mut self) {
        self.vector.as_mut_bytes().fill(0);
    }
}

impl<QI: QuantizedUnsignedIntegerTrait, QV: QuantizedUnsignedIntegerTrait> Default for MultiBitPackedVectorManager<QI, QV> {
    fn default() -> Self {
        Self::new()
    }
}

/// Smallest whole number of bytes that can hold `number_bits`, i.e. division by eight rounding up.
fn number_bits_to_number_bytes(number_bits: usize) -> usize {
    number_bits.div_ceil(8)
}

// Unit tests live in the module because `engines_common` is private to the crate, so an
// integration test under `tests/` cannot reach the allocator.
#[cfg(test)]
mod tests {
    use super::*;

    /// A manager shaped like the engine's: runs indexed by a small type, bits by a larger one.
    type TestManager = MultiBitPackedVectorManager<u16, u32>;

    #[test]
    fn runs_are_handed_out_in_allocation_order() {
        let mut manager = TestManager::new();

        assert_eq!(manager.get_new_range(8), 0);
        assert_eq!(manager.get_new_range(8), 1);
        assert_eq!(manager.get_new_range(8), 2);
        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn a_run_is_rounded_up_to_whole_bytes() {
        let mut manager = TestManager::new();

        let exact = manager.get_new_range(16);
        let ragged = manager.get_new_range(9);

        let (exact_bytes, exact_dangling) = manager.get_slice_by_index(exact).unwrap();
        assert_eq!(exact_bytes.number_bytes(), 2);
        assert_eq!(exact_dangling, 0);

        let (ragged_bytes, ragged_dangling) = manager.get_slice_by_index(ragged).unwrap();
        assert_eq!(ragged_bytes.number_bytes(), 2, "nine bits need two bytes");
        assert_eq!(ragged_dangling, 7, "the last byte carries seven unused bits");
    }

    #[test]
    fn an_empty_run_occupies_no_bytes() {
        let mut manager = TestManager::new();

        let empty = manager.get_new_range(0);

        let (bytes, dangling) = manager.get_slice_by_index(empty).unwrap();
        assert_eq!(bytes.number_bytes(), 0);
        assert_eq!(dangling, 0);
    }

    #[test]
    fn every_run_starts_zeroed() {
        let mut manager = TestManager::new();

        let run = manager.get_new_range(100);

        let (bytes, _) = manager.get_slice_by_index(run).unwrap();
        assert_eq!(bytes.count_set_bits(), 0);
    }

    /// The property the parallel burst kernel depends on: no two runs share a byte, so writing
    /// one run can never disturb another.
    #[test]
    fn runs_never_overlap_even_when_ragged() {
        let mut manager = TestManager::new();

        // Deliberately not multiples of eight, so a bit-packed layout would make them share bytes.
        let first = manager.get_new_range(3);
        let second = manager.get_new_range(5);
        let third = manager.get_new_range(11);

        for bit in 0..3 {
            manager.set_bit(first, bit, true);
        }
        for bit in 0..11 {
            manager.set_bit(third, bit, true);
        }

        let (second_bytes, _) = manager.get_slice_by_index(second).unwrap();
        assert_eq!(second_bytes.count_set_bits(), 0, "writing neighbouring runs must not leak into this one");

        let (first_bytes, _) = manager.get_slice_by_index(first).unwrap();
        assert_eq!(first_bytes.count_set_bits(), 3);
        let (third_bytes, _) = manager.get_slice_by_index(third).unwrap();
        assert_eq!(third_bytes.count_set_bits(), 11);
    }

    #[test]
    fn the_buffer_grows_to_fit_every_run() {
        let mut manager = TestManager::new();

        let first = manager.get_new_range(64);
        let second = manager.get_new_range(64);

        // Filling the last bit of the second run would panic or silently no-op if the backing
        // buffer had not actually grown to cover it.
        assert_eq!(manager.set_bit(second, 63, true), Some(false));
        assert_eq!(manager.get_bit(second, 63), Some(true));
        assert_eq!(manager.get_bit(first, 63), Some(false));
    }

    #[test]
    fn bits_round_trip_and_report_their_previous_value() {
        let mut manager = TestManager::new();
        let run = manager.get_new_range(20);

        assert_eq!(manager.set_bit(run, 5, true), Some(false));
        assert_eq!(manager.get_bit(run, 5), Some(true));
        assert_eq!(manager.set_bit(run, 5, false), Some(true));
        assert_eq!(manager.get_bit(run, 5), Some(false));
    }

    #[test]
    fn bits_past_the_run_length_are_rejected() {
        let mut manager = TestManager::new();
        // 12 bits occupy two bytes, so bit 12 is addressable in storage but not a member.
        let run = manager.get_new_range(12);

        assert_eq!(manager.get_bit(run, 12), None);
        assert_eq!(manager.set_bit(run, 12, true), None);
    }

    #[test]
    fn an_unknown_run_id_is_rejected_rather_than_panicking() {
        let mut manager = TestManager::new();
        manager.get_new_range(8);

        assert!(manager.get_slice_by_index(7).is_none());
        assert_eq!(manager.get_bit(7, 0), None);
        assert_eq!(manager.set_bit(7, 0, true), None);
    }

    #[test]
    fn set_bits_are_visited_in_ascending_order() {
        let mut manager = TestManager::new();
        let run = manager.get_new_range(40);

        for bit in [0u32, 7, 8, 31, 39] {
            manager.set_bit(run, bit, true);
        }

        let (bytes, _) = manager.get_slice_by_index(run).unwrap();
        let mut visited = Vec::new();
        bytes.for_each_set_bit(|bit| visited.push(bit));

        assert_eq!(visited, vec![0, 7, 8, 31, 39]);
        assert_eq!(bytes.count_set_bits(), visited.len());
    }

    #[test]
    fn clearing_one_run_leaves_the_others_alone() {
        let mut manager = TestManager::new();
        let first = manager.get_new_range(16);
        let second = manager.get_new_range(16);
        manager.set_bit(first, 1, true);
        manager.set_bit(second, 1, true);

        manager.clear(first);

        assert_eq!(manager.get_bit(first, 1), Some(false));
        assert_eq!(manager.get_bit(second, 1), Some(true));
    }

    #[test]
    fn clearing_everything_empties_every_run() {
        let mut manager = TestManager::new();
        let first = manager.get_new_range(16);
        let second = manager.get_new_range(16);
        manager.set_bit(first, 1, true);
        manager.set_bit(second, 9, true);

        manager.clear_all();

        assert_eq!(manager.get_bit(first, 1), Some(false));
        assert_eq!(manager.get_bit(second, 9), Some(false));
    }
}
