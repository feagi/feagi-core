use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantIndexedError {
    #[error("A generic parallel-data index was out of bounds")]
    QuantIndexedInvalidIndex { index: usize },
    #[error("A generic parallel-data sub-range was out of bounds or otherwise invalid")]
    QuantIndexedInvalidRange { start: usize, end: usize },
}
