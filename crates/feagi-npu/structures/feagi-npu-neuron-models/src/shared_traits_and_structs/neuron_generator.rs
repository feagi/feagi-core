use feagi_structures::quantization::{CorticalAreaNeuronQuantization, NPUGlobalQuantization};
use feagi_structures::quantization::quantizable_collections::spatial_collections::dim_3::quantizable_spatial_3d_collection_traits::{IterItemCoordinate3DRefMut, QuantizableSpatial3DCollectionUncheckedTrait, QuantizableSpatial3DQuantTypes};
use feagi_structures::quantization::quantizable_collections::spatial_collections::dim_4::quantizable_spatial_4d_collection_traits::{IterItemCoordinate4DRefMut, QuantizableSpatial4DCollectionUncheckedTrait, QuantizableSpatial4DQuantTypes};
use feagi_npu_neuron_models::::NeuronModelParametersTrait;

/// A Neuron Generator is a struct that handles generating neurons in the case of cortical area
/// creation or resets (such as from resizing)
pub trait NeuronGeneratorBase<NPUQ: NPUGlobalQuantization, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{

}


/// Neuron Generator for dimensional cortical areas. Functions iterate through xyz(t) neurons to
/// init them.
pub trait NeuronGeneratorNeuronDimensional<
    NPUQ: NPUGlobalQuantization,
    CANQ: CorticalAreaNeuronQuantization,
    NMP: NeuronModelParametersTrait<CANQ>,
>
{
    /// Called when the voxel density is 1 (1 neuron per voxel)
    fn init_neurons_single_voxel<
        Q3D: QuantizableSpatial3DQuantTypes,
        Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>
    >
    (
        &self,
        dimensions: &Q3D::Dim3DType,
        current_coordinate: Q3D::Coord3DType,
        enumerated_neuron_ref: &mut IterItemCoordinate3DRefMut<Q3D, Collection3DType>
    );

    /// Called when there is more than 1 neuron per voxel
    fn init_neurons_multi_voxel<
        Q4D: QuantizableSpatial4DQuantTypes,
        Collection3DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>
    >
    (
        &self,
        dimensions: &Q4D::Coord4DType,
        current_coordinate: Q4D::Coord4DType,
        enumerated_neuron_ref: &mut IterItemCoordinate4DRefMut<Q4D, Collection3DType>
    );
}

