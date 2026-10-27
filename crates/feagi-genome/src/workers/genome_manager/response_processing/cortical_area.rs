use std::collections::HashMap;
use feagi_basis::feagi_message::request::feagi_request_message_category::{FeagiCategoryResponder, FeagiRequestCategory};
use feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend;
use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationStandard;
use crate::graphs::feagi_graph::FeagiGraph;
use crate::graphs::implementations::designer_graph::DesignerGraph;
use crate::workers::genome_manager::requests::categories::prelude::*;

pub(crate) async fn cortical_area_response(
    responder: &FeagiCategoryResponder<CorticalAreaRequestCategory, FeagiRequestResponseChannelsAllocBackend>,
    graph: &mut DesignerGraph<FeagiIndexQuantizationStandard>) -> bool {
    
    responder.try_process_next(
        async | request: CorticalAreaRequests | {
           match &request {
                CorticalAreaRequests::Nothing => {
                    CorticalAreaResponses::Nothing
                }
                CorticalAreaRequests::ReadIpu(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadOpu(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadIpuTypes(m) => {
                    CorticalAreaResponses::ReadIpuTypes(Ok(ReadIpuTypesFeagiRequestMessageResponse {
                        types: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadOpuTypes(m) => {
                    CorticalAreaResponses::ReadOpuTypes(Ok(ReadOpuTypesFeagiRequestMessageResponse {
                        types: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadCorticalAreaIdList(_m) => {
            
                    let cortical_ids = graph.get_all_cortical_areas();

                   // CorticalAreaResponses::ReadCorticalAreaIdList(Ok(ReadCorticalAreaIdListFeagiRequestMessageResponse {
                    //    cortical_ids,
                    //}))
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalAreaNameList(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalIdNameMapping(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalTypes(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalMapDetailed(m) => {
                    CorticalAreaResponses::ReadCorticalMapDetailed(Ok(ReadCorticalMapDetailedFeagiRequestMessageResponse {
                        mappings: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadCorticalLocations2d(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadCorticalAreaGeometry(m) => {
                    CorticalAreaResponses::ReadCorticalAreaGeometry(Ok(ReadCorticalAreaGeometryFeagiRequestMessageResponse {
                        areas: HashMap::new(),
                    }))
                }
                CorticalAreaRequests::ReadCorticalVisibility(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadVoxelNeurons(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::ReadMemory(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCorticalNameLocation(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCorticalAreaProperties(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateMultiCorticalAreaProperties(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCorticalArea(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateCustomCorticalArea(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateClone(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateVoxelNeurons(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::CreateResize(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCorticalArea(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditMultiCorticalArea(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditReset(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCoord2d(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditSuppressCorticalVisibility(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCoordinates3d(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::EditCoord3d(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::DeleteCorticalArea(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
                CorticalAreaRequests::DeleteMultiCorticalArea(m) => {
                    CorticalAreaRequestCategory::not_implemented_response(&request)
                }
            }
        }
    ).await.is_ok()
}



