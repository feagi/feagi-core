use feagi_data::feagi_logging_and_errors::{FeagiError, FeagiErrorKey, generate_feagi_error};
use feagi_logging_and_errors::cortical_area_properties::cortical_id::CorticalIDPacked;

extern crate self as feagi_logging_and_errors;

#[derive(FeagiErrorKey)]
pub struct CorticalIDEtcErrKey {
    context: &'static str
}

#[derive(FeagiErrorKey)]
pub struct CorticalIDLookupErrKey {
    context: &'static str,
    given_bytes: [u8; CorticalIDPacked::BYTE_COUNT]
}

#[derive(FeagiErrorKey)]
pub struct CorticalAreaErrKey {
    context: &'static str
}

#[derive(FeagiErrorKey)]
pub struct CorticalUnitErrKey {
    context: &'static str
}

generate_feagi_error!(
    FeagiGenomeDefinitionsError,
    keys: {
        CorticalIdEtcError: CorticalIDEtcErrKey,
        CorticalIdLookupError: CorticalIDLookupErrKey,
        CorticalAreaError: CorticalAreaErrKey,
        CorticalUnitError: CorticalUnitErrKey,
    },
    sub_errors: {

    }
);
