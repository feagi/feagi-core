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

pub trait CorticalConfigurationDimensional<FGQ>:
CorticalConfiguration<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{
    // Dimensions of cortical area (4D) should be accessible
}



// Region CPU implementations

pub struct CorticalConfigurationDimensionalCPU<FGQ: FeagiGlobalQuantization>
{
    pub dimensions: SpatialIndexDimensions4D<FGQ::NeuronIndexCountQuant>,
    post_synaptic_potential_base: f32, // TODO address me!
    is_postsynaptic_potential_drive_by_membrane_potential: bool,
    post_synaptic_potential_should_be_uniform: bool,



    // NOTE: Using plain bools for now, no need to pack here
}

impl<FGQ: FeagiGlobalQuantization> PDITagGenericDevice for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> PDITagCPU for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> PDIElement for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> CorticalConfiguration<FGQ> for CorticalConfigurationDimensionalCPU<FGQ> {}

impl<FGQ: FeagiGlobalQuantization> CorticalConfigurationDimensional<FGQ> for CorticalConfigurationDimensionalCPU<FGQ> {
}

impl<FGQ: FeagiGlobalQuantization> CorticalConfigurationDimensionalCPU<FGQ> {
    pub fn new(dimensions: SpatialIndexDimensions4D<FGQ::NeuronIndexCountQuant>) -> Self {
        CorticalConfigurationDimensionalCPU {
            dimensions,
            post_synaptic_potential_base: 0.0,
            is_postsynaptic_potential_drive_by_membrane_potential: false,
            post_synaptic_potential_should_be_uniform: false,
        }
    }
}

//endregion



