use crate::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameters;
use crate::feagi_message::request::feagi_request_descriptor::FeagiRequestMessageDescriptor;
use crate::feagi_message::request::feagi_request_response_payloads::FeagiRequestPayload;

/// The request message to send to make a request. Contains internally typing context to aid in
/// routing and processing
pub trait FeagiRequestMessage {
    /// The descriptor for this Feagi Request Message
    type Descriptor: FeagiRequestMessageDescriptor<
        Message=Self,
        Parameters=Self::DescriptorParameters,
        RequestPayload=Self::DescriptorRequestPayload
    >;

    // NOTE: These 2 get set directly from the Descriptor by the generator
    type DescriptorParameters: FeagiMessagePathParameters;
    type DescriptorRequestPayload: FeagiRequestPayload;

    /// Assemble the message from components
    fn compose(parameters: Self::DescriptorParameters, request_payload: Self::DescriptorRequestPayload) -> Self;

    /// Disassemble the message
    fn decompose(self) -> (Self::DescriptorParameters, Self::DescriptorRequestPayload);

    // The struct should have the members of all the message data structs as pub members here
}





