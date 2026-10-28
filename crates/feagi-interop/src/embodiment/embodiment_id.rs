use feagi_basis::feagi_id::feagi_id;

pub const NUMBER_OF_BYTES: usize = 8;

feagi_id! {
    /// Unique session tracker of an embodiment registered to FEAGI
    pub EmbodimentID: NUMBER_OF_BYTES,
    base64 = Base64EncodedEmbodimentID,
}
