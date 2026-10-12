use crate::feagi_message::feagi_message_path::FeagiMessagePath;
use crate::feagi_message::request_response::feagi_request_message::FeagiMessageRequestMethod;

/// In the case of a Request Message, we have a request method and expect some form of response
pub trait FeagiRequestMessagePath: FeagiMessagePath {
    /// What type of Request method is being used
    const METHOD: FeagiMessageRequestMethod;

    /// Returns true if the given request type allows payloads (AKA is not Read/GET)
    const IS_PAYLOAD_ALLOWED: bool = Self::METHOD.is_payload_allowed();
}