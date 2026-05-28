use feagi_data::{create_quantized_decimal_wrapper, create_quantized_index_count_wrapper, create_quantized_spatial_index_coordinate_3d_wrapper, create_quantized_spatial_index_dimensions_3d_wrapper};
use feagi_data::shared_quantization_sets::NeuronModelQuantizationBase;

/// We may use different Quantizations in various places
create_quantized_decimal_wrapper!(NeuronVoxelPotentialGeneric);
create_quantized_index_count_wrapper!(NeuronVoxelAxisGeneric);
create_quantized_index_count_wrapper!(NeuronVoxelLinearIndexGeneric);
create_quantized_spatial_index_coordinate_3d_wrapper!(NeuronVoxelCoordinateGeneric, NeuronVoxelAxisGeneric, NeuronVoxelAxisGeneric, NeuronVoxelAxisGeneric);
create_quantized_spatial_index_dimensions_3d_wrapper!(NeuronVoxelDimensionsGeneric, NeuronVoxelCoordinateGeneric, NeuronVoxelLinearIndexGeneric, NeuronVoxelAxisGeneric, NeuronVoxelAxisGeneric, NeuronVoxelAxisGeneric);


/// 32 bit is the "universal" quantization
pub mod universal {
    use feagi_data::shared_quantization_sets::CorticalAreaModelQuantization;
    use super::{NeuronVoxelAxisGeneric, NeuronVoxelCoordinateGeneric, NeuronVoxelPotentialGeneric, NeuronVoxelLinearIndexGeneric};

    pub struct CANQ32;
    impl CorticalAreaModelQuantization for CANQ32 {
        type GlobalBurstIndexQuant = u32; // we never use this here
        type NeuronIndexCountQuant = u32;
        type NeuronPotentialQuant = f32;
    }

    pub type NeuronVoxelPotentialF32 = NeuronVoxelPotentialGeneric<f32>;
    pub type NeuronVoxelAxis32 = NeuronVoxelAxisGeneric<u32>;
    pub type NeuronVoxelLinearIndex32 = NeuronVoxelLinearIndexGeneric<u32>;
    pub type NeuronVoxelCoordinate32 = NeuronVoxelCoordinateGeneric<u32>;
    pub type NeuronVoxelDimensionsGeneric32 = NeuronVoxelAxisGeneric<u32>;
}
