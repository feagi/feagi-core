// Top level error enum for this crate, holds errors from individual models

use crate::genomic::FeagiStructuresGenomicError;

#[derive(Debug)]
pub enum FeagiStructuresError {
    GenomicError { genomic_error: FeagiStructuresGenomicError},
    JSONError { context: &'static str},
    InvalidValue {context: &'static str}
}

// TODO automatic impls

// TODO error stuff