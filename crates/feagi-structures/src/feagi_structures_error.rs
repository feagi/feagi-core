use crate::base_quantizable::coordinate::{Dimension2DType, Dimension3DType, UnsignedCoordinate2DType, UnsignedCoordinate3DType};

pub enum FeagiStructuresError {
    ValueCannotBeZero{context: &'static str},
    ValuesMustBeEqual{context: &'static str},
    Coordinate2DOutOfBounds{context: &'static str, coordinate: UnsignedCoordinate2DType<u32>, dimensions: Dimension2DType<u32>},
    Coordinate3DOutOfBounds{context: &'static str, coordinate: UnsignedCoordinate3DType<u32>, dimensions: Dimension3DType<u32>},
    JSONError{context: &'static str},
}