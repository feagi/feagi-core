
/// Used for spawning thread functions in `GenericThreadWorker`
pub trait GenericThreadWorkerSpawner<FinalResult: Send = (), CustomCommand: Send + Clone = ()> {
    type ThreadSpawnError: core::error::Error;
    
    fn spawn_thread_worker(
        self,
        command_receiver: thingbuf::mpsc::Receiver<GenericThreadWorkerCommand<CustomCommand>>
    ) -> Result<std::thread::JoinHandle<FinalResult>, Self::ThreadSpawnError>;
}

/// Used as a generic to handle threads. Can take a custom command type to allow sending more commands (including oneshot response commands)
pub struct GenericThreadWorker<FinalResult: Send = (), CustomCommand: Send + Clone = ()> {
    handle: std::thread::JoinHandle<FinalResult>,
    command_sender: thingbuf::mpsc::Sender<GenericThreadWorkerCommand<CustomCommand>>
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
        _ = self.command_sender.send(GenericThreadWorkerCommand::Stop).await;
        let result = self.handle.join();
        result.unwrap() // NO
    }
    
}


/// The command system wrapper that enforces stop as a requirement
#[derive(Clone, Default)]
pub enum GenericThreadWorkerCommand<CustomCommand: Send + Clone> {
    #[default]
    Stop,
    Custom(CustomCommand)
}

unsafe impl<CustomCommand: Send + Clone> Send for GenericThreadWorkerCommand<CustomCommand> {}