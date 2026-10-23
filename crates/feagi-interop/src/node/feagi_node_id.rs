use feagi_basis::{impl_byte_array_id, impl_byte_array_id_random};

pub const NUMBER_OF_BYTES_NODE_ID: usize = 8;

impl_byte_array_id!(
    /// Unique session tracker of a node tracked by FEAGI (generally for Zenoh)
    pub FeagiNodeID,
    NUMBER_OF_BYTES_NODE_ID
);

impl_byte_array_id_random!(FeagiNodeID, NUMBER_OF_BYTES_NODE_ID);