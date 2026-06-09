use feagi_structures::feagi_data::feagi_pdi::{PDICollection, PDIElement};
use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagGenericDevice;
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::wrapped_indexing::NPUCorticalAreaIndexGlobal;


/// Holds references to cortical area indexing elements, which themselves hold references to
/// the slices of data relevant to a cortical area
pub trait NPUGlobalCorticalAreaIndexingTable<FGQ: FeagiGlobalQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_total_number_cortical_areas(&self) -> NPUCorticalAreaIndexGlobal<FGQ>;
}

pub trait NPUGlobalCorticalAreaIndexingElement<FGQ: FeagiGlobalQuantization>:
PDIElement
+ PDITagGenericDevice
{}

///