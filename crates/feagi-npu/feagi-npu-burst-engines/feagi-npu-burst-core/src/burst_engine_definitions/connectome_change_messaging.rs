use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_values::EngineCorticalIndex;


pub enum EngineConnectomeChangeRequest<FIQ: FeagiIndexQuantization> {
    AddCorticalArea(()), // writer enum
    RemoveCorticalArea(EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>),
}


pub enum EngineConnectomeChangeResponse<FIQ: FeagiIndexQuantization> {
    CorticalAreaAdded{
        index: EngineCorticalIndex <FIQ::CorticalAreaIndexCountQuant>,
    },
    CorticalAreaRemoved {},
    
}


