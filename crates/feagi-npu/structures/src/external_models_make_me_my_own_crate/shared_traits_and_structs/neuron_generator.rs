use feagi_structures::quantization::{CorticalAreaNeuronQuantization, NPUGlobalQuantization};
use feagi_structures::quantization::quantizable_collections::spatial_collections::dim_3::quantizable_spatial_3d_collection_traits::{IterItemCoordinate3DRefMut, QuantizableSpatial3DCollectionUncheckedTrait, QuantizableSpatial3DQuantTypes};
use crate::external_models_make_me_my_own_crate::shared_traits_and_structs::NeuronModelParametersTrait;

/// A Neuron Generator is a struct that handles generating neurons in the case of cortical area
/// creation or resets (such as from resizing)
pub trait NeuronGeneratorBase<NPUQ: NPUGlobalQuantization, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{

}


/// Neuron Generator for dimensional cortical areas. Functions iterate through xyz(tO neurons to init them
pub trait NeuronGeneratorNeuronDimensional<
    NPUQ: NPUGlobalQuantization,
    CANQ: CorticalAreaNeuronQuantization,
    NMP: NeuronModelParametersTrait<CANQ>,
>
{
    fn init_neurons_single_voxel<
        Q3D: QuantizableSpatial3DQuantTypes,
        Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>
    >
    (&self,
     dimensions: &Q3D::Coord3DType,
     enumerated_neuron_ref: &mut IterItemCoordinate3DRefMut<Q3D, Collection3DType>);



}