use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerCommandHandler, FeagiWorkerCommands};
use feagi_basis::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_spawner_implementation::FeagiWorkerSpawnError;
use feagi_cns_processing::genome_manager::worker_builder::{GenomeManagerConfigurables, GenomeManagerRequesters, GenomeManagerWorkerBuilder, GenomeManagerWorkerBuilderError};
use feagi_interop::transport::rest::server::error::FeagiRestServerError;
use feagi_interop::transport::rest::server::server_basis::RestServerWorkerBuilder;
use feagi_interop::transport::rest::server::server_std::{StdRestServerBuilder, StdRestServerWorker};
use feagi_interop::node::feagi_node_flags::FeagiNodeFlags;
use feagi_interop::node::feagi_role::{FeagiRole, FeagiRoles};
use feagi_interop::transport::zenoh::zenoh_configuration::ZenohConfiguration;
use feagi_interop::transport::zenoh::zenoh_worker_builder::{FeagiBuildZenohWorkerError, ZenohStdWorker, ZenohWorkerBuilder};

type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;

/// How many commands can be queued on the brain server worker at once. Only `Stop` is sent
const COMMAND_QUEUE_LENGTH: usize = 1;

//region Builder

/// Launches the genome manager, a REST server serving the genome manager's request categories,
/// and a zenoh worker announcing the brain server on the network. The worker it builds owns all
/// of them, and stops them when stopped
pub struct DesignerBrainServerWorkerBuilder {
    pub rest_server_socket: core::net::SocketAddr,
    pub zenoh_configuration: ZenohConfiguration,
}

/// The brain server worker. Stopping it stops the zenoh worker, the REST server, and the genome
/// manager
pub type DesignerBrainServerWorker = FeagiWorker<FeagiStdThreadSpawner, ChannelBackend, (), ()>;

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for DesignerBrainServerWorkerBuilder {
    type ChannelBackend = ChannelBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type CustomRequest = ();
    type CustomResponse = ();
    type BuildError = DesignerBrainServerLaunchError;

    fn build_worker(self, spawner: &FeagiStdThreadSpawner, _inputs: Self::BuildInputs) -> Result<(DesignerBrainServerWorker, Self::BuildOutputs), Self::BuildError> {
        // The genome manager makes the request channels of its categories, and keeps the responders
        let genome_manager_builder = GenomeManagerWorkerBuilder::default();
        let (genome_manager_requesters, genome_manager_responders) = genome_manager_builder.spawn_required_channel_pairs();
        let (genome_manager, ()) = genome_manager_builder.build_worker(spawner, genome_manager_responders)?;

        // The REST server gets the requesters, as a group. None of the genome manager's
        // categories have configurables yet. If it fails to start, the genome manager is dropped
        // here, which disconnects it so it exits on its own
        let configurables = GenomeManagerConfigurables::without_configurables();
        let (rest_server, ()) = StdRestServerBuilder::<GenomeManagerRequesters<ChannelBackend>>::new(self.rest_server_socket)
            .build_worker(spawner, (genome_manager_requesters, configurables))?;

        // Not connected to the other workers yet, it only announces the brain server. If it fails
        // to start, the workers above are dropped the same way
        let mut roles = FeagiRoles::new();
        roles.set_role(FeagiRole::FeagiBrainDesignerMode, true);
        let mut flags = FeagiNodeFlags::new();
        flags.set_roles(roles);
        let (zenoh, ()) = ZenohWorkerBuilder::new(self.zenoh_configuration, flags).build_worker(spawner, ())?;

        let (command_requester, command_responder) = ChannelBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let handle = spawner.spawn_worker(move || server_runtime(command_responder, genome_manager, rest_server, zenoh))?;
        Ok((FeagiWorker::new(command_requester, handle), ()))
    }

    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs {}
}

//endregion

//region Worker

/// Holds the workers the brain server runs, and stops them when the brain server is stopped
struct DesignerBrainServerCommandHandler {
    genome_manager: FeagiWorker<FeagiStdThreadSpawner, ChannelBackend, (), ()>,
    rest_server: StdRestServerWorker,
    zenoh: ZenohStdWorker,
}

impl FeagiWorkerCommandHandler for DesignerBrainServerCommandHandler {
    type CustomRequest = ();
    type CustomResponse = ();

    /// Zenoh is stopped first, so other nodes see the brain server leave before it stops serving.
    /// Then the REST server, so no new requests reach the genome manager while it stops
    async fn on_stop(&mut self) {
        // A worker that already exited can't be stopped, but is stopped all the same
        let _ = self.zenoh.stop().await;
        let _ = self.rest_server.stop().await;
        let _ = self.genome_manager.stop().await;
    }

    async fn on_custom(&mut self, _request: ()) {}
}

/// Answers commands until stopped (or its handle is dropped), then waits for the workers it runs
/// to exit
async fn server_runtime(
    command_responder: FeagiResponderAlloc<FeagiWorkerRequest<()>, FeagiWorkerResponse<()>>,
    genome_manager: FeagiWorker<FeagiStdThreadSpawner, ChannelBackend, (), ()>,
    rest_server: StdRestServerWorker,
    zenoh: ZenohStdWorker,
) {
    let mut handler = DesignerBrainServerCommandHandler { genome_manager, rest_server, zenoh };
    FeagiWorkerCommands::new(command_responder).run_until_stopped(&mut handler).await;

    // If the brain server's handle was dropped without stopping, joining drops each worker's
    // handle first, so they exit too
    let DesignerBrainServerCommandHandler { genome_manager, rest_server, zenoh } = handler;
    let _ = zenoh.join();
    let _ = rest_server.join();
    let _ = genome_manager.join();
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum DesignerBrainServerLaunchError {
    #[error("Genome manager failed to start: {0}")]
    GenomeManager(#[from] GenomeManagerWorkerBuilderError),
    #[error("REST server failed to start: {0}")]
    RestServer(#[from] FeagiRestServerError),
    #[error("Zenoh worker failed to start: {0}")]
    Zenoh(#[from] FeagiBuildZenohWorkerError),
    #[error("Brain server worker failed to start: {0}")]
    Spawn(#[from] FeagiWorkerSpawnError),
}
