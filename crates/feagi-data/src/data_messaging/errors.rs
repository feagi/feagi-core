use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

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