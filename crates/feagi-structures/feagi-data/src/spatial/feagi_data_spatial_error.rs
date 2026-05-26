

#[derive(Debug)]
pub enum FeagiDataSpatialError {
    InvalidCoordinate{
        context: &'static str
    },
    InvalidDimensions{
        context: &'static str
    },
}
