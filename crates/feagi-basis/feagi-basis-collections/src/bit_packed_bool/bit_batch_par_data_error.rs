use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitBatchParDataError {
    #[error("A bit-batch word index was out of bounds")]
    BitBatchParDataInvalidIndex { index: usize },
    #[error("A bit-batch word sub-range was out of bounds or otherwise invalid")]
    BitBatchParDataInvalidRange { start: usize, end: usize },
}
