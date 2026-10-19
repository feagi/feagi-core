


pub trait RestServerWorker {

    /// Requests queued for the server worker. Only `Stop` is ever sent
    const DEFAULT_REQUEST_QUEUE_CAPACITY: usize = 1;
    
    
    
    
    
}