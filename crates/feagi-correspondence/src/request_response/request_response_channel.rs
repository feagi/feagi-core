

/// The request with the means to send the response back
pub struct RequestEnvelope<Request, Response, Responder>
where
    Request: Send + 'static,
    Response: Send + 'static,
    Responder: ResponseChannel<Output=Response>
{
    pub request: Request,
    pub response_channel: Responder

}


/// The method in which responses are sent back for a request
pub trait ResponseChannel: Send + Sync + 'static {
    type Output: Send + 'static;

    // Consumes or uses the channel to send the data back
    fn send_response(self, response: Self::Output);
}

// TODO async non blocking?

//region STD Impl

#[derive(Clone)]
pub struct STDBlockingRequestMaker<Request, Response>
where
    Request: Send + 'static,
    Response: Send + 'static,
{
    request_tx: std::sync::mpsc::Sender<RequestEnvelope<
        Request,
        Response,
        std::sync::mpsc::Sender<Response>
    >>,
}

impl<Request, Response> STDBlockingRequestMaker<Request, Response>
where
    Request: Send + 'static,
    Response: Send + 'static,
{
    pub fn new(
        request_tx: std::sync::mpsc::Sender<
            RequestEnvelope<
                Request,
                Response,
                std::sync::mpsc::Sender<Response>
            >
        >) -> Self {
        Self {request_tx}
    }

    pub fn make_blocking_request(&self, request_data: Request) -> Result<Response, ()> { // TODO error handling

        let (tx, rx) = std::sync::mpsc::channel();
        let envelope = RequestEnvelope {
            request: request_data,
            response_channel: tx
        };
        
        self.request_tx.send(envelope).map_err(|e| ())?;

        // NOTE: This blocks until we get a response
        rx.recv().map_err(|e| ())
    }
}

// STD Impl
impl<Response: Send + 'static> ResponseChannel for std::sync::mpsc::Sender<Response> {
    type Output = Response;

    fn send_response(self, response: Self::Output) {
        // If the receiving end was dropped/canceled, we ignore the send error
        let _ = self.send(response);
    }
}




//endregion

// TODO no-std embassy impl