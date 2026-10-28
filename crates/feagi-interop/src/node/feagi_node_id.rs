use feagi_basis::feagi_id::feagi_id;

pub const NUMBER_OF_BYTES_NODE_ID: usize = 8;

/// Length of the padded URL-safe base64 text of [FeagiNodeID] (8 bytes -> 12 characters)
pub const NUMBER_OF_BASE_64_BYTES_NODE_ID: usize = NUMBER_OF_BYTES_NODE_ID.div_ceil(3) * 4;

feagi_id! {
    /// Unique session tracker of a node tracked by FEAGI (generally for Zenoh)
    pub FeagiNodeID: NUMBER_OF_BYTES_NODE_ID,
    random,
    base64 = Base64EncodedFeagiNodeID,
}
