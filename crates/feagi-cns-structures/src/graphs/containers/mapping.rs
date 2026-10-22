use feagi_brain_models::::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::voxels::CorticalAreaVoxelCoordinates;




pub struct CorticalMappingEntry<FIQ: FeagiIndexQuantization> {
    pub connectivity_rule: ConnectivityRule<FIQ>
}



// TODO placeholder, pull from model crate instead!
pub enum ConnectivityRule<FIQ: FeagiIndexQuantization> {
    AllToAll,
    AllToOne(CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>),
    OneToAll(CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>),
    OneToOne(CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>, CorticalAreaVoxelCoordinates<FIQ::NeuronIndexQuant>),
}