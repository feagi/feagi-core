use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use crate::workers::genome_manager::requests::categories::prelude::*;

pub(crate) async fn burst_engine_response(
    responder: &FeagiCategoryResponder<BurstEngineRequestCategory, FeagiRequestResponseChannelsAllocBackend>) -> bool {

    responder.try_process_next(
        async | request: BurstEngineRequests | {
           match &request {
                BurstEngineRequests::Nothing => {
                    BurstEngineResponses::Nothing
                }
                BurstEngineRequests::ReadSimulationTimestep(_m) => {
                    BurstEngineResponses::ReadSimulationTimestep(Ok(ReadSimulationTimestepFeagiRequestMessageResponse {
                        simulation_timestep: 0.05,
                    }))
                }
                BurstEngineRequests::ReadStats(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadStatus(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadBurstCounter(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadConfig(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::ReadFrequencyStatus(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateSimulationTimestep(m) => {
                    let timestep = m.simulation_timestep;
                    // TODO
                    
                    BurstEngineRequestCategory::not_implemented_response(&request)
                    
                }
                BurstEngineRequests::CreateControl(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateStart(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateStop(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateHold(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::CreateResume(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
                BurstEngineRequests::EditConfig(m) => {
                    BurstEngineRequestCategory::not_implemented_response(&request)
                }
            }
        }
    ).await.is_ok()
}
