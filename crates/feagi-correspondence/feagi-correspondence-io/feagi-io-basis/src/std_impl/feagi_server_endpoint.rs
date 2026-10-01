use crate::configurations::{FeagiServerEndpointCommand, FeagiServerEndpointConfig};

/// Holds the actual server system on a std thread
pub struct FeagiServerEndpointThreadHandle {
    running_handle: std::thread::JoinHandle<()>, // Never returns anything
    /// Can be used to send messages to the running server itelf. Mainly used for shutdown
    command_channel: thingbuf::mpsc::Sender<FeagiServerEndpointCommand>,
}

impl FeagiServerEndpointThreadHandle {

    pub fn launch_new_server<FSEC: FeagiServerEndpointConfig>(config: FSEC) -> Self {
        let (sender, receiver) = thingbuf::mpsc::channel(1);
        let running_handle = config.launch_server_thread(receiver);
        Self {
            running_handle,
            command_channel: sender
        }
    }

    /// Consumes self and shuts down the server
    /// (sends a shutdown command, then blocks until the thread dies)
    pub async fn stop_server(self) -> () {
        _ = self.command_channel.send(FeagiServerEndpointCommand::StopServer).await;
        _ = self.running_handle.join(); // TODO force shutdown?
        ()
    }

}

