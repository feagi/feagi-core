use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiResponderAlloc;
use feagi_basis::parallelism::feagi_worker::worker_commands::{FeagiWorkerCommandHandler, FeagiWorkerCommands, FeagiWorkerStep};
use feagi_basis::parallelism::feagi_worker::worker_request_response::{FeagiWorkerRequest, FeagiWorkerResponse};
use crate::genome_manager::worker_builder::GenomeManagerResponders;

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
    genome_request_responders: GenomeManagerResponders
)
{
    
    let commands = FeagiWorkerCommands::new(responder);
    let mut command_handler = GenomeManagerCommandHandler;
    
    loop {
        match commands.try_handle_next(&mut command_handler).await {
            FeagiWorkerStep::Handled | FeagiWorkerStep::Idle => {}
            FeagiWorkerStep::Stopped | FeagiWorkerStep::Disconnected => break,
        }
    }
    
}