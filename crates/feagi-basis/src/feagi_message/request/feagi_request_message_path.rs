use crate::feagi_message::feagi_message_path::{FeagiMessagePath};

/// Request Response paths
pub trait FeagiRequestMessagePath: FeagiMessagePath {
    type Method: RequestMethodType;
    const METHOD: FeagiMessageEnum = Self::Method::METHOD;
}


//region Request Method

/// An enum for defining what type of request method is being used.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FeagiMessageEnum {
    Read = 0, // get
    Create = 1, // post
    Edit = 2, // put
    Delete = 3, // delete
    Patch = 4, // patch
}

impl FeagiMessageEnum {
    /// Returns true if a request payload is allowed (Mainly relevant for REST)
    pub const fn is_request_payload_allowed(&self) -> bool {
        !matches!(self, Self::Read)
    }
}

/// A Marker trait for defining what type of request method is being used. Matched to how rest defines it
pub trait RequestMethodType {
    const METHOD: FeagiMessageEnum;
}

/// Requesting getting something. Equal to Rest GET.
/// Note that no payload is allowed in GET rest calls!
pub struct RequestMethodRead;
impl RequestMethodType for RequestMethodRead {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Read;
}

/// Requesting creating something. Equal to Rest POST.
pub struct RequestMethodCreate;
impl RequestMethodType for RequestMethodCreate {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Create;
}

/// Requesting updating something. Equal to Rest PUT.
pub struct RequestMethodEdit;
impl RequestMethodType for RequestMethodEdit {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Edit;
}

/// Requesting getting something. Equal to Rest DELETE.
pub struct RequestMethodDelete;
impl RequestMethodType for RequestMethodDelete {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Delete;
}

/// Requesting getting something. Equal to Rest PATCH.
pub struct RequestMethodPatch;
impl RequestMethodType for RequestMethodPatch {
    const METHOD: FeagiMessageEnum = FeagiMessageEnum::Patch;
}

//endregion