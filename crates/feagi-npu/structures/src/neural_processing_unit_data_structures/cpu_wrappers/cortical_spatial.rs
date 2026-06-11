use feagi_structures::feagi_data::{create_quantized_index_count_wrapper, create_quantized_spatial_index_coordinate_4d_wrapper, create_quantized_spatial_index_dimensions_4d_wrapper};
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;



/// Defines a neuron with an index unique within its cortical area
create_quantized_index_count_wrapper!(NPUNeuronIndexCorticalLocal);

/// Defines the location along 1 axis of a dimensional neuron in 3d, within the context of its cortical area
create_quantized_index_count_wrapper!(NPUDimensionalNeuronAxialPosition);

/// Defines the density index / count of a neuron in regards to its voxel, as some cortical areas can define
/// multiple neurons per voxel (though most default to 1 neuron per voxel)
create_quantized_index_count_wrapper!(NPUDimensionalNeuronDensity);

impl<NeuronIndexQuant: QuantizedIndexCountTrait> NPUDimensionalNeuronDensity<NeuronIndexQuant> {
    pub fn is_single_neuron_per_voxel(&self) -> bool {self.0 == QuantizedIndexCountTrait::QUANT_ONE}
}

/// Defines the coordinate of a neuron within its cortical area, along the xyz axis and its d density index
create_quantized_spatial_index_coordinate_4d_wrapper!(NPUDimensionalNeuronCoordinate, NPUDimensionalNeuronAxialPosition, NPUDimensionalNeuronAxialPosition, NPUDimensionalNeuronAxialPosition, NPUDimensionalNeuronDensity);

/// Defines the dimensions of a dimensional cortical area, including the density of each voxel
create_quantized_spatial_index_dimensions_4d_wrapper!(NPUCorticalAreaDimensions, NPUDimensionalNeuronCoordinate, NPUNeuronIndexCorticalLocal, NPUDimensionalNeuronAxialPosition, NPUDimensionalNeuronAxialPosition, NPUDimensionalNeuronAxialPosition, NPUDimensionalNeuronDensity);

impl<NeuronIndexQuant: QuantizedIndexCountTrait> NPUCorticalAreaDimensions<NeuronIndexQuant> {
    pub fn contains_only_single_neuron_per_voxel(&self) -> bool {self.get_w().is_single_neuron_per_voxel()}
}