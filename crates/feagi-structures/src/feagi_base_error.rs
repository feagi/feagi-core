use crate::common_descriptors::{Coordinate2DU32, Coordinate3DU32, Dimension2DU32, Dimension3DU32};

pub enum FeagiBaseError {
    ValueCannotBeZero,
    ValuesMustBeEqual,
    Coordinate2DOutOfBounds{coordinate: Coordinate2DU32, dimensions: Dimension2DU32},
    Coordinate3DOutOfBounds{coordinate: Coordinate3DU32, dimensions: Dimension3DU32}
}