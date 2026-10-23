use feagi_basis::{impl_byte_array_id, impl_byte_array_id_random};

pub const NUMBER_OF_BYTES_NODE_ID: usize = 8;

/// Length of the padded URL-safe base64 text of [FeagiNodeID] (8 bytes -> 12 characters)
#[allow(clippy::manual_div_ceil)]
pub const NUMBER_OF_BASE_64_BYTES_NODE_ID: usize = (NUMBER_OF_BYTES_NODE_ID + 2) / 3 * 4;

impl_byte_array_id!(
    /// Unique session tracker of a node tracked by FEAGI (generally for Zenoh)
    pub FeagiNodeID,
    NUMBER_OF_BYTES_NODE_ID
);

impl_byte_array_id_random!(FeagiNodeID, NUMBER_OF_BYTES_NODE_ID);