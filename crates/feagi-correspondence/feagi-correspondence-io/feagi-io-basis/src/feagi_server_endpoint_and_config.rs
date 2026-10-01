use crate::contracts::request_response::request_response_set::RequestResponseEndpointSenderSet;



/// For holding the configuration (such as address, port, etc) for a specific server endpoint,
/// which then can be consumed to launch the server as an event loop on a thread
pub trait FeagiServerEndpointConfig: Sized {
    type ServerStartError;
    // These types are unrestrained as they may be of type or of empty "()" (for disabling)
    /// Struct passing in channels for making requests
    type RequestEndpointSenderSet: RequestResponseEndpointSenderSet;
    /// Struct for passing in channels for high rate data transfer
    type DataExchangeSet; // TODO
    
    #[cfg(feature = "std")]
    /// Consumes itself to launch an event loop server that runs on another thread
    fn launch_server_thread(
        self,
        command_channel: thingbuf::mpsc::Receiver<FeagiServerEndpointCommand>
    ) -> Result<std::thread::JoinHandle<()>, Self::ServerStartError>;

    #[cfg(feature = "std")]
    /// Consumes itself to launch the `FeagiServerEndpointThreadHandle` directly
    fn launch_server_endpoint(self) -> Result<FeagiServerEndpoint, Self::ServerStartError> {
        let (command, comply) = thingbuf::mpsc::channel(1);
        let running_handle = self.launch_server_thread(comply)?;
        Ok(FeagiServerEndpoint {
            running_handle,
            command_channel: command
        })
        
    }

    // TODO launch server "thread" for embassy
}


/// Holds the actual server endpoint on a std thread
pub struct FeagiServerEndpoint {
    #[cfg(feature = "std")]
    running_handle: std::thread::JoinHandle<()>, // Never returns anything
    /// Can be used to send messages to the running server itelf. Mainly used for shutdown
    command_channel: thingbuf::mpsc::Sender<FeagiServerEndpointCommand>,
}

impl FeagiServerEndpoint {
    /// Consumes self and shuts down the server
    /// (sends a shutdown command, then blocks until the thread dies)
    pub async fn stop_server(self) -> () {
        _ = self.command_channel.send(FeagiServerEndpointCommand::StopServer).await;
        _ = self.running_handle.join(); // TODO force shutdown?
        ()
    }

    // TODO other commands?
}




/// Used to send commands to the running server
#[derive(Copy, Clone, Default)]
pub enum FeagiServerEndpointCommand {
    /// Ends the server loop, closing the thread
    #[default]
    StopServer
}

