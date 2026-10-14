use crate::parallelism::feagi_channel::recycling::Recyclable;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GenericRequestResponseError<E: core::fmt::Debug + Sized + Send = ()> {
    #[error("The given request is not known, invalid, or is disabled")]
    UnknownRequest,
    #[error("Request was unable to be parsed")]
    FailedToParseRequest,
    #[error("Request channel is currently saturated! Is the system overloaded?")]
    RequestChannelIsSaturated,
    #[error("Request channel is currently closed! Has the program crashed?")]
    RequestChannelIsClosed,
    #[error(transparent)]
    Custom(E),
}

unsafe impl<E: core::fmt::Debug + Sized + Send> Send for GenericRequestResponseError<E> {}

impl<E: core::fmt::Debug + Sized + Send> Recyclable for GenericRequestResponseError<E> {
    fn new_for_recycler() -> Self {
        Self::UnknownRequest
    }

    fn recycle_self(&mut self) {
        // Nothing!
    }
}

