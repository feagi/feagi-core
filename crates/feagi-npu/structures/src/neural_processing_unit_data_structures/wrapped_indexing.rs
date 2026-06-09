//! Wrapped Types allow us to differentiate between quantizable indexes and minimize the risk of
//! confusion without performance penalty. Note that this is effectively a CPU only feature, but
//! still serves useful reference when designing other hardware backends adn for metadata

use feagi_structures::feagi_data::{create_quantized_decimal_wrapper, create_quantized_index_count_wrapper, create_quantized_spatial_index_coordinate_4d_wrapper, create_quantized_spatial_index_dimensions_4d_wrapper};
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;


// TODO Conversions into voxel coordinates for non dense neurons! (uses result and feagi error)

//region Neurons

//region Linear Indexing and Spatial

/// Defines a neuron with an index unique in the entire NPU
create_quantized_index_count_wrapper!(NPUNeuronIndexGlobal);

/// Defines a neuron with an index unique its neuron model type and membrane potential quantization
create_quantized_index_count_wrapper!(NPUNeuronIndexModelQuantizationLocal);

/// Defines a neuron with an index unique its membrane potential quantization
create_quantized_index_count_wrapper!(NPUNeuronIndexQuantizationLocal);

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

//endregion

//region Chunked Indexing

/// Defines a chunk of neurons for the global neuron index table. Chunk size may vary depending on
/// Neuron Model Quantization type
create_quantized_index_count_wrapper!(NPUNeuronChunkIndexGlobal);

/// Defines a chunk of neurons local to the vector of the neuron model and
/// quantization type. Chunk size may vary depending on Neuron Model Quantization type
create_quantized_index_count_wrapper!(NPUNeuronChunkIndexModelQuantizationLocal);

/// Defines a chunk of neurons local to the vector of the neurons membrane potentials quantization
/// type. Chunk size may vary depending on Neuron Model Quantization type
create_quantized_index_count_wrapper!(NPUNeuronChunkIndexQuantizationLocal);

/// Defines a chunk of neurons local to the cortical area of the neuron chunk.
/// Chunk size may vary depending on Neuron Model Quantization type
create_quantized_index_count_wrapper!(NPUNeuronChunkIndexCorticalAreaLocal);

//endregion


/// Defines the neuron membrane potential of a neuron. Uses neuron indexing.
create_quantized_decimal_wrapper!(NPUNeuronMembranePotential);

/// Defines the incoming potential of a neuron (the FCL / Fire Candidate List). Uses Neuron Indexing
create_quantized_decimal_wrapper!(NPUneuronFCLInputPotential);

/// In cases of a neuron recieving a massive number of inputs, we have a secondary write array for synapse outputs that we reduce into the main FCL first. This is the index on that type.
create_quantized_index_count_wrapper!(NPUneuronFCLInputSecondaryIndex);

/// In cases of a neuron recieving a massive number of inputs, we have a secondary write array for synapse outputs that we reduce into the main FCL first
create_quantized_decimal_wrapper!(NPUneuronFCLInputSecondaryPotential);

//endregion


/// Way of defining different neuron models and quantizations using a single u8 identifier
#[repr(u8)]
#[derive(Default)]
pub enum NeuronModelTypeAndQuantizationFlat {
    #[default]
    FeagiStandardModel_StandardQuantization = 0, // This should be zero since this is the most common usecase and we check specifically for this
}



/// Defines a cortical area with an index unique in the entire NPU
create_quantized_index_count_wrapper!(NPUCorticalAreaIndexGlobal);






//endregion



