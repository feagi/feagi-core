use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

generate_feagi_error! {
    /// Burst engine related error
    BurstEngineError,
    keys: {
        Phase: FeagiFailPhase,
        DataCorruption: FeagiFailConnectomeCorruption,
    },
    sub_errors: {
        
    },
}

/// Burst engine experienced a failure in executing a burst phase. 
#[derive(FeagiErrorKey)]
pub struct FeagiFailPhase {
    context: &'static str,
}

/// Invalid state in the connectome has been detected by the burst engine.
/// This error is fatal for the burst engine and requires halting and user intervention
#[derive(FeagiErrorKey)]
pub struct FeagiFailConnectomeCorruption {
    context: &'static str,
}