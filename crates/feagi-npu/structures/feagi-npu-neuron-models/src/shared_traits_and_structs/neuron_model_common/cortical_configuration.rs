use feagi_structures::feagi_data::feagi_ecs::element::{FeagiECSElementOnCPU, FeagiECSElementOnDevice};
use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::CorticalAreasIndexQuantization;


/// Base trait for Cortical configuration, which is simply general details about a cortical area
/// not related to the neuron model, such as size / dimensions
pub trait CorticalConfiguration<CAIQ>:
FeagiECSElementOnDevice
where
    CAIQ: CorticalAreasIndexQuantization,
{
    // Only contain usable data if on the CPU
}

//region Implementations


pub struct CorticalConfigurationCPUDimensional<CAIQ: CorticalAreasIndexQuantization>
{
    dimensions: SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>,
}

impl<CAIQ: CorticalAreasIndexQuantization> FeagiECSElementOnDevice for CorticalConfigurationCPUDimensional<CAIQ> {}

impl<CAIQ: CorticalAreasIndexQuantization> FeagiECSElementOnCPU for CorticalConfigurationCPUDimensional<CAIQ> {}

impl<CAIQ: CorticalAreasIndexQuantization> CorticalConfiguration<CAIQ> for CorticalConfigurationCPUDimensional<CAIQ> {}

impl<CAIQ: CorticalAreasIndexQuantization> CorticalConfigurationCPUDimensional<CAIQ> {
    pub(crate) fn new(dimensions: SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>) -> Self {
        CorticalConfigurationCPUDimensional { dimensions }
    }
}

//endregion



