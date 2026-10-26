use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;
use crate::workers::genome_manager::requests::categories::burst_engine::{BurstEngineRequestCategory, BurstEngineRequests, BurstEngineResponses, ReadSimulationTimestepFeagiRequestMessageResponse};

pub(crate) async fn burst_engine_response(
    responder: &FeagiCategoryResponder<BurstEngineRequestCategory, FeagiRequestResponseChannelsAllocBackend>) -> bool {

    responder.try_process_next(
        async | request: BurstEngineRequests | {
           match &request {
                BurstEngineRequests::Nothing => {
                    BurstEngineResponses::Nothing
                }
                BurstEngineRequests::ReadSimulationTimestep(_) => {
                    BurstEngineResponses::ReadSimulationTimestep(Ok(ReadSimulationTimestepFeagiRequestMessageResponse {
                        simulation_timestep: 0.05,
                    }))
                }
                BurstEngineRequests::ReadStats(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadStatus(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadBurstCounter(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadConfig(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadFrequencyStatus(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateSimulationTimestep(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateControl(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateStart(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateStop(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateHold(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateResume(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::EditConfig(_) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
            }
        }
    ).await.is_ok()
}
