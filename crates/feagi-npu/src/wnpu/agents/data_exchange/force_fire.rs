use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelLinearIndex;
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;
use feagi_genomic_context::cortical_area::CorticalID;


/// Identifies multiple cortical areas that are force firing something
pub type VoxelForceFire<NeuronIndex: QuantizedUnsignedIntegerTrait> = Vec<CorticalAreaVoxelForceFire<NeuronIndex>>;

#[derive(Clone)]
pub struct CorticalAreaVoxelForceFire<NeuronIndex: QuantizedUnsignedIntegerTrait> {
    pub cortical_id: CorticalID,
    pub force_firing: Vec<NeuronVoxelLinearIndex<NeuronIndex>>,
}