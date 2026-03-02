pub enum FeagiBaseError {
    ValueCannotBeZero,
    Coordinate2DOutOfBounds{coordinate: &Coordinate2D, dimensions: &Dimension2D},
    Coordinate3DOutOfBounds{coordinate: &Coordinate3D, dimensions: &Dimension3D}
}