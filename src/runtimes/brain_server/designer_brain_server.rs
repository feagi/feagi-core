use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerCommandHandler, FeagiWorkerCommands, FeagiWorkerStep};
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
use crate::runtimes::brain_server::request_system::{
    ReadCorticalAreaVisualizationSkipRateFeagiRequestMessageResponse, ReadCorticalAreaVisualizationSuppressionThresholdFeagiRequestMessageResponse,
    ReadHealthCheckFeagiRequestMessageResponse, SystemRequestCategory, SystemRequests, SystemResponses,
};

type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;

/// The request categories the REST server serves: the genome manager's, and system, which the
/// brain server worker answers itself
type RestGroup = (GenomeManagerRequesters<ChannelBackend>, FeagiCategoryRequester<SystemRequestCategory, ChannelBackend>);

/// OpenAPI document of `RestGroup`, written at compile time
#[cfg(feature = "rest_open_api")]
const OPEN_API_JSON: &str = feagi_interop::feagi_open_api_json!(RestGroup, ChannelBackend);

/// How many commands can be queued on the brain server worker at once. Only `Stop` is sent
const COMMAND_QUEUE_LENGTH: usize = 1;

/// How many system requests can be queued on the brain server worker at once
const SYSTEM_REQUEST_QUEUE_LENGTH: usize = 4;

/// How long the brain server worker sleeps when no command or request is waiting. Bounds how
/// late a system request is answered while idle
const IDLE_POLL_INTERVAL: core::time::Duration = core::time::Duration::from_millis(1);

//region Builder

/// Launches the genome manager, a REST server serving the genome manager's and the system request
/// categories, and a zenoh worker announcing the brain server on the network. The worker it builds
/// owns all of them, answers system requests itself, and stops them when stopped
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

        // System requests are answered by the brain server worker itself, which keeps the responder
        let (system_requester, system_responder) = SystemRequestCategory::spawn_alloc_requester_responser_pair(SYSTEM_REQUEST_QUEUE_LENGTH);

        // The REST server gets the requesters, as a group. None of the categories have
        // configurables yet. If it fails to start, the genome manager is dropped here, which
        // disconnects it so it exits on its own
        let configurables = (
            GenomeManagerConfigurables::without_configurables(),
            SystemRequestCategory::EMPTY_CONFIGURABLES.expect("the system category declares no configurables"),
        );
        #[allow(unused_mut)]
        let mut rest_server_builder = StdRestServerBuilder::<RestGroup>::new(self.rest_server_socket);
        #[cfg(feature = "rest_open_api")]
        {
            rest_server_builder.open_api_json = Some(OPEN_API_JSON);
        }
        let (rest_server, ()) = rest_server_builder.build_worker(spawner, ((genome_manager_requesters, system_requester), configurables))?;

        // Not connected to the other workers yet, it only announces the brain server. If it fails
        // to start, the workers above are dropped the same way
        let mut roles = FeagiRoles::new();
        roles.set_role(FeagiRole::FeagiBrainDesignerMode, true);
        let mut flags = FeagiNodeFlags::new();
        flags.set_roles(roles);
        let (zenoh, ()) = ZenohWorkerBuilder::new(self.zenoh_configuration, flags).build_worker(spawner, ())?;

        let (command_requester, command_responder) = ChannelBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let handler = DesignerBrainServerCommandHandler { system: Some(system_responder), genome_manager, rest_server, zenoh };
        let handle = spawner.spawn_worker(move || server_runtime(command_responder, handler))?;
        Ok((FeagiWorker::new(command_requester, handle), ()))
    }

    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs {}
}

//endregion

//region Worker

/// Holds the workers the brain server runs and the system responder, and stops them when the
/// brain server is stopped
struct DesignerBrainServerCommandHandler {
    /// Taken when stopping, so system requests are no longer answered
    system: Option<FeagiCategoryResponder<SystemRequestCategory, ChannelBackend>>,
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
        // System requests are answered on this worker, which is busy stopping. Dropping the
        // responder fails any the REST server is waiting on, so it is free to stop
        self.system = None;
        let _ = self.rest_server.stop().await;
        let _ = self.genome_manager.stop().await;
    }

    async fn on_custom(&mut self, _request: ()) {}
}

impl DesignerBrainServerCommandHandler {
    /// Answers the next queued system request, if there is one. Returns whether one was answered
    async fn answer_system_request(&self) -> bool {
        let Some(system) = &self.system else {
            return false;
        };
        // An empty or closed channel just has nothing to answer
        system.try_process_next(async |request| system_response(&request)).await.is_ok()
    }
}

/// The response to a system request
// TODO actually handle system requests. Until then the requests Brain Visualizer makes on launch
// are answered with placeholder data, and every other request as not implemented, so requesters
// don't wait forever
fn system_response(request: &SystemRequests) -> SystemResponses {
    match request {
        SystemRequests::ReadHealthCheck(_) => SystemResponses::ReadHealthCheck(Ok(ReadHealthCheckFeagiRequestMessageResponse {
            burst_engine: true,
            connected_agents: None,
            influxdb_availability: false,
            neuron_count_max: 0,
            synapse_count_max: 0,
            latest_changes_saved_externally: false,
            genome_availability: true,
            genome_validity: Some(true),
            brain_readiness: true,
            genome_loading: false,
            genome_state: "loaded".to_string(),
            feagi_session: Some(1),
            fitness: None,
            cortical_area_count: None,
            neuron_count: Some(0),
            memory_neuron_count: None,
            regular_neuron_count: None,
            synapse_count: Some(0),
            estimated_brain_size_in_MB: None,
            genome_num: Some(1),
            genome_timestamp: None,
            simulation_timestep: Some(0.05),
            memory_area_stats: None,
            amalgamation_pending: None,
            brain_regions_hash: None,
            cortical_areas_hash: None,
            brain_geometry_hash: None,
            morphologies_hash: None,
            cortical_mappings_hash: None,
            agent_data_hash: None,
            brain_regions_root: None,
            fatigue: None,
        })),
        SystemRequests::ReadCorticalAreaVisualizationSkipRate(_) => {
            SystemResponses::ReadCorticalAreaVisualizationSkipRate(Ok(ReadCorticalAreaVisualizationSkipRateFeagiRequestMessageResponse { skip_rate: 1 }))
        }
        SystemRequests::ReadCorticalAreaVisualizationSuppressionThreshold(_) => SystemResponses::ReadCorticalAreaVisualizationSuppressionThreshold(Ok(
            ReadCorticalAreaVisualizationSuppressionThresholdFeagiRequestMessageResponse { threshold: 1 },
        )),
        _ => SystemRequestCategory::not_implemented_response(request),
    }
}

/// Answers commands and system requests until stopped (or its handle is dropped), then waits
/// for the workers it runs to exit
async fn server_runtime(
    command_responder: FeagiResponderAlloc<FeagiWorkerRequest<()>, FeagiWorkerResponse<()>>,
    mut handler: DesignerBrainServerCommandHandler,
) {
    let commands = FeagiWorkerCommands::new(command_responder);
    loop {
        let mut busy = match commands.try_handle_next(&mut handler).await {
            FeagiWorkerStep::Handled => true,
            FeagiWorkerStep::Idle => false,
            FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => break,
        };

        busy |= handler.answer_system_request().await;

        if !busy {
            std::thread::sleep(IDLE_POLL_INTERVAL);
        }
    }

    // If the brain server's handle was dropped without stopping, joining drops each worker's
    // handle first, so they exit too. The system responder goes first, so a REST request
    // waiting on it fails rather than holding the REST server up
    let DesignerBrainServerCommandHandler { system, genome_manager, rest_server, zenoh } = handler;
    drop(system);
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
