use crate::base_quantizable::coordinate::{Dimension2DUSize, Dimension3DUSize, UnsignedCoordinate2DUSize, UnsignedCoordinate3DUSize};

pub enum FeagiStructuresError {
    ValueCannotBeZero{context: &'static str},
    ValuesMustBeEqual{context: &'static str},
    Coordinate2DOutOfBounds{context: &'static str, coordinate: UnsignedCoordinate2DUSize, dimensions: Dimension2DUSize},
    Coordinate3DOutOfBounds{context: &'static str, coordinate: UnsignedCoordinate3DUSize, dimensions: Dimension3DUSize},
    JSONError{context: &'static str},
}