use serde::{Deserialize, Serialize};
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_models::neuron_model::genome_compose::cortical_writer_by_model_quant::CorticalWriterByModelQuant;
use crate::standard::wnpu::connectome_request::connectome_request::{ConnectomeRequest, ConnectomeRequestEnum};


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

    // TODO Core, Sensory, and other typed ones with defaults

    /// With custom context, make a request to make a cortical area
    pub fn custom(temp_id: CorticalID, writer: CorticalWriterByModelQuant) -> ConnectomeRequest // TODO way writer is organized makes it unclear
    {
        CorticalAreaRequestEnum::AddCorticalArea(temp_id, writer).into()
    }
}




#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum CorticalAreaRequestEnum {
    AddCorticalArea(CorticalID, CorticalWriterByModelQuant) // TODO Dimensions, writer
}

impl Into<ConnectomeRequest> for CorticalAreaRequestEnum
{
    fn into(self) -> ConnectomeRequest {
        let a = ConnectomeRequestEnum::CorticalArea(self);
        a.into()
    }
}