use core::ops::Range;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;

#[derive(Debug, Clone)]
pub struct MultiBitPackedVector<Q: QuantizedIndexCountTrait>(Vec<u8>);

impl<Q: QuantizedIndexCountTrait> MultiBitPackedVector<Q>
{
    pub fn new() -> Self {
        MultiBitPackedVector(Vec::new())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub struct MultiBitPackedSlice<'a, Q: QuantizedIndexCountTrait>(&'a [u8]);

impl<'a, Q: QuantizedIndexCountTrait> MultiBitPackedSlice<'a, Q> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    pub fn number_bytes(&self) -> Q {
        Q::quant_from_usize(self.0.len())
    }

    /// Raw pointer to the first byte.
    ///
    /// # Safety
    /// The returned pointer is valid for reads for as long as this slice is
    /// alive.
    pub unsafe fn as_byte_ptr(&self) -> *const u8 {
        self.0.as_ptr()
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
        self.0.as_ptr() as *mut u8
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


pub struct MultiBitPackedVectorManager<QI: QuantizedIndexCountTrait, QV: QuantizedIndexCountTrait> {
    /// Global bit packed vector
    vector: MultiBitPackedVector<QV>,
    /// Indexed used ranges of slices with number dangling bits per slice (each slice starts on a byte)
    used_ranges: Vec<Option<(Range<QV>, u8)>>,
    /// Ranges of bytes that are free
    free_ranges: Vec<Range<QV>>,
    /// Quick lookup of skipped indexes
    skipped_indexes: Vec<QI>,
}

impl<QI: QuantizedIndexCountTrait, QV: QuantizedIndexCountTrait> MultiBitPackedVectorManager<QI, QV>
{
    pub fn new() -> Self {
        Self {
            vector: MultiBitPackedVector::new(),
            used_ranges: Vec::new(),
            free_ranges: Vec::new(),
            skipped_indexes: Vec::new(),
        }
    }

    pub fn get_new_range(&mut self, number_bits: QV) -> QI
    {
        let (number_bytes, dangling_bits) = minimum_number_bytes(number_bits);

        // TODO this is very simplistic (and doesnt account for removals) for now!

        let id_index = QI::quant_from_usize(self.used_ranges.len());

        let first = self.used_ranges.first();
        let first_byte = if first.is_none() {
            QV::QUANT_ZERO
        } else {
            first.unwrap().unwrap().0.end
        };

        let end_byte = first_byte + number_bytes;

        self.used_ranges.push(Some((first_byte..end_byte, dangling_bits)));

        id_index
    }

    /// From an id index, returns the assigned byte slice and number of dangling
    /// bits for that slice. Returns `None` if the id is missing or currently
    /// unassigned.
    pub fn get_slice_by_index(&self, index: QI) -> Option<(MultiBitPackedSlice<'_, QV>, u8)> {
        let (range, dangling_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        let data = self.vector.as_bytes().get(start..end)?;
        Some((MultiBitPackedSlice::new(data), *dangling_bits))
    }

    /// Mutable counterpart of [`Self::get_slice_by_index`].
    pub fn get_slice_by_index_mut(&mut self, index: QI) -> Option<(MultiBitPackedSlice<'_, QV>, u8)> {
        let (range, dangling_bits) = self.used_ranges.get(index.quant_to_usize())?.as_ref()?;
        let start = range.start.quant_to_usize();
        let end = range.end.quant_to_usize();
        let data = self.vector.as_mut_bytes().get_mut(start..end)?;
        Some((MultiBitPackedSlice::new(data), *dangling_bits))
    }
}






/// given a global bit index, converts it to a byte index and the bit local index
fn bit_index_to_dual_index<Q: QuantizedIndexCountTrait>(bit_index: Q) -> (Q, u8)
{
    let byte_bits = bit_index & Q::QUANT_BYTE_BIT_MASK;
    (bit_index >> 3, byte_bits.quant_to_u8())
}

/// Return the smallest whole number of bytes needed to hold a number of bits and the number of dangling bits
/// (divide by 8 rounding up)
fn minimum_number_bytes<Q: QuantizedIndexCountTrait>(bit_count: Q) -> (Q, u8) {
    let and = bit_count & Q::QUANT_BYTE_BIT_MASK;
    if and == Q::QUANT_ZERO
    {
        return (bit_count >> 3, and.quant_to_u8())
    }
    (Q::QUANT_ONE + (bit_count >> 3), and.quant_to_u8())
}
