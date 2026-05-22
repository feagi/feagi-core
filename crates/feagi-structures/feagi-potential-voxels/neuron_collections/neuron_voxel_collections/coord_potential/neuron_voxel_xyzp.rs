use crate::neuron::neuron_collections::common_neuron_structs::{NeuronVoxelCoordinate, NeuronVoxelPotential};
use crate::neuron::neuron_collections::neuron_voxel_collections::traits::NeuronVoxel;
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// A single neuron voxel storing spatial coordinates and activation potential in XYZP format.
///
/// Represents a voxel containing neural activity at a specific 3D location within
/// a cortical area, along with its current activation/voltage level.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZP<CANQ: CorticalAreaNeuronQuantization>
{
    /// coordinate within the cortical area.
    pub coordinate: NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>,
    /// potential (voltage) of the voxel
    pub potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>,
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelXYZP<CANQ> {

    pub fn new(coordinate: NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>) -> Self {
        NeuronVoxelXYZP {
            coordinate,
            potential,
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxel<CANQ> for NeuronVoxelXYZP<CANQ> {

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
impl<CANQ: CorticalAreaNeuronQuantization> std::fmt::Display for NeuronVoxelXYZP<CANQ> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "NeuronVoxelXYZP({}, {}, {}, {})",
            self.coordinate.x,
            self.coordinate.y,
            self.coordinate.z,
            self.potential
        );
        write!(f, "{}", s)
    }
}
