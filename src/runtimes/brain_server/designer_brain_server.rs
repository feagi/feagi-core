use feagi_basis::feagi_message::request::feagi_request_message_category::FeagiCategoryRequester;
use feagi_basis::feagi_message::request::feagi_request_message_grouping::{NCat, NNull};
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerCommandHandler, FeagiWorkerCommands};
use feagi_basis::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_spawner_implementation::FeagiWorkerSpawnError;
use feagi_cns_processing::genome_manager::requests::categories::burst_engine::{BurstEngineConfigurables, BurstEngineRequestCategory};
use feagi_cns_processing::genome_manager::requests::categories::connectome::{ConnectomeConfigurables, ConnectomeRequestCategory};
use feagi_cns_processing::genome_manager::requests::categories::cortical_area::{CorticalAreaConfigurables, CorticalAreaRequestCategory};
use feagi_cns_processing::genome_manager::requests::categories::cortical_mapping::{CorticalMappingConfigurables, CorticalMappingRequestCategory};
use feagi_cns_processing::genome_manager::requests::categories::genome::{GenomeConfigurables, GenomeRequestCategory};
use feagi_cns_processing::genome_manager::requests::categories::morphology::{MorphologyConfigurables, MorphologyRequestCategory};
use feagi_cns_processing::genome_manager::requests::categories::region::{RegionConfigurables, RegionRequestCategory};
use feagi_cns_processing::genome_manager::worker_builder::{GenomeManagerRequesters, GenomeManagerWorkerBuilder, GenomeManagerWorkerBuilderError};
use feagi_interop::transport::rest::server::error::FeagiRestServerError;
use feagi_interop::transport::rest::server::server_basis::RestServerWorkerBuilder;
use feagi_interop::transport::rest::server::server_std::{StdRestServerBuilder, StdRestServerWorker};

type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;

/// How many commands can be queued on the brain server worker at once. Only `Stop` is sent
const COMMAND_QUEUE_LENGTH: usize = 1;

//region Request Group

/// Every request category the genome manager serves, as the group the REST server serves
type DesignerRequestGroup = NCat<
    FeagiCategoryRequester<BurstEngineRequestCategory, ChannelBackend>,
    NCat<
        FeagiCategoryRequester<ConnectomeRequestCategory, ChannelBackend>,
        NCat<
            FeagiCategoryRequester<CorticalAreaRequestCategory, ChannelBackend>,
            NCat<
                FeagiCategoryRequester<CorticalMappingRequestCategory, ChannelBackend>,
                NCat<
                    FeagiCategoryRequester<GenomeRequestCategory, ChannelBackend>,
                    NCat<FeagiCategoryRequester<MorphologyRequestCategory, ChannelBackend>, NCat<FeagiCategoryRequester<RegionRequestCategory, ChannelBackend>, NNull>>,
                >,
            >,
        >,
    >,
>;

/// The configurables of every category in `DesignerRequestGroup`, nested the same way
type DesignerRequestGroupConfigurables = NCat<
    &'static BurstEngineConfigurables,
    NCat<
        &'static ConnectomeConfigurables,
        NCat<
            &'static CorticalAreaConfigurables,
            NCat<
                &'static CorticalMappingConfigurables,
                NCat<&'static GenomeConfigurables, NCat<&'static MorphologyConfigurables, NCat<&'static RegionConfigurables, NNull>>>,
            >,
        >,
    >,
>;

// None of the categories have configurables yet
static BURST_ENGINE_CONFIGURABLES: BurstEngineConfigurables = BurstEngineConfigurables {};
static CONNECTOME_CONFIGURABLES: ConnectomeConfigurables = ConnectomeConfigurables {};
static CORTICAL_AREA_CONFIGURABLES: CorticalAreaConfigurables = CorticalAreaConfigurables {};
static CORTICAL_MAPPING_CONFIGURABLES: CorticalMappingConfigurables = CorticalMappingConfigurables {};
static GENOME_CONFIGURABLES: GenomeConfigurables = GenomeConfigurables {};
static MORPHOLOGY_CONFIGURABLES: MorphologyConfigurables = MorphologyConfigurables {};
static REGION_CONFIGURABLES: RegionConfigurables = RegionConfigurables {};

/// Puts the genome manager's requesters in a group, with the configurables of their categories
fn designer_request_group(requesters: GenomeManagerRequesters) -> (DesignerRequestGroup, DesignerRequestGroupConfigurables) {
    let group = NCat {
        head: requesters.burst_engine,
        tail: NCat {
            head: requesters.connectome,
            tail: NCat {
                head: requesters.cortical_area,
                tail: NCat {
                    head: requesters.cortical_mapping,
                    tail: NCat {
                        head: requesters.genome,
                        tail: NCat {
                            head: requesters.morphology,
                            tail: NCat { head: requesters.region, tail: NNull },
                        },
                    },
                },
            },
        },
    };
    let configurables = NCat {
        head: &BURST_ENGINE_CONFIGURABLES,
        tail: NCat {
            head: &CONNECTOME_CONFIGURABLES,
            tail: NCat {
                head: &CORTICAL_AREA_CONFIGURABLES,
                tail: NCat {
                    head: &CORTICAL_MAPPING_CONFIGURABLES,
                    tail: NCat {
                        head: &GENOME_CONFIGURABLES,
                        tail: NCat {
                            head: &MORPHOLOGY_CONFIGURABLES,
                            tail: NCat { head: &REGION_CONFIGURABLES, tail: NNull },
                        },
                    },
                },
            },
        },
    };
    (group, configurables)
}

//endregion

//region Builder

/// Launches the genome manager, and a REST server serving the genome manager's request
/// categories. The worker it builds owns both, and stops them when stopped
pub struct DesignerBrainServerWorkerBuilder {
    pub rest_server_socket: core::net::SocketAddr,
}

/// The brain server worker. Stopping it stops the REST server and the genome manager
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

        // The REST server gets the requesters, as a group. If it fails to start, the genome
        // manager is dropped here, which disconnects it so it exits on its own
        let (group, configurables) = designer_request_group(genome_manager_requesters);
        let (rest_server, ()) = StdRestServerBuilder::<DesignerRequestGroup>::new(self.rest_server_socket).build_worker(spawner, (group, configurables))?;

        let (command_requester, command_responder) = ChannelBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let handle = spawner.spawn_worker(move || server_runtime(command_responder, genome_manager, rest_server))?;
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
}

impl FeagiWorkerCommandHandler for DesignerBrainServerCommandHandler {
    type CustomRequest = ();
    type CustomResponse = ();

    /// The REST server is stopped first, so no new requests reach the genome manager while it stops
    async fn on_stop(&mut self) {
        // A worker that already exited can't be stopped, but is stopped all the same
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
) {
    let mut handler = DesignerBrainServerCommandHandler { genome_manager, rest_server };
    FeagiWorkerCommands::new(command_responder).run_until_stopped(&mut handler).await;

    // If the brain server's handle was dropped without stopping, joining drops each worker's
    // handle first, so they exit too
    let DesignerBrainServerCommandHandler { genome_manager, rest_server } = handler;
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
    #[error("Brain server worker failed to start: {0}")]
    Spawn(#[from] FeagiWorkerSpawnError),
}
