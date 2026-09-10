use feagi_basis_error_logging::prelude::*;
use crate::identifiers::feagi_identifier_error::FeagiGenomeIdenfitierError;

#[derive(FeagiFail)]
pub struct FeagiFailGenomeEtc {
    context: &'static str,
}


generate_feagi_error! {
    FeagiGenomeContextError,
    keys: {
        Etc: FeagiFailGenomeEtc,
    },
    sub_errors: {
        Identifier: FeagiGenomeIdenfitierError
    },
}
