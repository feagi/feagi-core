use feagi_data::create_quantized_index_count_wrapper;


/// Index for grouping cortical units of the same type within a genome.
///
/// This index distinguishes between multiple instances of the same cortical type.
/// For example, multiple vision sensors would have different CorticalUnitIndex
/// values (0, 1, 2, etc.) while sharing the same base cortical type.
///
/// # Range
/// Values are limited to 0-255 (u8) and are encoded in hexadecimal within cortical IDs.
/// This provides support for up to 256 instances of each cortical unit type.
///
/// # Usage in Cortical IDs
/// The index appears as the last two characters of a cortical ID:
/// - \"ivis00\" = Vision sensor, grouping index 0
/// - \"ivis01\" = Vision sensor, grouping index 1
/// - \"omot0A\" = Motor output, grouping index 10 (hexadecimal A)
create_quantized_index_count_wrapper!(CorticalUnitIndex, u8);


/// Index for cortical areas within a cortical unit. This allows easy identification of various
/// cortical areas (which can be called CorticalSubUnits in this case) within a cortical unit
create_quantized_index_count_wrapper!(CorticalSubUnitIndex, u8);