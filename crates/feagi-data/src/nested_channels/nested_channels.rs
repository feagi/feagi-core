use core::time::Duration;
use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};


/// Represents a Sender and Receiver, in this case broadly oriented for handling messages into
/// and out of a thread
pub trait NestedChannelPair<ToIn: Send, ToOut: Send> {
    type InnerChannelPair; //<ToOut, ToIn>;
    type OuterChannelPair; //<ToIn, ToOut>;

    /// Create 2 new pairs of senders / receivers
    fn new_pairs(going_in_length: usize, going_out_length: usize) -> (Self::OuterChannelPair, Self::InnerChannelPair);

    /// Send data over the channel, blocking the thread until it does (or erroring)
    fn block_send(&mut self, to_in: ToIn) -> Result<(), NestedChannelError>;

    /// Tries to send data over the channel, erroring immediately if it cannot
    fn try_send(&mut self, to_in: ToIn) -> Result<(), NestedChannelError>;

    /// Send data over the channel, blocking the thread until it does until a timeout (or erroring)
    fn send_timeout(&mut self, to_in: ToIn, timeout: Duration) -> Result<(), NestedChannelError>;

    /// Waits to receive data over the channel, blocking the thread until it does (or erroring)
    fn block_receive(&mut self) -> Result<ToOut, NestedChannelError>;

    /// Tries to receive data from the channel, erroring immediately if there is nothing available
    fn try_receive(&mut self) -> Result<ToOut, NestedChannelError>;

    /// Waits to receive data over the channel, blocking the thread until a timeout (or erroring)
    fn receive_timeout(&mut self, timeout: Duration) -> Result<ToOut, NestedChannelError>;
}


//region NestedChannelError

generate_feagi_error! {
    NestedChannelError,
    keys: {
        SendFailed: FeagiFailChannelSendFailed,
        SendChannelFull: FeagiFailChannelSendFull,
        ReceiveFailed: FeagiFailChannelReceiveFailed,
        ReceiveChannelEmpty: FeagiFailChannelReceiveEmpty,
        SendTimeout: FeagiFailChannelSendTimeout,
        ReceiveTimeout: FeagiFailChannelReceiveTimeout,
    },
    sub_errors: {

    },
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendFailed {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendFull {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveFailed {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveEmpty {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendTimeout {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveTimeout {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailThreadChannelTimeout {
    context: &'static str,
    // TODO duration?
}

//endregion