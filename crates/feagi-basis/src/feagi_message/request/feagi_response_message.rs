/// The struct that gets returned as a response (assuming no error, which is handled separately)
pub trait FeagiResponsePayload: serde::Serialize + serde::de::DeserializeOwned {

}

// Messages without any response fields use the unit type
impl FeagiResponsePayload for () {}