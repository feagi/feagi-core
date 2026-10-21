use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use feagi_cns_processing::genome_manager::worker_builder::GenomeManagerWorkerBuilder;
use feagi_interop::transport::rest::server::server_basis::RestServerWorkerBuilder;
use feagi_interop::transport::rest::server::server_std::StdRestServerBuilder;

pub struct DesignerBrainServerWorkerBuilder {
    pub rest_server_socket: core::net::SocketAddr
}

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for DesignerBrainServerWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type CustomRequest = ();
    type CustomResponse = ();
    type BuildError = DesignerBrainServerLaunchError;

    fn build_worker(self, spawner: &FeagiStdThreadSpawner, inputs: Self::BuildInputs) -> Result<(FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, Self::CustomRequest, Self::CustomResponse>, Self::BuildOutputs), Self::BuildError> {
        let (cmd_requester, cmd_responder) = FeagiRequestResponseChannelsAllocBackend::create_channels(1);

        let genome_manager_builder = GenomeManagerWorkerBuilder::default();
        let (genome_manager_requesters, genome_manager_responders) = genome_manager_builder.spawn_required_channel_pairs();


        let picoserve_builder = StdRestServerBuilder::new(self.rest_server_socket);



    }

    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs {
        ()
    }
}

fn server_runtime() {

    
}

#[derive(Debug, thiserror::Error)]
pub enum DesignerBrainServerLaunchError {
    #[error("Unknown Start Error")]
    UnknownError
}


