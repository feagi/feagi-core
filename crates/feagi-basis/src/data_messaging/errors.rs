use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiFail};

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




#[derive(FeagiFail)]
pub struct FeagiFailChannelSendFull {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailChannelSendTimeout {
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
pub struct FeagiFailChannelEtc {
    context: &'static str,
    // TODO duration?
}

//endregion