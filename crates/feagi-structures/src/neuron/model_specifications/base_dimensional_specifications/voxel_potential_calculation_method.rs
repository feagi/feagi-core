use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType, QuantizableValueType};
use crate::neuron::model_specifications::base_specifications::NeuronMembranePotential;
use crate::neuron::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::VoxelPotential;
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// Describes what method a voxel's potential is calculated if it has multiple neurons
pub enum NeuronVoxelMultiPotentialCalculationMethod {
    Sum,
    Average,
    Max
}

impl NeuronVoxelMultiPotentialCalculationMethod {
    /// From a slice of multiple individual neurons in a voxel and their potential, to the
    /// potential of the voxel using the given enums method.
    /// Assumes Slice is not empty and that given slice length is correct!
    pub fn get_independent_neuron_potentials_as_voxel_potential<CANQ: CorticalAreaNeuronQuantization>(
        &self, neuron_slice: &[NeuronMembranePotential<CANQ::NeuronValueQuant>], slice_length_as_float: f32)
        -> VoxelPotential<CANQ::NeuronValueQuant> {

        // TODO debug check of slice length

        match self {
            NeuronVoxelMultiPotentialCalculationMethod::Sum => {
                neuron_slice
                    .iter()
                    .fold(VoxelPotential::ZERO, |acc, &neuron_pot| {
                        acc.saturating_add(VoxelPotential(neuron_pot.0))
                    })
            }
            NeuronVoxelMultiPotentialCalculationMethod::Average => {
                // TODO is this the best way to handle this for different quantizations?
                let sum = neuron_slice.iter().fold(
                    VoxelPotential::ZERO,
                    |acc, &neuron_pot| {
                        acc.saturating_add(VoxelPotential(neuron_pot.0))
                    },
                );
                sum / VoxelPotential::from_f32(slice_length_as_float)
            }
            NeuronVoxelMultiPotentialCalculationMethod::Max => {
                VoxelPotential(neuron_slice.iter().max().unwrap().0)
            }

        }
    }

    // TODO version of above but for entire structs
}