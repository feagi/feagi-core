use serde::{Deserialize, Serialize};
use feagi_genomic_context::cortical_area::CorticalID;
use crate::wnpu::connectome_request::connectome_request::{ConnectomeRequest, ConnectomeRequestEnum};


#[doc(hidden)]
/// Add, Remove, or Edit a Cortical Area
pub struct CorticalAreaRequestBuilder;

impl CorticalAreaRequestBuilder {

    /// Add a cortical area
    pub fn add_area() -> CorticalAreaAdderRequestBuilder {
        CorticalAreaAdderRequestBuilder
    }
    
}



#[doc(hidden)]
/// Add a cortical area
pub struct CorticalAreaAdderRequestBuilder;

impl CorticalAreaAdderRequestBuilder {
    
}








#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum CorticalAreaRequestEnum {
    AddCorticalArea(CorticalID, )
}

impl Into<ConnectomeRequest> for CorticalAreaRequestEnum
{
    fn into(self) -> ConnectomeRequest {
        let a = ConnectomeRequestEnum::CorticalArea(self);
        a.into()
    }
}