use feagi_structures::feagi_data::{create_quantized_index_count_wrapper, create_quantized_spatial_index_coordinate_4d_wrapper, create_quantized_spatial_index_dimensions_4d_wrapper};
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;


/// Defines a neuron with an index unique within its cortical area
create_quantized_index_count_wrapper!(NPUWrappedNeuronCorticalLocalIndex);



/// Defines the location along 1 axis of a dimensional neuron in 3d, within the context of its cortical area
create_quantized_index_count_wrapper!(NPUWrappedDimensionalNeuronAxialPosition);

/// Defines the density index / count of a neuron in regards to its voxel, as some cortical areas can define
/// multiple neurons per voxel (though most default to 1 neuron per voxel)
create_quantized_index_count_wrapper!(NPUWrappedDimensionalNeuronDensity);


/// Defines the coordinate of a neuron within its cortical area, along the xyz axis and its d density index
create_quantized_spatial_index_coordinate_4d_wrapper!(NPUWrappedDimensionalNeuronCoordinate, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronDensity);

/// Defines the dimensions of a dimensional cortical area, including the density of each voxel
create_quantized_spatial_index_dimensions_4d_wrapper!(NPUWrappedCorticalAreaDimensions, NPUWrappedDimensionalNeuronCoordinate, NPUWrappedNeuronCorticalLocalIndex, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronDensity);