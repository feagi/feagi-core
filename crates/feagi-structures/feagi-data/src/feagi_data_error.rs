use crate::feagi_error::FeagiErrorBase;
use crate::quantizable::FeagiDataQuantizedError;
use crate::spatial::FeagiDataSpatialError;
// NOTE: Yes this is in the root of the crate, so we can establish this as a pattern

/// Root error for the Feagi-Data crate
pub enum FeagiDataError {
    QuantizationError(FeagiDataQuantizedError),
    SpatialError(FeagiDataSpatialError)
}

impl FeagiErrorBase for FeagiDataError {}

