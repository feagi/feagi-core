use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum FeagiNetworkError {
    CannotBind(String),
}

impl Display for FeagiNetworkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FeagiNetworkError::CannotBind(msg) => {
                write!(f, "FeagiNetworkError: Unable to Bind! {}", msg)
            }
        }
    }
}

impl Error for FeagiNetworkError { // TODO
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}