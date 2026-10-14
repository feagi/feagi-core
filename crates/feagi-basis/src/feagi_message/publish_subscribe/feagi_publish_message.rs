use crate::feagi_message::feagi_message::FeagiMessage;
use crate::feagi_message::feagi_message_element::MessageElementSerializable;

pub trait FeagiPublishMessage: FeagiMessage {
    type OptionalMetadata: MessageElementSerializable; // TODO ?
}