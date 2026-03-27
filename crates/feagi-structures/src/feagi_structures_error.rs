use crate::base_quantizable::coordinate::{Dimension2DType, Dimension3DType, UnsignedCoordinate2DType, UnsignedCoordinate3DType};

pub enum FeagiStructuresError {
    ValueCannotBeZero{context: &'static str},
    ValuesMustBeEqual{context: &'static str},
    Coordinate2DOutOfBounds{context: &'static str, coordinate: UnsignedCoordinate2DType<u32>, dimensions: Dimension2DType<u32>},
    Coordinate3DOutOfBounds{context: &'static str, coordinate: UnsignedCoordinate3DType<u32>, dimensions: Dimension3DType<u32>},
    JSONError{context: &'static str},
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for FeagiStructuresError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ValueCannotBeZero { context } => {
                write!(f, "value cannot be zero: {context}")
            }
            Self::ValuesMustBeEqual { context } => {
                write!(f, "values must be equal: {context}")
            }
            Self::Coordinate2DOutOfBounds {
                context,
                coordinate,
                dimensions,
            } => {
                write!(
                    f,
                    "2D coordinate out of bounds ({context}): coordinate {coordinate}, dimensions {dimensions}"
                )
            }
            Self::Coordinate3DOutOfBounds {
                context,
                coordinate,
                dimensions,
            } => {
                write!(
                    f,
                    "3D coordinate out of bounds ({context}): coordinate {coordinate}, dimensions {dimensions}"
                )
            }
            Self::JSONError { context } => write!(f, "JSON error: {context}"),
        }
    }
}

#[cfg(all(feature = "alloc", feature = "std"))]
impl std::error::Error for FeagiStructuresError {}