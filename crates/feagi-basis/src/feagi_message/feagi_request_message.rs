use crate::feagi_message::feagi_message_components::{MessageParameters, MessagePayload, MessageQueryables};
use crate::feagi_message::feagi_message_path::FeagiRequestMessagePath;
use crate::feagi_message::feagi_response_message::FeagiResponsePayload;

/// The request message to send to make a request. Contains internally typing context to aid in
/// routing and processing
pub trait FeagiRequestMessage {
    /// The descriptor for this Feagi Request Message
    type Descriptor: FeagiRequestMessageDescriptor<
        Message=Self,
        Parameters=Self::DescriptorParameters,
        Queryables=Self::DescriptorQueryables,
        RequestPayload=Self::DescriptorRequestPayload
    >;
    
    // NOTE: These 3 get set directly from the Descriptor by the generator
    type DescriptorParameters: MessageParameters;
    type DescriptorQueryables: MessageQueryables;
    type DescriptorRequestPayload: MessagePayload;

    /// Assemble the message from components
    fn compose(parameters: Self::DescriptorParameters, queryables: Self::DescriptorQueryables, request_payload: Self::DescriptorRequestPayload) -> Self;

    /// Disassemble the message
    fn decompose(self) -> (Self::DescriptorParameters, Self::DescriptorQueryables, Self::DescriptorRequestPayload);

    // The struct should have the members of all the message data structs as pub members here
}


/// A zero sized struct that contains static information about a specific `FeagiRequestMessage`
pub trait FeagiRequestMessageDescriptor {
    /// Details about this message / endpoint. Used in doc generation
    const MESSAGE_DESCRIPTION: &'static str;

    /// The message associated with this descriptor
    type Message: FeagiRequestMessage<Descriptor=Self>;

    /// Defines the path that this Feagi Message Takes
    type MessagePath: FeagiRequestMessagePath;

    /// Intra path runtime configurables
    type Parameters: MessageParameters;
    /// In REST, this becomes part of the path. In Zenoh, this is coupled with the payload
    type Queryables: MessageQueryables;
    /// The payload sent with the request message. In Rest this isnt allowed for GET requests,
    /// so only queryables is allowed there
    type RequestPayload: MessagePayload; // Cannot be anything on get style requests

    /// The payload that is sent back as a response
    type ResponsePayload: FeagiResponsePayload;

    /// The custom error type that can be sent back if something went wrong with the response
    type ResponseError: core::error::Error;
    
    // This struct should be zero sized, no fields!
}











