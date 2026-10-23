use core::net::SocketAddrV4;
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use crate::node::feagi_node_id::FeagiNodeID;
// TODO reconsider how we are doing this. A force connection block? vs scouting?
// TODO common trait to unite seperation?
// TODO std / no-std seperation, other feature seperation

pub struct ZenohWorkerBuilder {
    feagi_node_id: FeagiNodeID,
    ws_server_bind: Option<SocketAddrV4>,
    ws_client_bind: Option<SocketAddrV4>
}

impl ZenohWorkerBuilder {

    pub fn new_no_config() -> Self {
        Self {
            feagi_node_id: FeagiNodeID::new_random(),
            ws_server_bind: None,
            ws_client_bind: None,
        }
    }



    // TODO specific node id override

    pub fn set_ws_server_bind(&mut self, socket: SocketAddrV4) {
        self.ws_server_bind = Some(socket)
    }

    pub fn set_ws_client_bind(&mut self, socket: SocketAddrV4) {
        self.ws_client_bind = Some(socket)
    }

}


impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for ZenohWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type CustomRequest = ();
    type CustomResponse = ();
    type BuildError = FeagiBuildZenohWorkerError;

    fn build_worker(self, spawner: &FeagiStdThreadSpawner, inputs: Self::BuildInputs) -> Result<(FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, Self::CustomRequest, Self::CustomResponse>, Self::BuildOutputs), Self::BuildError> {
        todo!()
    }

    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs {
        ()
    }
}


#[derive(thiserror::Error, Debug)]
pub enum FeagiBuildZenohWorkerError {
    #[error("Server failed to bind to socket!")] // TODO socket?
    FailedToBindToSocket,
    #[error("Server failed to configure its socket!")]
    FailedToConfigureSocket,
}