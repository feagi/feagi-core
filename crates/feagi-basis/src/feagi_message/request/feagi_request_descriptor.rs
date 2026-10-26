use crate::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameters;
use crate::feagi_message::feagi_message_path::FeagiMessagePath;
use crate::feagi_message::request::feagi_request_message::FeagiRequestMessage;
use crate::feagi_message::request::feagi_request_message_category::FeagiRequestCategory;
use crate::feagi_message::request::feagi_request_response_payloads::{FeagiMessageError, FeagiRequestPayload, FeagiResponsePayload};

/// A zero sized struct that contains static information about a specific `FeagiRequestMessage`
pub trait FeagiRequestMessageDescriptor {
    /// Details about this message / endpoint. Used in doc generation
    const MESSAGE_DESCRIPTION: &'static str;

    /// The request method as an enum
    const METHOD: FeagiMessageEnum = Self::Method::METHOD;

    /// The message associated with this descriptor
    type Message: FeagiRequestMessage<Descriptor = Self, DescriptorParameters = Self::Parameters>;

    /// Defines the path that this Feagi Message Takes. Its parameters are this message's, and
    /// its configurables are those of the message's category
    type MessagePath: FeagiMessagePath<Parameters = Self::Parameters, Configurables = <Self::Category as FeagiRequestCategory>::Configurables>;

    /// The request method as a type
    type Method: RequestMethodType;

    /// Intra path runtime settable paths
    type Parameters: FeagiMessagePathParameters;

    /// The payload sent with the request message
    type RequestPayload: FeagiRequestPayload; // Note unique handling for rest / get style requests

    /// The payload that is sent back as a response
    type ResponsePayload: FeagiResponsePayload;

    /// The custom error type that can be sent back if something went wrong with the response
    type ResponseError: FeagiMessageError;

    /// The fields of `RequestPayload`, for documentation (such as open-api)
    const REQUEST_PAYLOAD_FIELDS: &'static [crate::feagi_message::feagi_message_description::FeagiFieldDescription];

    /// The fields of `ResponsePayload`, for documentation (such as open-api)
    const RESPONSE_PAYLOAD_FIELDS: &'static [crate::feagi_message::feagi_message_description::FeagiFieldDescription];

    /// The category this message belongs to, whose channel it travels over
    type Category: FeagiRequestCategory;

    /// Wraps the message into its variant of the category's request enum
    fn into_category_request(message: Self::Message) -> <Self::Category as FeagiRequestCategory>::RequestEnum;

    /// Wraps a response to this message into its variant of the category's response enum
    fn into_category_response(response: Result<Self::ResponsePayload, Self::ResponseError>)
        -> <Self::Category as FeagiRequestCategory>::ResponseEnum;

    /// Unwraps a response to this message from the category's response enum. Gives the enum
    /// back if it holds a response to some other message
    fn try_from_category_response(
        response: <Self::Category as FeagiRequestCategory>::ResponseEnum,
    ) -> Result<Result<Self::ResponsePayload, Self::ResponseError>, <Self::Category as FeagiRequestCategory>::ResponseEnum>;

    // This struct should be zero sized, no fields!
}

//region Request Method

/// An enum for defining what type of request method is being used.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeagiMessageEnum {
    Read = 0,   // get
    Create = 1, // post
    Edit = 2,   // put
    Delete = 3, // delete
    Patch = 4,  // patch
}

impl FeagiMessageEnum {
    /// Returns true if a request payload is in the URL (only for rest GET requests)
    pub const fn is_payload_in_url(&self) -> bool {
        matches!(self, Self::Read)
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
