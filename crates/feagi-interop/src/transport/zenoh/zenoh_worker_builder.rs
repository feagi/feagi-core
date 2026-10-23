use core::net::SocketAddr;
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use crate::node::feagi_node_flags::FeagiNodeFlags;
use crate::node::feagi_node_id::FeagiNodeID;
use crate::transport::zenoh::zenoh_configuration::ZenohConfiguration;
// TODO reconsider how we are doing this. A force connection block? vs scouting?
// TODO common trait to unite seperation?
// TODO std / no-std seperation, other feature seperation

pub struct ZenohWorkerBuilder {
    feagi_node_id: FeagiNodeID,
    zenoh_config: ZenohConfiguration,
    flags: FeagiNodeFlags,
}

impl ZenohWorkerBuilder {

    pub fn new(zenoh_config: ZenohConfiguration, flags: FeagiNodeFlags) -> Self {
        Self {
            feagi_node_id: FeagiNodeID::new_random(),
            zenoh_config,
            flags
        }
    }

    // TODO specific node id override

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