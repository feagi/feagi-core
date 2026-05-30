use feagi_structures::feagi_data::feagi_ecs::element::FeagiECSElement;
use feagi_structures::feagi_data::feagi_ecs::tag_device::FeagiECSTagGenericDevice;
use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::FeagiGlobalIndexQuantization;


/// Base trait for Cortical configuration, which is simply general details about a cortical area
/// not related to the neuron model, such as size / dimensions
pub trait CorticalConfiguration<CAIQ>:
FeagiECSElement
where
    CAIQ: FeagiGlobalIndexQuantization,
{
    // Only contain usable data if on the CPU
}

//region Implementations


pub struct CorticalConfigurationCPUDimensional<CAIQ: FeagiGlobalIndexQuantization>
{
    pub dimensions: SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>,
    //pub check_for_neuron_activity_following_every_burst: bool, // TODO maybe we should always have this on....
    // TODO how do we mark if any neuron activity? atomics?
}

impl<CAIQ: FeagiGlobalIndexQuantization> FeagiECSTagGenericDevice for CorticalConfigurationCPUDimensional<CAIQ> {}

impl<CAIQ: FeagiGlobalIndexQuantization> FeagiECSElement for CorticalConfigurationCPUDimensional<CAIQ> {}

impl<CAIQ: FeagiGlobalIndexQuantization> CorticalConfiguration<CAIQ> for CorticalConfigurationCPUDimensional<CAIQ> {}

impl<CAIQ: FeagiGlobalIndexQuantization> CorticalConfigurationCPUDimensional<CAIQ> {
    pub(crate) fn new(dimensions: SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>) -> Self {
        CorticalConfigurationCPUDimensional { dimensions }
    }
}

//endregion



