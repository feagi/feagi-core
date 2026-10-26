use std::collections::HashMap;
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::implementations::dynamic::FeagiRequestResponseChannelsAllocBackend;
use crate::workers::genome_manager::requests::categories::morphology::{MorphologyRequestCategory, MorphologyRequests, MorphologyResponses, ReadMorphologiesFeagiRequestMessageResponse};

pub(crate) async fn morphology_response(
    responder: &FeagiCategoryResponder<MorphologyRequestCategory, FeagiRequestResponseChannelsAllocBackend>) -> bool {

    responder.try_process_next(
        async | request: MorphologyRequests | {
           match &request {
                MorphologyRequests::Nothing => {
                    MorphologyResponses::Nothing
                }
                MorphologyRequests::ReadMorphologyList(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::ReadMorphologyTypes(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::ReadListTypes(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::ReadMorphologies(_) => {
                    MorphologyResponses::ReadMorphologies(Ok(ReadMorphologiesFeagiRequestMessageResponse {
                        morphologies: HashMap::new(),
                    }))
                }
                MorphologyRequests::ReadInfo(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::CreateMorphology(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::CreateMorphologyProperties(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::CreateMorphologyUsage(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::EditMorphology(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::EditRename(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::DeleteMorphology(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
                MorphologyRequests::DeleteDelete(_) => {
                    MorphologyRequestCategory::not_implemented_response(&request)
                }
            }
        }
    ).await.is_ok()
}
