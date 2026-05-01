use feagi_structures::neurons::descriptors::NeuronCount;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::DimensionalNeuronModelDataSharedTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

/// Defines how neurons will be generated when creating a new dimensional cortical area
#[cfg(feature = "alloc")]
pub trait DimensionalCorticalAreaGeneratorTrait<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization> {

    /// The type of neuron model / cortical data that is generated
    type DimensionNeuronModelType: DimensionalNeuronModelDataSharedTrait<Q, DNQ>;

    /// Returns the number of neurons that the generated cortical area will have
    fn number_of_neurons(&self) -> NeuronCount<Q::NeuronIndexCountQuant>;

    /// Generates (Allocates) a new neuron/cortical data set representing a cortical area for RAM
    /// NPU deployments
    fn generate_new_cortical_area_data_ram(&self) -> Self::DimensionNeuronModelType;

    /// Given a dead cortical area of enough capacity, will overwrite its internals and reactivate
    /// it, allowing the creation of new cortical areas without additional allocations in RAM
    fn overwrite_dead_cortical_area_data_ram(&self, dead_area_overwriting: &mut Self::DimensionNeuronModelType) -> Result<(), FeagiNPUNeuronError>;

    // TODO feature gated cortical area generation for other hardware implementations!
}
