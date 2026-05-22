

#[derive(Debug)]
pub enum FeagiDataSpatialError {
    InvalidCoordinate{
        context: &'static str
    },
}
