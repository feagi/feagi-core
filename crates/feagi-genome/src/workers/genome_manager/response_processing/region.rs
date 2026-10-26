use std::collections::HashMap;
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use crate::workers::genome_manager::requests::categories::region::{ReadRegionsMembersFeagiRequestMessageResponse, RegionMembersEntry, RegionRequestCategory, RegionRequests, RegionResponses};

pub(crate) async fn region_response(
    responder: &FeagiCategoryResponder<RegionRequestCategory, FeagiRequestResponseChannelsAllocBackend>) -> bool {

    responder.try_process_next(
        async | request: RegionRequests | {
           match &request {
                RegionRequests::Nothing => {
                    RegionResponses::Nothing
                }
                RegionRequests::ReadRegionsMembers(_) => {
                    // Only the root region, with no cortical areas. No parent marks it as the root
                    let root = RegionMembersEntry {
                        title: "root".to_string(),
                        description: String::new(),
                        parent_region_id: None,
                        coordinate_2d: [0, 0],
                        coordinate_3d: [0, 0, 0],
                        areas: Vec::new(),
                        regions: Vec::new(),
                        inputs: Vec::new(),
                        outputs: Vec::new(),
                        designated_inputs: Vec::new(),
                        designated_outputs: Vec::new(),
                    };
                    RegionResponses::ReadRegionsMembers(Ok(ReadRegionsMembersFeagiRequestMessageResponse {
                        regions: HashMap::from([("root".to_string(), root)]),
                    }))
                }
                RegionRequests::ReadRegions(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::ReadRegionTitles(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::ReadRegion(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::CreateRegion(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::CreateClone(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::EditRegion(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::EditRelocateMembers(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::EditChangeRegionParent(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::EditChangeCorticalAreaRegion(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::DeleteRegion(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
                RegionRequests::DeleteRegionAndMembers(_) => {
                    RegionRequestCategory::not_implemented_response(&request)
                }
            }
        }
    ).await.is_ok()
}
