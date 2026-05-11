use crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType;
use crate::neuron_collections::neuron_voxel_collections::traits::NeuronVoxel;
use crate::neuron_collections::neuron_voxel_collections::voxel_structs::{NeuronVoxelIndexCount, NeuronVoxelPotential};
use crate::quantization_level::CorticalAreaNeuronQuantization;

#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelIP<CANQ: CorticalAreaNeuronQuantization>
{
    pub index: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    /// potential (voltage) of the voxel
    pub potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>,
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelIP<CANQ>
{
    pub fn new(
        index: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
        potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>,
    ) -> Self {
        Self {
            index,
            potential,
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxel<CANQ> for NeuronVoxelIP<CANQ>
{
    fn get_voxel_potential(&self) -> NeuronVoxelPotential<CANQ::NeuronValueQuant> {
        self.potential
    }

    fn get_voxel_potential_ref(&self) -> &NeuronVoxelPotential<CANQ::NeuronValueQuant> {
        &self.potential
    }

    fn set_voxel_potential_ref_mut(&mut self) -> &mut NeuronVoxelPotential<CANQ::NeuronValueQuant> {
        &mut self.potential
    }

    fn set_voxel_potential(&mut self, potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>) {
        self.potential = potential;
    }
}

#[cfg(feature = "alloc")]
impl<CANQ: CorticalAreaNeuronQuantization> std::fmt::Display for NeuronVoxelIP<CANQ>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "NeuronVoxelIP({}, {})",
            self.index,
            self.potential
        );
        write!(f, "{}", s)
    }
}