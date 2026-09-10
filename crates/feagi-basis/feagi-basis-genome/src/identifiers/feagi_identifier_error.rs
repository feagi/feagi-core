use feagi_basis_error_logging::prelude::*;

#[derive(FeagiFail)]
pub struct FeagiFailCorticalID {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailMappingID {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailBrainRegionID {
    context: &'static str,
}

generate_feagi_error! {
    FeagiGenomeIdenfitierError,
    keys: {
        CorticalID: FeagiFailCorticalID,
        MappingID: FeagiFailMappingID,
        BrainRegionID: FeagiFailBrainRegionID,
    },
    sub_errors: {

    },
}
