use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParDataError {
    #[error("A generic parallel-data index was out of bounds")]
    ParDataInvalidIndex { index: usize },
    #[error("A generic parallel-data sub-range was out of bounds or otherwise invalid")]
    ParDataInvalidRange { start: usize, end: usize },
}
