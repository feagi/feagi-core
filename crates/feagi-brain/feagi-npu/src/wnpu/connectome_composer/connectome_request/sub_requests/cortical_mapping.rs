use serde::{Deserialize, Serialize};
use feagi_genomic_context::cortical_area::CorticalID;
use crate::wnpu::connectome_composer::connectome_request::connectome_request::{ConnectomeRequest, ConnectomeRequestEnum};

#[doc(hidden)]
/// Add or Remove a Cortical Mapping
pub struct CorticalMappingRequestBuilder;

impl CorticalMappingRequestBuilder {
    /*
    /// Add a cortical mapping between 2 cortical areas
    pub fn create_mapping() -> CorticalMappingAdderRequestBuilder {
        CorticalMappingAdderRequestBuilder
    }
    
     */
    
}

/*
#[doc(hidden)]
/// Add a cortical mapping of a type
pub struct CorticalMappingAdderRequestBuilder;

impl CorticalMappingAdderRequestBuilder {
    
    /// Map a dimensional cortical area to a dimensional cortical area
    pub fn dimensional_to_dimensional(source: CorticalID, destination: CorticalID, connectivity_rule_and_model: CorticalMappingEntryWriterByModelQuant) -> ConnectomeRequest // TODO needs connectivity
    {
        CorticalMappingRequestEnum::CreateDimensionalMapping(source, destination, connectivity_rule_and_model).into()
    }
    
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum CorticalMappingRequestEnum {
    CreateDimensionalMapping(CorticalID, CorticalID, CorticalMappingEntryWriterByModelQuant), // TODO Connectivity rule, model
}

impl Into<ConnectomeRequest> for CorticalMappingRequestEnum
{
    fn into(self) -> ConnectomeRequest {
        let a = ConnectomeRequestEnum::CorticalMapping(self);
        a.into()
    }
}

 */