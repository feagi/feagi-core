use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::{FeagiRequestResponseChannelsAllocBackend, FeagiResponderAlloc};
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerCommandHandler, FeagiWorkerCommands, FeagiWorkerStep};
use feagi_basis::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};
use feagi_cns_structures::feagi_brain_models::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_cns_structures::graphs::implementations::designer_graph::DesignerGraph;
use crate::genome_manager::response_processing::burst_engine::burst_engine_response;
use crate::genome_manager::response_processing::cortical_area::cortical_area_response;
use crate::genome_manager::response_processing::morphology::morphology_response;
use crate::genome_manager::response_processing::region::region_response;
use crate::genome_manager::worker_builder::GenomeManagerResponders;

/// How long the genome manager sleeps when no command or request is waiting. Bounds how late a
/// request is answered while idle
const IDLE_POLL_INTERVAL: core::time::Duration = core::time::Duration::from_millis(1);

struct GenomeManagerCommandHandler;

impl FeagiWorkerCommandHandler for GenomeManagerCommandHandler {
    type CustomRequest = ();
    type CustomResponse = ();

    async fn on_custom(&mut self, _request: Self::CustomRequest) -> Self::CustomResponse {
        // TODO custom command handling
    }
}

/// The actual genome manager loop. Listens for various requests on the state of the genome, or
/// for edits, and then acts upon them
pub(crate) async fn run_genome_manager(
    responder: FeagiResponderAlloc<FeagiWorkerRequest<()>, FeagiWorkerResponse<()>>,
    genome_request_responders: GenomeManagerResponders<FeagiRequestResponseChannelsAllocBackend>
)
{

    let mut graph = DesignerGraph::<FeagiIndexQuantizationStandard>::new("Blank Genome".to_string());

    let commands = FeagiWorkerCommands::new(responder);
    let mut command_handler = GenomeManagerCommandHandler;
    let GenomeManagerResponders { burst_engine, connectome, cortical_area, cortical_mapping, genome, morphology, region } = genome_request_responders;

    loop {
        let mut busy = match commands.try_handle_next(&mut command_handler).await {
            FeagiWorkerStep::Handled => true,
            FeagiWorkerStep::Idle => false,
            FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => break,
        };

        // TODO actually handle requests. Until then the unhandled ones are answered as not
        // implemented, so requesters don't wait forever
        busy |= burst_engine_response(&burst_engine).await;
        busy |= answer_not_implemented(&connectome).await;
        busy |= cortical_area_response(&cortical_area, &mut graph).await;
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



