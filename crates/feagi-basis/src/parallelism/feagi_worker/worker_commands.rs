use crate::parallelism::feagi_channel::recycling::Recyclable;

/// A request from a worker handle to its worker. Establishes universal requests but also
/// allows custom ones
pub enum FeagiWorkerRequest<CustomRequest> {
    /// Do nothing, no effect. Also the value of an unused channel slot
    Nothing,
    /// Stop the worker. The worker replies before it exits
    Stop,
    /// Some specific request for the worker
    Custom(CustomRequest),
}


impl<CustomRequest> Recyclable for FeagiWorkerRequest<CustomRequest> {
    fn new_for_recycler() -> Self {
        FeagiWorkerRequest::Nothing
    }

    fn recycle_self(&mut self) {
        *self = FeagiWorkerRequest::Nothing;
    }
}

/// A worker's reply to a request
pub enum FeagiWorkerResponse<CustomReply> {
    /// No reply content. Also the value of an unused channel slot
    Nothing,
    /// The request was received and acted upon
    Acknowledged,
    /// Reply to a custom request
    Custom(CustomReply),
}

impl<CustomReply> Recyclable for FeagiWorkerResponse<CustomReply> {
    fn new_for_recycler() -> Self {
        FeagiWorkerResponse::Nothing
    }

    fn recycle_self(&mut self) {
        *self = FeagiWorkerResponse::Nothing;
    }
}
