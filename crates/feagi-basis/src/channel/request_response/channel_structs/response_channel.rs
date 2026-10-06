use thingbuf::mpsc::{Receiver, Sender};
use thingbuf::mpsc::errors::TryRecvError;
use crate::channel::recycling::{Recyclable, RecyclableRecyclePolicy};
use crate::threading::thread_messaging::ChannelError;

// NOTE: Do NOT allow this to be clonable!
/// Allows processing of multiple incoming datas and sending responses via oneshots in an MPSC
/// pattern. Call 'loop_process_incoming_requests' in a  loop to poll for incoming requests,
/// process them, and then return the result
pub struct RequestResponder<Request, Response>
where
    Request: Send + Recyclable,
    Response: Send + Recyclable,
{
    incoming_requests: Receiver<
        (Request, Sender<Response, RecyclableRecyclePolicy<Response>>), 
        RecyclableRecyclePolicy<(Request, Sender<Response, RecyclableRecyclePolicy<Response>>)>
    >,
    // outgoing responses go over oneshot
}


impl<Request, Response> RequestResponder<Request, Response>
where
    Request: Send + Recyclable,
    Response: Send + Recyclable,
{

    pub(crate) fn new(incoming_requests: Receiver<(Request, Sender<Response, RecyclableRecyclePolicy<Response>>), RecyclableRecyclePolicy<(Request, Sender<Response, RecyclableRecyclePolicy<Response>>)>>) -> Self {
        Self {incoming_requests}
    }

    /*
    pub async fn loop_process_incoming_requests_mut<ProcessorFuncMut>(&self, mut processor_func: ProcessorFuncMut) -> Result<(), ChannelError>
    where
        ProcessorFuncMut: AsyncFnMut(Req) -> Res
    {
        self.loop_process_incoming_requests::<ProcessorFuncMut>(&self, processor_func).await
    }
     */

    /// To be called in a loop. If there is incoming data, it will call the given processor async
    /// function on it to generate a response. Otherwise, it will just return immediately
    pub async fn loop_process_incoming_requests<ProcessorFunc>(&self, processor_func: ProcessorFunc) -> Result<(), ChannelError>
    where
        ProcessorFunc: AsyncFn(Request) -> Response
    {
        let maybe_incoming = self.incoming_requests.try_recv();

        match maybe_incoming {
            Err(e) => {
                match e {
                    TryRecvError::Empty => {
                        return Ok(()) // nothing to do, return
                    }
                    TryRecvError::Closed => {
                        return Err(ChannelError::SendClosed);
                    }
                    _ => {
                        // For some reason the linter is forcing this, even though this is impossible??
                        unreachable!("This enum only has 2 errors?")
                    }
                }
            }
            Ok(incoming) => {
                let response = processor_func(incoming.0).await;
                let sender = incoming.1;
                _ = sender.send(response).await; // We do not care if the oneshot is closed
                drop(sender); // to be absolutely clear, destroy the oneshot returner
                Ok(())
            }
        }
    }
}