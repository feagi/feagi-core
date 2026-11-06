use crate::genome::genomes::designer_genome::DesignerGenome;
use crate::workers::genome_manager::requests::category_structs::GenomeManagerResponders;
use crate::workers::genome_manager::response_processing::burst_engine::burst_engine_response;
use crate::workers::genome_manager::response_processing::cortical_area::cortical_area_response;
use crate::workers::genome_manager::response_processing::morphology::morphology_response;
use crate::workers::genome_manager::response_processing::region::region_response;
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_processor::FeagiWorkerCommandProcessor;
use feagi_basis::parallelism::feagi_worker::worker_internals::feagi_worker_command_router::{FeagiWorkerPollingRouter, FeagiWorkerStep};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::genome::genome_editing::{apply_genome_edit_sequence, GenomeEditError, GenomeEditSequence, GenomeEditStepResult, NeuralProcessorRequester};

/// How long the genome manager sleeps when no command or request is waiting. Bounds how late a
/// request is answered while idle
const IDLE_POLL_INTERVAL: core::time::Duration = core::time::Duration::from_millis(1);

/// Custom requests to the genome manager, through its command channel
pub enum GenomeManagerRequest<FIQ: FeagiIndexQuantization> {
    /// The ID of the root brain region, which every other is within
    RootBrainRegion,
    /// Apply a genome edit sequence, editing the connectome of the neural processor as needed
    ApplyEditSequence(GenomeEditSequence<FIQ>),
}

/// Custom responses from the genome manager, one per kind of request
pub enum GenomeManagerResponse {
    RootBrainRegion(BrainRegionID),
    /// What each step of the sequence did, in order, or why it was not applied
    EditSequenceApplied(Result<Vec<GenomeEditStepResult>, GenomeEditError>),
}

/// The state of the genome manager: the genome, and the command channel of the neural processor
/// whose connectome it edits
pub struct GenomeManagerCommandHandler<FIQ: FeagiIndexQuantization> {
    genome: DesignerGenome<FIQ>,
    neural_processor: NeuralProcessorRequester<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> FeagiWorkerCommandProcessor for GenomeManagerCommandHandler<FIQ> {
    type CustomRequest = GenomeManagerRequest<FIQ>;
    type CustomResponse = GenomeManagerResponse;

    async fn on_custom(&mut self, request: Self::CustomRequest) -> Self::CustomResponse {
        match request {
            GenomeManagerRequest::RootBrainRegion => GenomeManagerResponse::RootBrainRegion(self.genome.root_brain_region_id()),
            GenomeManagerRequest::ApplyEditSequence(sequence) => {
                GenomeManagerResponse::EditSequenceApplied(apply_genome_edit_sequence(&mut self.genome, &mut self.neural_processor, sequence).await)
            }
        }
    }
}

/// The actual genome manager loop. Listens for various requests on the state of the genome, or
/// for edits, and then acts upon them
pub(crate) async fn run_genome_manager<FIQ: FeagiIndexQuantization>(
    responder: FeagiResponderAlloc<FeagiWorkerRequest<GenomeManagerRequest<FIQ>>, FeagiWorkerResponse<GenomeManagerResponse>>,
    genome_request_responders: GenomeManagerResponders<FeagiRequestResponseChannelsAllocBackend>,
    neural_processor: NeuralProcessorRequester<FIQ>,
)
{
    let handler = GenomeManagerCommandHandler {
        genome: DesignerGenome::<FIQ>::init("Genome Test Name".to_string()),
        neural_processor,
    };
    let mut commands = FeagiWorkerPollingRouter::new(handler, responder);
    let GenomeManagerResponders { burst_engine, connectome, cortical_area, cortical_mapping, genome, morphology, region } = genome_request_responders;

    loop {
        let mut busy = match commands.try_handle_next().await {
            FeagiWorkerStep::Handled => true,
            FeagiWorkerStep::Idle => false,
            FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => break,
        };

        // TODO actually handle requests. Until then the unhandled ones are answered as not
        // implemented, so requesters don't wait forever
        busy |= burst_engine_response(&burst_engine).await;
        busy |= answer_not_implemented(&connectome).await;
        busy |= cortical_area_response(&cortical_area, &mut commands.processor_mut().genome).await;
        busy |= answer_not_implemented(&cortical_mapping).await;
        busy |= answer_not_implemented(&genome).await;
        busy |= morphology_response(&morphology).await;
        busy |= region_response(&region).await;

        if !busy {
            std::thread::sleep(IDLE_POLL_INTERVAL);
        }
    }

}

/// Answers the next queued request of `Category`, if there is one, with its message's not
/// implemented error. Returns whether a request was answered
async fn answer_not_implemented<Category: FeagiRequestCategory>(responder: &FeagiCategoryResponder<Category, FeagiRequestResponseChannelsAllocBackend>) -> bool {
    // An empty or closed channel just has nothing to answer
    responder.try_answer_not_implemented().await.is_ok()
}



