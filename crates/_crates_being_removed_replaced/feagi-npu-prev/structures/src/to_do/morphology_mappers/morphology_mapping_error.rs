use feagi_structures::feagi_data::feagi_logging_and_errors::{generate_feagi_error, FeagiErrorKey, FeagiError};

#[derive(FeagiErrorKey)]
#[feagi_error(crate = "::feagi_structures::feagi_data::feagi_logging_and_errors")]
pub struct MorphologyMappingEtcErrKey {
    context: &'static str,
}

generate_feagi_error!{
    crate: "::feagi_structures::feagi_data::feagi_logging_and_errors",
    MorphologyMappingError,
    keys : {
        MappingEtc: MorphologyMappingEtcErrKey
    },
    sub_errors: {

    },
}