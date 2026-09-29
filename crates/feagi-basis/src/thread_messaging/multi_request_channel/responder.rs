use crate::thread_messaging::multi_request_channel::requester::PooledOneshotTransmitterChannel;

/// Channel in which Requests are passed into, as well as the oneshots for responses
pub trait RequestReceiverChannel<
    Req: Send,
    Res: Send,
    Oneshot: PooledOneshotTransmitterChannel<Res>
>: Clone {
    type Error;

    /// Pass in the request and the oneshot for the response
    async fn send_request_to_processor(
        &mut self, request: Req,
        oneshot: Oneshot)
        -> Result<(), Self::Error>;
}


/// Contains the Request Receiver channel and Calls the processing
pub trait RequestResponder<Res, Req, Oneshot>
where
    Req: Send,
    Res: Send,
    Oneshot: PooledOneshotTransmitterChannel<Res>,
{
    /// Awaits on the incoming channel, returns the Request and Oneshot
    async fn await_incoming_requests(&mut self) -> (Req, Oneshot);
    
    /// The actual processing. This assumes the data needed is on the struct itself
    async fn process_request_to_response(&mut self, request: Req) -> Res;
    
    /// To be called in a loop. Waits for any requests, processes them, then returns the response
    /// through the given oneshot before tossing it
    async fn loop_request_processing(&mut self) {
        let (request, mut oneshot)  = self
            .await_incoming_requests()
            .await;
        let response = self.process_request_to_response(request).await;
        oneshot.return_response_through_oneshot(response).await;
    }
}