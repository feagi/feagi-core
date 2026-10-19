

pub mod path_builder {
    use feagi_basis::feagi_message::feagi_message_path::FeagiMessagePath;

    // TODO maybe this this should be the backend for zenoh create qeuryable / publish / etc?
    /// Creates a key expression (path) for zenoh
    pub fn create_key_expr_for_zenoh(message_path: &impl FeagiMessagePath) -> () {

    }

    // TODO add data paths for zenoh router (FeagiPublishingMessagePath?)
}