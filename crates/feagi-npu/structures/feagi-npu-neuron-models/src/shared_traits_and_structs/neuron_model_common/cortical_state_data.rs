use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::CorticalAreasIndexQuantization;


//region Base "Tag" Traits

/// Base trait for Cortical State Data, which is simply general details about a cortical area
/// not related to the neuron model, such as size / dimensions
pub trait CorticalDataStateDataCommonDevice<CAIQ>
where
    CAIQ: CorticalAreasIndexQuantization,
{
    // Nothing
}

pub trait CorticalDataStateDataLinearDevice<CAIQ>:
CorticalDataStateDataCommonDevice<CAIQ>
where
    CAIQ: CorticalAreasIndexQuantization,
{
    // Method to get number of neurons
}

pub trait CorticalDataStateDataDimensionalDevice<CAIQ>:
CorticalDataStateDataCommonDevice<CAIQ>
where
    CAIQ: CorticalAreasIndexQuantization,
{
    // Method to get 4d dimensions (xyz depth)

    // Inherits method to get number of neurons
}
//endregion

//region CPU

pub trait CorticalDataStateDataCommonCPU<CAIQ>:
CorticalDataStateDataCommonDevice<CAIQ>
where
    CAIQ: CorticalAreasIndexQuantization,
{

}

pub trait CorticalDataStateDataLinearCPU<CAIQ>:
CorticalDataStateDataCommonCPU<CAIQ>
+ CorticalDataStateDataCommonDevice<CAIQ>
where
    CAIQ: CorticalAreasIndexQuantization,
{
    fn get_total_number_of_neurons(&self) -> CAIQ::NeuronIndexCountQuant;
}

pub trait CorticalDataStateDataDimensionalCPU<CAIQ>:
CorticalDataStateDataCommonCPU<CAIQ>
+ CorticalDataStateDataCommonDevice<CAIQ>
where
    CAIQ: CorticalAreasIndexQuantization,
{
    // We do 4d for density as the 4th field. Even on nondense implementations, we often need to pad to 16 bytes anyway
    fn get_dimensions_4d(&self) -> &SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>;
}

//region Implementations

// TODO linear

pub struct CorticalStateDimensionalCPUData<CAIQ: CorticalAreasIndexQuantization>
{
    dimensions: SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>,
}

impl<CAIQ: CorticalAreasIndexQuantization> CorticalDataStateDataCommonDevice<CAIQ> for CorticalStateDimensionalCPUData<CAIQ> {}

impl<CAIQ: CorticalAreasIndexQuantization> CorticalDataStateDataCommonCPU<CAIQ> for CorticalStateDimensionalCPUData<CAIQ> {}

impl<CAIQ: CorticalAreasIndexQuantization> CorticalDataStateDataDimensionalCPU<CAIQ> for CorticalStateDimensionalCPUData<CAIQ>
{
    fn get_dimensions_4d(&self) -> &SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant> {
        &self.dimensions
    }
}

impl<CAIQ: CorticalAreasIndexQuantization> CorticalStateDimensionalCPUData<CAIQ> {
    pub(crate) fn new(dimensions: SpatialIndexDimensions4D<CAIQ::NeuronIndexCountQuant>) -> Self {
        CorticalStateDimensionalCPUData { dimensions }
    }
}

//endregion


//endregion


