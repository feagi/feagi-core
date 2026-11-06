use crate::runtimes::brain_server::request_system::{
    ReadCorticalAreaVisualizationSkipRateFeagiRequestMessageResponse, ReadCorticalAreaVisualizationSuppressionThresholdFeagiRequestMessageResponse,
    ReadHealthCheckFeagiRequestMessageResponse, SystemRequestCategory, SystemRequests, SystemResponses,
};
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryRequester, FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::FeagiWorkerSpawnError;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::{FeagiWorkerPollingRouter, FeagiWorkerStep};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::target_frequency::TargetFrequency;
use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;
use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_connectome::connectome_basis::engine::{BurstEngineCPURayon, BurstEngineCPURayonInitError};
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::NeuronModelFeagiAdvancedDefaultQuant;
use feagi_connectome::connectome_basis::neuron_model::foundation_power::NeuronModelFoundationPowerFloat32;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_connectome::neural_processors::singular::{
    NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse, NeuronProcessorSingleEngineVisualizationReceiver,
    NeuronProcessorSingleEngineVisualizationTransmitter, NeuronProcessorSingleEngineWorkerBuilder, NeuronProcessorSingleEngineWorkerBuilderError,
};
use feagi_genome::basis::specifications::brain_regions::BrainRegionID;
use feagi_genome::cortical_area::foundation::Power;
use feagi_genome::cortical_area::processing::Interconnect;
use feagi_genome::cortical_area::CorticalAreaTypeEnum;
use feagi_genome::genome::containers::cortical_area::{CorticalAreaGenerator, CorticalAreaGeneratorEnum};
use feagi_genome::genome::genome_editing::{GenomeEditSequence, GenomeEditStepResult};
use feagi_genome::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_genome::workers::genome_manager::requests::category_structs::{GenomeManagerConfigurables, GenomeManagerRequesters};
use feagi_genome::workers::genome_manager::worker_builder::{GenomeManagerWorkerBuilder, GenomeManagerWorkerBuilderError};
use feagi_genome::workers::genome_manager::worker_loop::{GenomeManagerRequest, GenomeManagerResponse};
use feagi_interop::node::feagi_node_flags::FeagiNodeFlags;
use feagi_interop::node::feagi_role::{FeagiRole, FeagiRoles};
use feagi_interop::transport::rest::server::error::FeagiRestServerError;
use feagi_interop::transport::rest::server::server_basis::RestServerWorkerBuilder;
use feagi_interop::transport::rest::server::server_std::{StdRestServerBuilder, StdRestServerWorker};
use feagi_interop::transport::zenoh::zenoh_configuration::ZenohConfiguration;
use feagi_interop::transport::zenoh::zenoh_worker_builder::{FeagiBuildZenohWorkerError, ZenohStdWorker, ZenohWorkerBuilder};

type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;

/// The quantization of the brain server's connectome
type FIQ = FeagiIndexQuantizationStandard;

/// The burst engine the neural processor runs
type BurstEngine = BurstEngineCPURayon<FIQ, NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>>;

/// The neural processor worker, running the burst engine
type NeuralProcessorWorker = FeagiWorker<FeagiStdThreadSpawner, ChannelBackend, NeuronProcessorSingleEngineRequest<FIQ>, NeuronProcessorSingleEngineResponse<FIQ>>;

/// The genome manager worker, editing the neural processor's connectome
type GenomeManagerWorker = FeagiWorker<FeagiStdThreadSpawner, ChannelBackend, GenomeManagerRequest<FIQ>, GenomeManagerResponse>;

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

/// How often the neural processor bursts at launch, matching the `simulation_timestep` reported by
/// the health check
const INITIAL_BURST_FREQUENCY_HZ: f64 = 20.0;

/// How often the neural processor sends visualization at launch
const INITIAL_VISUALIZATION_FREQUENCY_HZ: f64 = 20.0;

//region Builder

/// Launches the genome manager, a neural processor running a CPU burst engine, a REST server
/// serving the genome manager's and the system request categories, and a zenoh worker announcing
/// the brain server on the network. The worker it builds owns all of them, answers system requests
/// itself, and stops them when stopped
pub struct DesignerBrainServerWorkerBuilder {
    pub rest_server_socket: core::net::SocketAddr,
    pub zenoh_configuration: ZenohConfiguration,
}

/// The brain server worker. Stopping it stops the zenoh worker, the REST server, the neural
/// processor, and the genome manager
pub type DesignerBrainServerWorker = FeagiWorker<FeagiStdThreadSpawner, ChannelBackend, (), ()>;

impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for DesignerBrainServerWorkerBuilder {
    type ChannelBackend = ChannelBackend;
    type BuildInputs = ();
    type BuildOutputs = ();
    type RequiredChannelPairs = ();
    type WorkerCommandHandler = DesignerBrainServerCommandHandler;
    type WorkerStartError = DesignerBrainServerLaunchError;

    fn build_worker(self, spawner: &FeagiStdThreadSpawner, _inputs: Self::BuildInputs) -> Result<(DesignerBrainServerWorker, Self::BuildOutputs), WorkerStartError<Self::WorkerStartError>> {
        // Starts with no cortical areas, which the genome manager adds. Its visualization receiver
        // is held until it is wired to zenoh
        let neural_processor_builder = NeuronProcessorSingleEngineWorkerBuilder::<FIQ, BurstEngine> {
            initial_target_frequency: TargetFrequency::new_from_frequency(INITIAL_BURST_FREQUENCY_HZ),
            initial_visualization_frequency: TargetFrequency::new_from_frequency(INITIAL_VISUALIZATION_FREQUENCY_HZ),
            engine_init_parameters: (),
        };
        let (visualization_transmitter, visualization_receiver) = neural_processor_builder.spawn_related_channel_pairs();
        let (neural_processor, neural_processor_commands) = neural_processor_builder
            .build_worker(spawner, visualization_transmitter)
            .map_err(|error| WorkerStartError::WorkerError(error.into()))?;

        // The genome manager makes the request channels of its categories, and keeps the responders.
        // It edits the neural processor's connectome through a clone of its command channel
        let genome_manager_builder = GenomeManagerWorkerBuilder::<FIQ>::default();
        let (genome_manager_requesters, genome_manager_responders) = genome_manager_builder.spawn_related_channel_pairs();
        let (genome_manager, ()) = genome_manager_builder
            .build_worker(spawner, (genome_manager_responders, neural_processor_commands))
            .map_err(|error| WorkerStartError::WorkerError(error.into()))?;

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
        let (rest_server, ()) = rest_server_builder
            .build_worker(spawner, ((genome_manager_requesters, system_requester), configurables))
            .map_err(|error| WorkerStartError::WorkerError(error.into()))?;

        // Not connected to the other workers yet, it only announces the brain server. If it fails
        // to start, the workers above are dropped the same way
        let mut roles = FeagiRoles::new();
        roles.set_role(FeagiRole::FeagiBrainDesignerMode, true);
        let mut flags = FeagiNodeFlags::new();
        flags.set_roles(roles);
        let (zenoh, ()) = ZenohWorkerBuilder::new(self.zenoh_configuration, flags)
            .build_worker(spawner, ())
            .map_err(|error| WorkerStartError::WorkerError(error.into()))?;

        let (command_requester, command_responder) = ChannelBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let handler = DesignerBrainServerCommandHandler {
            system: Some(system_responder),
            genome_manager,
            neural_processor,
            visualization_receiver,
            rest_server,
            zenoh,
        };
        let handle = spawner.spawn_worker(move || server_runtime(command_responder, handler))?;
        Ok((FeagiWorker::new(command_requester, handle), ()))
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {}
}

//endregion

//region Worker

/// Holds the workers the brain server runs and the system responder, and stops them when the
/// brain server is stopped
pub struct DesignerBrainServerCommandHandler {
    /// Taken when stopping, so system requests are no longer answered
    system: Option<FeagiCategoryResponder<SystemRequestCategory, ChannelBackend>>,
    genome_manager: GenomeManagerWorker,
    neural_processor: NeuralProcessorWorker,
    // TODO wire to zenoh as a publishing event. Until then nothing reads it, so the neural
    // processor skips visualization once its slots fill
    #[allow(dead_code)]
    visualization_receiver: NeuronProcessorSingleEngineVisualizationReceiver<FIQ>,
    rest_server: StdRestServerWorker,
    zenoh: ZenohStdWorker,
}

impl FeagiWorkerCommandProcessor for DesignerBrainServerCommandHandler {
    type CustomRequest = ();
    type CustomResponse = ();

    /// Zenoh is stopped first, so other nodes see the brain server leave before it stops serving.
    /// Then the REST server, so no new requests reach the genome manager while it stops. The
    /// neural processor is stopped before the genome manager, which will be editing it
    async fn on_stop(&mut self) {
        // A worker that already exited can't be stopped, but is stopped all the same
        let _ = self.zenoh.stop().await;
        // System requests are answered on this worker, which is busy stopping. Dropping the
        // responder fails any the REST server is waiting on, so it is free to stop
        self.system = None;
        let _ = self.rest_server.stop().await;
        let _ = self.neural_processor.stop().await;
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
    // TODO placeholder until a genome is loaded or sent by a client
    build_example_brain(&mut handler.genome_manager).await;

    let mut commands = FeagiWorkerPollingRouter::new(handler, command_responder);
    loop {
        let mut busy = match commands.try_handle_next().await {
            FeagiWorkerStep::Handled => true,
            FeagiWorkerStep::Idle => false,
            FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => break,
        };

        busy |= commands.processor_mut().answer_system_request().await;

        if !busy {
            std::thread::sleep(IDLE_POLL_INTERVAL);
        }
    }

    // If the brain server's handle was dropped without stopping, joining drops each worker's
    // handle first, so they exit too. The system responder goes first, so a REST request
    // waiting on it fails rather than holding the REST server up
    let DesignerBrainServerCommandHandler { system, genome_manager, neural_processor, visualization_receiver, rest_server, zenoh } =
        commands.into_processor();
    drop(system);
    drop(visualization_receiver);
    let _ = zenoh.join();
    let _ = rest_server.join();
    let _ = neural_processor.join();
    let _ = genome_manager.join();
}

//endregion

//region Example Brain

/// TODO placeholder. Builds a small brain through the genome manager's command channel, one edit
/// sequence at a time: a brain region under the root, a 1x1x1 power area in the root, and a
/// 10x10x10 FEAGI advanced interconnect area in the new region. A step that fails is reported,
/// and the steps relying on it are skipped
async fn build_example_brain(genome_manager: &mut GenomeManagerWorker) {
    let root = match genome_manager.make_request(GenomeManagerRequest::RootBrainRegion).await {
        Ok(GenomeManagerResponse::RootBrainRegion(root)) => root,
        _ => {
            eprintln!("Example brain: could not read the root brain region");
            return;
        }
    };

    let mut sequence = GenomeEditSequence::new();
    let _ = sequence.brain_region().add_now(root, example_metadata("Example Region"), example_position([0, 0, 0]));
    let region = match apply_edit_sequence(genome_manager, sequence).await.as_deref() {
        Ok([GenomeEditStepResult::BrainRegionAdded(region)]) => *region,
        other => {
            eprintln!("Example brain: could not add the brain region: {other:?}");
            return;
        }
    };
    println!("Example brain: added brain region {region:?} under the root brain region {root:?}");

    // The defaults of each cortical type, only the dimensions differ
    match CorticalAreaGenerator::<FIQ, NeuronModelFoundationPowerFloat32>::from_defaults::<Power>(example_voxels([1, 1, 1])) {
        Ok(power) => {
            let power: CorticalAreaGeneratorEnum<FIQ> = CorticalAreaTypeEnum::from_data::<Power, NeuronModelFoundationPowerFloat32>(power);
            add_example_cortical_area(genome_manager, power, root, "Example Power", [0, 0, 0]).await
        }
        Err(error) => eprintln!("Example brain: could not make the power area: {error}"),
    }
    match CorticalAreaGenerator::<FIQ, NeuronModelFeagiAdvancedDefaultQuant>::from_defaults::<Interconnect>(example_voxels([10, 10, 10])) {
        Ok(interconnect) => {
            let interconnect: CorticalAreaGeneratorEnum<FIQ> = CorticalAreaTypeEnum::from_data::<Interconnect, NeuronModelFeagiAdvancedDefaultQuant>(interconnect);
            add_example_cortical_area(genome_manager, interconnect, region, "Example Interconnect", [5, 0, 0]).await
        }
        Err(error) => eprintln!("Example brain: could not make the interconnect area: {error}"),
    }
}

/// An area of `[x, y, z]` voxels, one neuron deep
fn example_voxels([x, y, z]: [usize; 3]) -> DimensionalCorticalAreaDimensions<u32> {
    DimensionalCorticalAreaDimensions::new_dimensions_from_usize([x, y, z, 1]).expect("example dimensions are valid")
}

async fn add_example_cortical_area(
    genome_manager: &mut GenomeManagerWorker,
    generator: CorticalAreaGeneratorEnum<FIQ>,
    parent_region: BrainRegionID,
    name: &str,
    position: [i64; 3],
) {
    let mut sequence = GenomeEditSequence::new();
    // Faux identifiers are not used yet
    let _ = sequence.cortical_area().add(0, generator, parent_region, example_metadata(name), example_position(position));
    match apply_edit_sequence(genome_manager, sequence).await.as_deref() {
        Ok([GenomeEditStepResult::CorticalAreaAdded(area)]) => println!("Example brain: added cortical area '{name}' {area:?} in brain region {parent_region:?}"),
        other => eprintln!("Example brain: could not add cortical area '{name}': {other:?}"),
    }
}

async fn apply_edit_sequence(genome_manager: &mut GenomeManagerWorker, sequence: GenomeEditSequence<FIQ>) -> Result<Vec<GenomeEditStepResult>, String> {
    match genome_manager.make_request(GenomeManagerRequest::ApplyEditSequence(sequence)).await {
        Ok(GenomeManagerResponse::EditSequenceApplied(results)) => results.map_err(|error| error.to_string()),
        Ok(_) => Err("the genome manager answered with an unexpected response".to_string()),
        Err(error) => Err(error.to_string()),
    }
}

fn example_metadata(name: &str) -> GenomeObjectMetadata {
    GenomeObjectMetadata { name: name.to_string(), description: String::new() }
}

fn example_position(position_3d: [i64; 3]) -> GenomeObjectPositioning {
    let [x, y, _] = position_3d;
    GenomeObjectPositioning {
        position_2d: SignedIntegerSpatial::new_from_array([x, y]),
        position_3d: SignedIntegerSpatial::new_from_array(position_3d),
    }
}

//endregion

#[derive(Debug, thiserror::Error)]
pub enum DesignerBrainServerLaunchError {
    #[error("Genome manager failed to start: {0}")]
    GenomeManager(#[from] WorkerStartError<GenomeManagerWorkerBuilderError>),
    #[error("Neural processor failed to start: {0}")]
    NeuralProcessor(#[from] WorkerStartError<NeuronProcessorSingleEngineWorkerBuilderError<BurstEngineCPURayonInitError>>),
    #[error("REST server failed to start: {0}")]
    RestServer(#[from] WorkerStartError<FeagiRestServerError>),
    #[error("Zenoh worker failed to start: {0}")]
    Zenoh(#[from] WorkerStartError<FeagiBuildZenohWorkerError>),
    #[error("Brain server worker failed to start: {0}")]
    Spawn(#[from] FeagiWorkerSpawnError),
}
