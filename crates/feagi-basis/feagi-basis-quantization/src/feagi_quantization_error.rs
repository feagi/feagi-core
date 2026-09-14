use feagi_basis_error_logging::prelude::*;
use crate::generic_collections::feagi_data_collections_error::FeagiDataCollectionError;
use crate::values::quantizable::FeagiDataValueQuantizationError;
use crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError;

#[derive(FeagiFail)]
/// Some other quantization related error occured
pub struct FeagiFailQuantizationEtc {
    context: &'static str,
}


generate_feagi_error! {
    /// An error related to some quantizable collection for FEAGI
    FeagiQuantizationError,
    keys: {
        Etc: FeagiFailQuantizationEtc,
    },
    sub_errors: {
        DataCollection: FeagiDataCollectionError,
        Spatial: FeagiDataValuesSpatialError,
        Value: FeagiDataValueQuantizationError
    },
}