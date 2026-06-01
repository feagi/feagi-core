use feagi_structures::feagi_data::feagi_pdi::PDIElement;
use feagi_structures::feagi_data::feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalQuantization;

// NOTE: We only need one implementation per device type, so dont make traits for CPU/Device


/// Base trait for Cortical configuration, which is simply general details about a cortical area
/// not related to the neuron model, but needed broadly for compute. This is NOT extendable by
/// any neuron model!
pub trait CorticalConfiguration<FGQ>:
PDIElement
where
    FGQ: FeagiGlobalQuantization,
{
    // Only contain usable data if on the CPU

    // Number of neurons contained should be accessible
}


// Region CPU implementations

pub struct CorticalConfigurationDimensionalCPU<FGQ: FeagiGlobalQuantization>
{
    pub dimensions: SpatialIndexDimensions4D<FGQ::NeuronIndexCountQuant>,
    //pub check_for_neuron_activity_following_every_burst: bool, // TODO maybe we should always have this on....
    // TODO how do we mark if any neuron activity? atomics?
}

impl<FGQ: FeagiGlobalQuantization> PDITagGenericDevice for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> PDITagCPU for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> PDIElement for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> CorticalConfiguration<FGQ> for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> CorticalConfigurationDimensionalCPU<FGQ> {
    pub fn new(dimensions: SpatialIndexDimensions4D<FGQ::NeuronIndexCountQuant>) -> Self {
        CorticalConfigurationDimensionalCPU { dimensions }
    }
}

//endregion



