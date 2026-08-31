use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_npu_burst_engines::feagi_npu_burst_core::wrapped_values::EngineCorticalIndex;


pub struct CorticalArea<FIQ: FeagiIndexQuantization> {
    id: CorticalID,
    index: (u16, EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>),
    properties: (),
    incoming_mappings: Vec<CorticalID>,
    outgoing_mappings: Vec<CorticalID>,
    has_recursive_mappings: bool,
}