use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_init::NeuronInitializer;
use crate::neuron_models::initializers::UniformNeuronInit;
#[cfg(feature = "alloc")]
use crate::neuron_models::initializers::StoredNeuronInit;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::{NeuronModel, NeuronModelTag};
use feagi_connectome_neural::neural_basis::quantization_levels::{MembranePotentialQuantization, FeagiIndexQuantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf};
use half::{bf16, f16};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Foundation Death type
    FoundationDeath

    NeuronLayout: NeuronLayoutTagVoxel

    NeuronInitializers: {
        /// Every neuron starts with the same data
        Uniform: UniformNeuronInit<NM>,
        /// Every neuron has its own data, such as when loaded from a connectome
        #[cfg(feature = "alloc")]
        Stored: StoredNeuronInit<NM>,
    }

    // Stored in a genome as every neuron starting with the same data
    GenomeNeuronInit: UniformNeuronInit<NM>

    QuantizationLevels: {
        Float16: {
            MembranePotentialQuant: f16,
        }

        BFloat16: {
            MembranePotentialQuant: bf16,
        }

        Float32: {
            MembranePotentialQuant: f32,
        }

        Float64: {
            MembranePotentialQuant: f64,
        }
    }
}

cortical_dynamics_quantization_foundation_death_template!(expand_cortical_dynamics_quantization);
