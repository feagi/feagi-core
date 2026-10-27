use crate::workers::genome_manager::worker_loop::{run_genome_manager, GenomeManagerCommandHandler};
use feagi_basis::feagi_message::request::feagi_request_message_category::FeagiRequestCategory;
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::{FeagiWorkerBuilder, WorkerStartError};
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use crate::workers::genome_manager::requests::category_structs::{GenomeManagerRequesters, GenomeManagerResponders};

/// How many commands can be in queue on this worker at one time
const COMMAND_QUEUE_LENGTH: usize = 2;

/// Responsible for spawning the genome manager, which handles high level genome processing
/// and is the proxy through which connectomes are modified
pub struct GenomeManagerWorkerBuilder {
    pub burst_engine_request_queue_length: usize,
    pub connectome_request_queue_length: usize,
    pub cortical_area_request_queue_length: usize,
    pub cortical_mapping_request_queue_length: usize,
    pub genome_queue_length: usize,
    pub morphology_queue_length: usize,
    pub region_queue_length: usize,
}

impl Default for GenomeManagerWorkerBuilder {
       
    fn default() -> Self {
        Self {
            burst_engine_request_queue_length: 8,
            connectome_request_queue_length: 8,
            cortical_area_request_queue_length: 8,
            cortical_mapping_request_queue_length: 8,
            genome_queue_length: 8,
            morphology_queue_length: 8,
            region_queue_length: 8,
        }
    }
}

// This is STD thread only for the time being
impl FeagiWorkerBuilder<FeagiStdThreadSpawner> for GenomeManagerWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;
    type BuildInputs = GenomeManagerResponders<FeagiRequestResponseChannelsAllocBackend>;
    type BuildOutputs = ();
    /// This defines the channels of the various categories of request this worker goes through
    type RequiredChannelPairs = (
        GenomeManagerRequesters<FeagiRequestResponseChannelsAllocBackend>,
        GenomeManagerResponders<FeagiRequestResponseChannelsAllocBackend>,
    );
    type WorkerCommandHandler = GenomeManagerCommandHandler; // TODO NPU launcher configuration
    type WorkerStartError = GenomeManagerWorkerBuilderError;

    fn build_worker(self, spawner: &FeagiStdThreadSpawner, inputs: Self::BuildInputs) -> Result<(FeagiWorker<FeagiStdThreadSpawner, Self::ChannelBackend, (), ()>, Self::BuildOutputs), WorkerStartError<Self::WorkerStartError>> {
        
        let (responders) = inputs;
        let (cmd_requester, cmd_responder) = FeagiRequestResponseChannelsAllocBackend::create_channels(COMMAND_QUEUE_LENGTH);
        let handle = spawner.spawn_worker(move || run_genome_manager(cmd_responder, responders)).unwrap(); // TODO actual error handling
        Ok((FeagiWorker::new(cmd_requester, handle), ()))
    }

    fn spawn_related_channel_pairs(&self) -> Self::RequiredChannelPairs {
        let (burst_engine_req, burst_engine_res) = BurstEngineRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (connectome_req, connectome_res) = ConnectomeRequestCategory::spawn_alloc_requester_responser_pair(self.connectome_request_queue_length);
        let (cortical_area_req, cortical_area_res) = CorticalAreaRequestCategory::spawn_alloc_requester_responser_pair(self.cortical_area_request_queue_length);
        let (cortical_mapping_req, cortical_mapping_res) = CorticalMappingRequestCategory::spawn_alloc_requester_responser_pair(self.cortical_mapping_request_queue_length);
        let (genome_req, genome_res) = GenomeRequestCategory::spawn_alloc_requester_responser_pair(self.genome_queue_length);
        let (morphology_req, morphology_res) = MorphologyRequestCategory::spawn_alloc_requester_responser_pair(self.morphology_queue_length);
        let (region_req, region_res) = RegionRequestCategory::spawn_alloc_requester_responser_pair(self.region_queue_length);

        let requesters = GenomeManagerRequesters {
            burst_engine: burst_engine_req,
            connectome: connectome_req,
            cortical_area: cortical_area_req,
            cortical_mapping: cortical_mapping_req,
            genome: genome_req,
            morphology: morphology_req,
            region: region_req,
        };
        let responders = GenomeManagerResponders {
            burst_engine: burst_engine_res,
            connectome: connectome_res,
            cortical_area: cortical_area_res,
            cortical_mapping: cortical_mapping_res,
            genome: genome_res,
            morphology: morphology_res,
            region: region_res,
        };
        (requesters, responders)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenomeManagerWorkerBuilderError {
    #[error(transparent)]
    ThreadError(std::io::Error),
    #[error("Unknown Start Error")]
    UnknownError
}

impl From<std::io::Error> for GenomeManagerWorkerBuilderError {
    fn from(v: std::io::Error) -> Self {GenomeManagerWorkerBuilderError::ThreadError(v)}
}

