use crate::node::feagi_node_flags::FeagiNodeFlags;
use crate::node::feagi_node_id::{Base64EncodedFeagiNodeID, FeagiNodeID};
use crate::transport::zenoh::zenoh_configuration::{ZenohConfiguration, ZenohConfigurationBackend};
use crate::transport::zenoh::zenoh_liveliness_path::{FeagiNodeLivelinessPath, FeagiNodeLivelinessPathConfigurables};
use crate::transport::zenoh::zenoh_std::configuration_backend::ZenohStdConfigurationBackend;
use crate::transport::zenoh::zenoh_std::zenoh_configuration_builder::error::ZenohConfigBuildError;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::FeagiWorkerSpawnError;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::FeagiWorkerAwaitingRouter;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use zenoh::Wait;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
// TODO reconsider how we are doing this. A force connection block? vs scouting?
// TODO common trait to unite seperation?
// TODO std / no-std seperation, other feature seperation

/// Requests queued for the zenoh worker. Only `Stop` is ever sent so it doesn't need to be long here
const REQUEST_QUEUE_CAPACITY: usize = 1;

/// How long to wait for the network to answer whether the node ID is already taken
const NODE_ID_CHECK_TIMEOUT: core::time::Duration = core::time::Duration::from_secs(2);

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

    /// The ID the node announces itself with
    pub fn feagi_node_id(&self) -> FeagiNodeID {
        self.feagi_node_id
    }

}

/// The zenoh worker (std). Holds the zenoh session and the node's liveliness token for as long as
/// it runs
pub type ZenohStdWorker = FeagiWorker<FeagiStdThreadSpawner, FeagiRequestResponseChannelsAllocBackend, (), ()>;

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for ZenohWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type WorkerCommandHandler = ZenohCommandHandler;
    type WorkerStartError = FeagiBuildZenohWorkerError;

    /// Opens the zenoh session, checks no other node has the same ID, and declares the node's
    /// liveliness token, all on the calling thread so failures are returned here. Then starts the
    /// worker thread, which holds the session and token
    fn build_worker(self, spawner: &FeagiStdThreadSpawner, _inputs: Self::BuildInputs) -> Result<(ZenohStdWorker, Self::BuildOutputs), WorkerStartError<Self::WorkerStartError>> {
        let zenoh_config = ZenohStdConfigurationBackend::build_zenoh_config(self.zenoh_config)
            .map_err(|error| WorkerStartError::WorkerError(error.into()))?;
        // The sync API runs on zenoh's own runtime, so it works on threads without one
        let session = zenoh::open(zenoh_config).wait().map_err(|error| WorkerStartError::WorkerError(FeagiBuildZenohWorkerError::FailedToOpenSession(error)))?;

        let liveliness_configurables = FeagiNodeLivelinessPathConfigurables::new(self.feagi_node_id);
        let replies = session
            .liveliness()
            .get(FeagiNodeLivelinessPath::wildcard_key(&liveliness_configurables).as_str())
            .timeout(NODE_ID_CHECK_TIMEOUT)
            .wait()
            .map_err(|error| WorkerStartError::WorkerError(FeagiBuildZenohWorkerError::FailedToCheckNodeID(error)))?;
        // Ends once the query is answered or times out
        while let Ok(reply) = replies.recv() {
            let Ok(sample) = reply.result() else { continue };
            if FeagiNodeLivelinessPath::parse_key(&liveliness_configurables, sample.key_expr().as_str()).is_some() {
                return Err(WorkerStartError::WorkerError(FeagiBuildZenohWorkerError::NodeIDTaken(self.feagi_node_id)));
            }
        }

        let token = session
            .liveliness()
            .declare_token(FeagiNodeLivelinessPath::key(&liveliness_configurables, self.flags).as_str())
            .wait()
            .map_err(|error| WorkerStartError::WorkerError(FeagiBuildZenohWorkerError::FailedToDeclareLivelinessToken(error)))?;

        let (requester, requests) = FeagiRequestResponseChannelsAllocBackend::create_channels(REQUEST_QUEUE_CAPACITY);
        let handle = spawner.spawn_worker(move || run_zenoh_worker(requests, session, token))?;
        Ok((FeagiWorker::new(requester, handle), ()))
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {
        ()
    }
}

/// Holds the session and token. Stopping withdraws the token and closes the session, so other
/// nodes see this node leave before the stop is acknowledged
pub struct ZenohCommandHandler {
    session: zenoh::Session,
    token: Option<zenoh::liveliness::LivelinessToken>,
}

impl FeagiWorkerCommandProcessor for ZenohCommandHandler {
    // No custom requests are defined
    type CustomRequest = ();
    type CustomResponse = ();

    async fn on_stop(&mut self) {
        // Nothing to do if these fail, the node is leaving either way
        if let Some(token) = self.token.take() {
            let _ = token.undeclare().wait();
        }
        let _ = self.session.close().wait();
    }

    async fn on_custom(&mut self, _request: ()) {}
}

/// Answers commands until stopped or the worker handle is dropped. Dropping the session and token
/// on exit closes and withdraws them if `Stop` never came
async fn run_zenoh_worker(
    requests: FeagiResponderAlloc<FeagiWorkerRequest<()>, FeagiWorkerResponse<()>>,
    session: zenoh::Session,
    token: zenoh::liveliness::LivelinessToken,
) {
    let handler = ZenohCommandHandler { session, token: Some(token) };
    FeagiWorkerAwaitingRouter::new(handler, requests).run_until_stopped().await;
}


#[derive(thiserror::Error, Debug)]
pub enum FeagiBuildZenohWorkerError {
    #[error("Zenoh configuration is invalid: {0}")]
    InvalidConfiguration(#[from] ZenohConfigBuildError),
    #[error("Zenoh session failed to open: {0}")]
    FailedToOpenSession(zenoh::Error),
    #[error("Failed to check whether the node ID is taken: {0}")]
    FailedToCheckNodeID(zenoh::Error),
    #[error("Another node already has the node ID {}", Base64EncodedFeagiNodeID::from(.0))]
    NodeIDTaken(FeagiNodeID),
    #[error("Failed to declare the liveliness token: {0}")]
    FailedToDeclareLivelinessToken(zenoh::Error),
    #[error("Zenoh worker failed to start: {0}")]
    FailedToStartWorker(#[from] FeagiWorkerSpawnError),
}
