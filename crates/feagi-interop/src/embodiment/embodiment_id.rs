use feagi_basis::impl_byte_array_id;

pub const NUMBER_OF_BYTES: usize = 8;

impl_byte_array_id!(
    /// Unique session tracker of an embodiment registered to FEAGI
    pub EmbodimentID,
    NUMBER_OF_BYTES
);
