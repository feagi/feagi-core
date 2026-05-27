use feagi_data::feagi_logging_and_errors::{FeagiError, FeagiErrorKey, generate_feagi_error};
extern crate self as feagi_logging_and_errors;

#[derive(FeagiErrorKey)]
pub struct CorticalIDErrKey {
    context: &'static str
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
        cortical_id_error: CorticalIDErrKey,
        cortical_area_error: CorticalAreaErrKey,
        cortical_unit_error: CorticalUnitErrKey,
    },
    sub_errors: {

    }
);
