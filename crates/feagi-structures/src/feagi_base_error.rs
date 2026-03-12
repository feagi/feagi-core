use crate::descriptors::{Dimension2DUSize, Dimension3DUSize, UnsignedCoordinate2DUSize, UnsignedCoordinate3DUSize};

pub enum FeagiBaseError {
    ValueCannotBeZero,
    ValuesMustBeEqual,
    Coordinate2DOutOfBounds{coordinate: UnsignedCoordinate2DUSize, dimensions: Dimension2DUSize},
    Coordinate3DOutOfBounds{coordinate: UnsignedCoordinate3DUSize, dimensions: Dimension3DUSize},
}