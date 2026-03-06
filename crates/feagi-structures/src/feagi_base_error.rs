use crate::common_descriptors::{Coordinate2D, Coordinate3D, Dimension2D, Dimension3D};

pub enum FeagiBaseError {
    ValueCannotBeZero,
    ValuesMustBeEqual,
    Coordinate2DOutOfBounds{coordinate: Coordinate2D, dimensions: Dimension2D},
    Coordinate3DOutOfBounds{coordinate: Coordinate3D, dimensions: Dimension3D}
}