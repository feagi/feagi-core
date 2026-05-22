// Top level error enum for this crate, holds errors from individual models

#[derive(Debug)]
pub enum FeagiCommonError {
    JSONError { context: &'static str},
    InvalidValue {context: &'static str}
}


// TODO error stuff