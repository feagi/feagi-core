use feagi_basis_macros::prelude::*;


generate_feagi_error! {
    ChannelError,
    keys: {
        SendFull: FeagiFailChannelSendFull,
        SendTimeout: FeagiFailChannelSendTimeout,
        SendEtc: FeagiFailChannelSendEtc,
        SendClosed: FeagiFailChannelClosed,
        ReceiveTimeout: FeagiFailChannelReceiveTimeout,
        ReceiveEtc: FeagiFailChannelReceiveEtc,
        PoolFull: FeagiFailPoolFull,
        PoolEmpty: FeagiFailPoolEmpty
    },
    sub_errors: {
        
    },
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelSendFull {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelSendTimeout {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelClosed {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelSendEtc {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelReceiveTimeout {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelReceiveEtc {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailPoolFull {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailPoolEmpty {
    context: &'static str,
}

impl<T> From<thingbuf::mpsc::errors::Closed<T>> for ChannelError {
    fn from(_: thingbuf::mpsc::errors::Closed<T>) -> Self {
        ChannelError::SendClosed(FeagiFailChannelClosed::new("Thingbuf cannot send as the channel has been closed!"))
    }
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
    ThreadMessaging,
    keys: {
        Etc: FeagiFailChannelEtc
    },
    sub_errors: {
        SendingError: ChannelSendingError,
        ReceivingError: ChannelReceivingError,
    },
}








#[derive(FeagiFail)]
pub struct FeagiFailChannelEtc {
    context: &'static str,
    // TODO duration?
}

//endregion