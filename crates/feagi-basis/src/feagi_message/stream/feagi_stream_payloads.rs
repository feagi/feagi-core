
/// The struct that gets sent every frame during a stream. Requires borrowed deserializing for
/// higher performance
pub trait FeagiStreamPayload<'de>: serde::Serialize + serde::Deserialize<'de> {}

/// For stream payloads that once declared, do not change size (if size must change, the stream must
/// be restarted)
pub trait FeagiStreamPayloadConstantSized<'de>: FeagiStreamPayload<'de> {
    // NOTE: No struct to indicate size as that depends per struct
}

/// The struct that may optionally be sent per frame
pub trait FeagiStreamOptionalPayload<'de>: serde::Serialize + serde::Deserialize<'de> {}

/// Default unit impl to make it easy to specify no optional payload exists
impl<'de> FeagiStreamOptionalPayload<'de> for () {}
