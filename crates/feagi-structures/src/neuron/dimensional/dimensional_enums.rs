use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType, QuantizableValueType};
use crate::neuron::individual_neuron_structs::IndividualNeuronMembranePotential;
use crate::neuron::dimensional::dimensional_structs::NeuronVoxelPotential;
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// Common Neuron related structs used in various collection types and in other areas

/// Describes what method a collection is using to store potential data. Mainly matters when neuron
/// density != 1
pub enum DimensionalNeuronCollectionElementType {
    /// Potential is stored per individual neuron, particularly relevant for NPU
    IndividualNeuron,
    /// Potential is stored for a given voxel of neuron(s)
    Voxel
}

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
        &self, neuron_slice: &[IndividualNeuronMembranePotential<CANQ::NeuronValueQuant>], slice_length_as_float: f32)
        -> NeuronVoxelPotential<CANQ::NeuronValueQuant> {
        
        // TODO debug check of slice length
        
        match self {
            NeuronVoxelMultiPotentialCalculationMethod::Sum => {
                neuron_slice
                    .iter()
                    .fold(NeuronVoxelPotential::ZERO, |acc, &neuron_pot| {
                        acc.saturating_add(NeuronVoxelPotential(neuron_pot.0))
                    })
            }
            NeuronVoxelMultiPotentialCalculationMethod::Average => {
                // TODO is this the best way to handle this for different quantizations?
                let sum = neuron_slice.iter().fold(
                    NeuronVoxelPotential::ZERO,
                    |acc, &neuron_pot| {
                        acc.saturating_add(NeuronVoxelPotential(neuron_pot.0))
                    },
                );
                sum / NeuronVoxelPotential::from_f32(slice_length_as_float)
            }
            NeuronVoxelMultiPotentialCalculationMethod::Max => {
                NeuronVoxelPotential(neuron_slice.iter().max().unwrap().0)
            }

        }
    }
    
    // TODO version of above but for entire structs
}