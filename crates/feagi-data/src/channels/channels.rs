use core::time::Duration;
use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};


/// Represents a Sender and Receiver, in this case broadly oriented for handling messages into
/// and out of a thread
pub trait ChannelPair<ToA: Send, ToB: Send> {
    type AChannelPair; //<ToB, ToA>;
    type BChannelPair; //<ToA, ToB>;

    /// Create 2 new pairs of senders / receivers
    fn new_pairs(going_to_b_buffer_length: usize, going_to_a_buffer_length: usize) -> (Self::BChannelPair, Self::AChannelPair);

    /// Send data over the channel, blocking the thread until it does (or erroring)
    fn block_send(&mut self, to_a: ToA) -> Result<(), ChannelSendingError>;

    /// Tries to send data over the channel, erroring immediately if it cannot
    fn try_send(&mut self, to_a: ToA) -> Result<(), ChannelSendingError>;

    /// Send data over the channel, blocking the thread until it does until a timeout (or erroring)
    fn send_timeout(&mut self, to_a: ToA, timeout: Duration) -> Result<(), ChannelSendingError>;

    /// Waits to receive data over the channel, blocking the thread until it does (or erroring)
    fn block_receive(&mut self) -> Result<ToB, ChannelReceivingError>;

    /// Tries to receive data from the channel, erroring immediately if there is nothing available
    fn try_receive(&mut self) -> Result<Option<ToB>, ChannelReceivingError>;

    /// Waits to receive data over the channel, blocking the thread until a timeout (or erroring)
    fn receive_timeout(&mut self, timeout: Duration) -> Result<ToB, ChannelReceivingError>;
}


//region ChannelError

generate_feagi_error! {
    ChannelSendingError,
    keys: {
        SendFailed: FeagiFailChannelSendEtc,
        SendChannelFull: FeagiFailChannelSendFull,
        SendTimeout: FeagiFailChannelSendTimeout,
    },
    sub_errors: {

    },
}

generate_feagi_error! {
    ChannelReceivingError,
    keys: {
        ReceiveFailed: FeagiFailChannelReceiveEtc,
        ReceiveTimeout: FeagiFailChannelReceiveTimeout,
    },
    sub_errors: {

    },
}

generate_feagi_error! {
    ChannelError,
    keys: {
        Etc: FeagiFailChannelEtc
    },
    sub_errors: {
        SendingError: ChannelSendingError,
        ReceivingError: ChannelReceivingError,
    },
}




#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendFull {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendTimeout {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelSendEtc {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveTimeout {
    context: &'static str,
}

#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelReceiveEtc {
    context: &'static str,
}



#[derive(FeagiErrorKey)]
pub struct FeagiFailChannelEtc {
    context: &'static str,
    // TODO duration?
}

//endregion