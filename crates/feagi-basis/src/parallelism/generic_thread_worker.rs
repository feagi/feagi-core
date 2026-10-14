/// Used for spawning thread functions in `GenericThreadWorker`
pub trait GenericThreadWorkerSpawner<FinalResult: Send = (), CustomCommand: Send + Clone = ()> {
    type ThreadSpawnError: core::error::Error;
    
    fn spawn_thread_worker(
        self,
        command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerNotification<CustomCommand>>
    ) -> Result<std::thread::JoinHandle<FinalResult>, Self::ThreadSpawnError>;
}

/// Used as a generic to handle threads. Can take a custom command type to allow sending more commands (including oneshot response commands)
pub struct GenericThreadWorker<FinalResult: Send = (), CustomCommand: Send + Clone = ()> {
    handle: std::thread::JoinHandle<FinalResult>,
    command_sender: thingbuf::mpsc::Sender<GenericThreadWorkerNotification<CustomCommand>>
}

impl<FinalResult: Send, CustomCommand: Send + Clone> GenericThreadWorker<FinalResult, CustomCommand> {
    
    pub fn new_and_start<WorkerSpawner>(spawner: WorkerSpawner) -> Result<Self, WorkerSpawner::ThreadSpawnError>
    where 
        WorkerSpawner: GenericThreadWorkerSpawner<FinalResult, CustomCommand>
    {
        let (sender, receiver) = thingbuf::mpsc::channel(1);
        
        let handle = spawner.spawn_thread_worker(receiver)?;
        
        Ok(Self {
            handle,
            command_sender: sender
        })
    }
    
    pub async fn stop_thread(self) -> FinalResult {
        _ = self.command_sender.send(GenericThreadWorkerNotification::Stop).await;
        let result = self.handle.join();
        result.unwrap() // NO
    }
    
}


/// The command system wrapper that enforces stop as a requirement
#[derive(Clone, Default)]
pub enum GenericThreadWorkerNotification<CustomCommand: Send + Clone> {
    #[default]
    /// Do nothing, no effect. Mainly here to simplify pattern matching
    Nothing,
    /// Stop the worker
    Stop,
    /// Channel Closed, worker likely needs to be stopped
    ChannelClosed,
    /// Some specific command for the worker
    Custom(CustomCommand)
}

impl<CustomCommand: Send + Clone> From<Result<GenericThreadWorkerNotification<CustomCommand>, thingbuf::mpsc::errors::TryRecvError>>
for GenericThreadWorkerNotification<CustomCommand> {
    fn from(value: Result<GenericThreadWorkerNotification<CustomCommand>, thingbuf::mpsc::errors::TryRecvError>) -> Self {
        match value {
            Ok(command) => {command}
            Err(err) => {
                match err {
                    thingbuf::mpsc::errors::TryRecvError::Empty => {GenericThreadWorkerNotification::Nothing}
                    thingbuf::mpsc::errors::TryRecvError::Closed => {GenericThreadWorkerNotification::ChannelClosed}
                    _ => {GenericThreadWorkerNotification::ChannelClosed } // Not sure how this can happen?
                }
            }
        }
    }
}

unsafe impl<CustomCommand: Send + Clone> Send for GenericThreadWorkerNotification<CustomCommand> {}