use crate::feagi_message::feagi_message_path::{FeagiRequestMessagePath};
use message_components::{MessageQueryables, MessageRequest, MessageParameters};

/// The request message to send to make a request. Contains internally typing context to aid in
/// routing and processing
pub trait FeagiRequestMessage {
    /// The descriptor for this Feagi Request Message
    type Descriptor: FeagiRequestMessageDescriptor<Message=Self>;

    /// Assemble the message from components
    fn compose(parameters: Self::Descriptor::Parameters, queryables: Self::Descriptor::Queryables, payload: Self::Descriptor::Payload) -> Self;

    /// Disassemble the message
    fn decompose(self) -> (Self::Descriptor::Parameters, Self::Descriptor::Queryables, Self::Descriptor::Payload);

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
    type Payload: MessageRequest; // Cannot be anything on get style requests

    /// The payload that is sent back as a response
    type Response: FeagiResponseData;

    /// The custom error type that can be sent back if something went wrong with the response
    type ResponseError: core::error::Error;


    // The generated implementation will have these functions generated, and the struct itself
    // will have each field as a pub member as well
}


pub mod message_components {

    /// All parameters for
    pub trait MessageParameters: serde::Serialize + serde::de::DeserializeOwned {
        // All fields must implement `FeagiMessageParameterFieldSerializable`
    }

    /// To be used on the fields of Parameters for Feagi Message Parameters, to ensure they can be decoded by REST calls
    pub trait FeagiMessageParameterFieldSerializable:
    core::fmt::Debug
    + core::str::FromStr
    + serde::Serialize
    + serde::de::DeserializeOwned
    {
        // TODO does zenoh need anything? probably the const to return the "any" str representation for server side
    }


    pub trait MessageQueryables: serde::Serialize + serde::de::DeserializeOwned {
        // Mainly only is used directly in rest a URL queryables
    }


    pub trait MessageRequest: serde::Serialize + serde::de::DeserializeOwned {

    }

    #[doc(hidden)]
    /// In zenoh, we package these together
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct MessageQueryablesAndRequest<Queryables: MessageQueryables, Request: MessageRequest> {
        pub queryables: Queryables,
        pub request: Request,
    }

    // /// Field name -> Doc String. Used mainly by OpenAPI generator
    //pub type StaticFieldDocumentation = (&'static str, &'static str);
}



/// The struct that gets returned as a response (assuming no error, which is handled seperately)
pub trait FeagiResponseData : serde::Serialize + serde::de::DeserializeOwned {

}

#[derive(thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum FeagiResponseError<Processing: core::error::Error + serde::Serialize + serde::de::DeserializeOwned> { // TODO
    #[error("Failed to connect to make request")]
    NoConnection,
    #[error(transparent)]
    ProcessingError(Processing)
}











