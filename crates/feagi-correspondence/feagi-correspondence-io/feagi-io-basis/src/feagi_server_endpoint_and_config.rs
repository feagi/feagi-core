use core::net::SocketAddrV4;
use feagi_basis::prelude::*;
use crate::contracts::request_response::request_response_set::RequestResponseEndpointSenderSet;



/// For holding the configuration (such as address, port, etc) for a specific server endpoint,
/// which then can be consumed to launch the server as an event loop on a thread
pub trait FeagiServerEndpointConfig: Sized {
    /// Struct passing in channels for making requests
    type RequestEndpointSenderSet: RequestResponseEndpointSenderSet;
    /// Struct for passing in channels for high rate data transfer
    type DataExchangeSet; // TODO
}

/// Consumes a FeagiServerEndpointConfig to lauch a server
pub trait FeagiServerEndpointLauncher<Config: FeagiServerEndpointConfig> {
    #[cfg(feature = "std")]
    /// Consumes the config to launch an event loop server that runs on another thread
    fn launch_server_thread(
        config: Config,
        command_channel: thingbuf::mpsc::Receiver<FeagiServerEndpointCommand>,
    ) -> Result<std::thread::JoinHandle<()>, FeagiRestServerStartError>;

    #[cfg(feature = "std")]
    /// Consumes the config to launch the `FeagiServerEndpointThreadHandle` directly
    fn launch_server_endpoint(config: Config) -> Result<FeagiServerEndpoint, FeagiRestServerStartError> {
        let (command, comply) = thingbuf::mpsc::channel(1);
        let running_handle = Self::launch_server_thread(config, comply)?;
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
        #[cfg(feature = "std")]
        {
            _ = self.running_handle.join(); // TODO force shutdown?
        }
        #[cfg(not(feature = "std"))]
        {
            // TODO no-std
        }
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

//region Server Start Error

#[derive(FeagiFail)]
pub struct FeagiFailStartRestServerEtc {
    context: &'static str,
}

#[derive(FeagiFail)]
pub struct FeagiFailStartRestServerCannotBind {
    context: &'static str,
    failed_to_bind: SocketAddrV4
}

generate_feagi_error! {
    FeagiRestServerStartError,
    keys: {
        Etc: FeagiFailStartRestServerEtc,
        CannotBind: FeagiFailStartRestServerCannotBind,
    },
    sub_errors: {

    },
}

//endregion