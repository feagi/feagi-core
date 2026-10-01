

/// For holding the configuration (such as address, port, etc) for a specific server endpoint,
/// which then can be consumed to launch the server as an event loop on a thread
pub trait FeagiServerEndpointConfig {

    // These types are unrestrained as they may be of type or of empty "()" (for disabling)
    
    /// Struct passing in channels for making requests
    type RequestModules;
    /// Struct for passing in channels for high rate data transfer
    type DataExchangeModules;
    
    #[cfg(feature = "std")]
    /// Consumes itself to launch an event loop server that runs on another thread
    fn launch_server_thread(
        self,
        command_channel: thingbuf::mpsc::Receiver<FeagiServerEndpointCommand>
    ) -> std::thread::JoinHandle<()>;
    
    // TODO launch server "thread" for embassy
}


#[derive(Copy, Clone, Default)]
pub enum FeagiServerEndpointCommand {
    /// Ends the server loop, returning
    #[default]
    StopServer
}
