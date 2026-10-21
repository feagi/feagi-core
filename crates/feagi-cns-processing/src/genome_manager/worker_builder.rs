use feagi_basis::feagi_message::request::feagi_request_message_category::FeagiRequestCategory;
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::worker_spawner_implementation::WorkerSpawnerImplementation;
use crate::genome_manager::requests::categories;

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
impl<Spawner: WorkerSpawnerImplementation> FeagiWorkerBuilder<Spawner> for GenomeManagerWorkerBuilder {
    type ChannelBackend = FeagiRequestResponseChannelsAllocBackend;

    type BuildInputs = ();
    /// Genome Manager makes many request channels for the various categories it covers
    type BuildOutputs = ();
    /// This defines the channels of the various categories of request this worker goes through
    type RequiredChannelPairs = ();
    type CustomRequest = (); // TODO NPU launcher configuration
    type CustomResponse = ();
    type BuildError = GenomeManagerWorkerBuilderError;

    fn build_worker(self, spawner: &Spawner, inputs: Self::BuildInputs) -> Result<(FeagiWorker<Spawner, Self::ChannelBackend, Self::CustomRequest, Self::CustomResponse>, Self::BuildOutputs), Self::BuildError> {
        todo!()
    }

    fn spawn_required_channel_pairs(&self) -> Self::RequiredChannelPairs {
        let (burst_engine_req, burst_engine_res) =  categories::burst_engine::BurstEngineRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (connectome_req, connectome_res) =  categories::connectome::ConnectomeRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (cortical_area_req, cortical_area_res) =  categories::cortical_area::CorticalAreaRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (cortical_mapping_req, cortical_mapping_res) =  categories::cortical_mapping::CorticalMappingRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (genome_req, genome_res) =  categories::genome::GenomeRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (morphology_req, morphology_res) =  categories::morphology::MorphologyRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);
        let (region_req, region_res) =  categories::region::RegionRequestCategory::spawn_alloc_requester_responser_pair(self.burst_engine_request_queue_length);

        (
            (burst_engine_req, burst_engine_res),
            (connectome_req, connectome_res),
            (cortical_area_req, cortical_area_res),
            (cortical_mapping_req, cortical_mapping_res),
            (genome_req, genome_res),
            (morphology_req, morphology_res),
            (region_req, region_res),
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenomeManagerWorkerBuilderError {
    #[error("Unknown Start Error")]
    UnknownError
}