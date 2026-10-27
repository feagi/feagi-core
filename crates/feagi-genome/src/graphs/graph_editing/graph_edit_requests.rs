// TODO into NPU Edit Action
// TODO NPU Edit Action Into Graph Edit Outcome

use feagi_brain_basis::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::graphs::graph_editing::edits::cortical_area::AddCorticalAreaRequest;

pub enum GraphEditRequests<FIQ: FeagiIndexQuantization> {
    AddCorticalArea(AddCorticalAreaRequest<FIQ>)
}