

#[derive(Debug)]
pub enum FeagiStructuresGenomicError {
    BrainRegionError { context: &'static str },
    CorticalAreaError { context: &'static str },
    CorticalIdError { context: &'static str },
    CorticalUnitError { context: &'static str },
}

impl core::fmt::Display for FeagiStructuresGenomicError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BrainRegionError { context } => write!(f, "{context}"),
            Self::CorticalAreaError { context } => write!(f, "{context}"),
            Self::CorticalIdError { context } => write!(f, "{context}"),
            Self::CorticalUnitError { context } => write!(f, "{context}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FeagiStructuresGenomicError {}
