use feagi_basis::feagi_message::stream::feagi_stream_payloads::FeagiStreamPayload;
use crate::feagi_message::stream::feagi_stream_message::FeagiStreamMessage;
use crate::feagi_message::stream::feagi_stream_payloads::FeagiStreamOptionalPayload;

/// A zero sized struct that contains static information about a specific `FeagiRequestMessage`
pub trait FeagiStreamMessageDescriptor {

    /// The message associated with this descriptor
    type Message: FeagiStreamMessage<Descriptor = Self>;

    // TODO for now use a string path, only zenoh is using this anyways
    /*
    /// Defines the path that this Feagi Message Takes. Its parameters are this message's, and
    /// its configurables are those of the message's category
    type MessagePath: FeagiMessagePath<Parameters = Self::Parameters, Configurables = <Self::Category as FeagiRequestCategory>::Configurables>;

     */

    /// The payload sent every frame
    type StreamPayload<'de>: FeagiStreamPayload<'de>;

    /// Payload that could be sent additionally with a frame, normally for additional context.
    /// can be set as () to effectively disable
    type OptionalStreamPayload<'de>: FeagiStreamOptionalPayload<'de>;

}

// TODO parameters for things like blocking stragedy? Allow config vs fixed?