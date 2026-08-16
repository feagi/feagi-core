use feagi_data::neuron_voxels::voxel_vector::VoxelVector;
use feagi_data::values::quantizable::{QuantizedDecimalTrait, QuantizedUnsignedIntegerTrait};
use feagi_genomic_context::cortical_area::CorticalID;

// TODO this isnt efficient

/// Multiple cortical areas with dense mp data
pub type CorticalAreaVoxelPotentials<NeuronIndex: QuantizedUnsignedIntegerTrait, MPQuant: QuantizedDecimalTrait> = Vec<SingleCorticalAreaVoxelPotentials<NeuronIndex, MPQuant>>;

pub struct SingleCorticalAreaVoxelPotentials<NeuronIndex: QuantizedUnsignedIntegerTrait, MPQuant: QuantizedDecimalTrait> {
    pub cortical_id: CorticalID,
    pub voxel_potentials: VoxelVector<NeuronIndex, MPQuant>
}