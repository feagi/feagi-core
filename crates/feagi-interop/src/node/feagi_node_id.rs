use feagi_basis::impl_byte_array_id;

pub const NUMBER_OF_BYTES_NODE_ID: usize = 8;

impl_byte_array_id!(
    /// Unique session tracker of a node tracked by FEAGI (generally for Zenoh)
    pub FeagiNodeID,
    NUMBER_OF_BYTES_NODE_ID
);
