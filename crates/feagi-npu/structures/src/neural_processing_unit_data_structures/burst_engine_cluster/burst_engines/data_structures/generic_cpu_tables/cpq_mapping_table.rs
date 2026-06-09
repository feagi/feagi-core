use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagCPU;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, CorticalPotentialQuantizationFloat32, FeagiGlobalQuantization};

/// Add this to CPQ Typed collections to easily make tables with the CPQMappingTable
pub(crate) trait CPQMappingCollection<FGQ: FeagiGlobalQuantization>:
PDITagCPU
{}


pub(crate) struct CPQMappingTable<FGQ, CPQCollection>
where
    FGQ: FeagiGlobalQuantization,
    CPQCollection: CPQMappingCollection<FGQ>
{
    pub float32: CPQCollection<FGQ, CorticalPotentialQuantizationFloat32>
}