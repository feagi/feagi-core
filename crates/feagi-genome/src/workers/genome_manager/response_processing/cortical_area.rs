use std::collections::HashMap;
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationStandard;
use crate::graphs::implementations::designer_graph::DesignerGraph;
use crate::workers::genome_manager::requests::categories::cortical_area::{CorticalAreaRequestCategory, CorticalAreaRequests, CorticalAreaResponses, ReadCorticalAreaGeometryFeagiRequestMessageResponse, ReadCorticalMapDetailedFeagiRequestMessageResponse, ReadIpuTypesFeagiRequestMessageResponse, ReadOpuTypesFeagiRequestMessageResponse};

pub(crate) async fn cortical_area_response(
    responder: &FeagiCategoryResponder<CorticalAreaRequestCategory, FeagiRequestResponseChannelsAllocBackend>,
    graph: &mut DesignerGraph<FeagiIndexQuantizationStandard>) -> bool {
    
    responder.try_process_next(
        async | request: CorticalAreaRequests | {
           match &request {
                CorticalAreaRequests::Nothing => {
                    CorticalAreaResponses::Nothing
                }
                CorticalAreaRequests::ReadIpu(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadOpu(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadIpuTypes(r) => {
                    CorticalAreaResponses::ReadIpuTypes(Ok(ReadIpuTypesFeagiRequestMessageResponse {
                        types: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadOpuTypes(r) => {
                    CorticalAreaResponses::ReadOpuTypes(Ok(ReadOpuTypesFeagiRequestMessageResponse {
                        types: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadCorticalAreaIdList(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalAreaNameList(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalIdNameMapping(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalTypes(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalMapDetailed(r) => {
                    CorticalAreaResponses::ReadCorticalMapDetailed(Ok(ReadCorticalMapDetailedFeagiRequestMessageResponse {
                        mappings: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadCorticalLocations2d(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalAreaGeometry(r) => {
                    CorticalAreaResponses::ReadCorticalAreaGeometry(Ok(ReadCorticalAreaGeometryFeagiRequestMessageResponse {
                        areas: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadCorticalVisibility(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadVoxelNeurons(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadMemory(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCorticalNameLocation(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCorticalAreaProperties(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateMultiCorticalAreaProperties(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCorticalArea(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCustomCorticalArea(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateClone(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateVoxelNeurons(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateResize(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCorticalArea(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditMultiCorticalArea(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditReset(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCoord2d(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditSuppressCorticalVisibility(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCoordinates3d(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCoord3d(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::DeleteCorticalArea(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::DeleteMultiCorticalArea(r) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
            }
        }
    ).await.is_ok()
}



