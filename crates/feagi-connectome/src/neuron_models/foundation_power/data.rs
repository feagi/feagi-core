use crate::quantization_levels::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_spec::NoCustomNeuronInit;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::{NeuronLayoutOf, NeuronModel, NeuronModelTag};
use feagi_connectome_neural::neural_basis::quantization_levels::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use half::{bf16, f16};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Foundation Power type
    FoundationPower

    NeuronLayout: NeuronLayoutTypeVoxel

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

cortical_dynamics_quantization_foundation_power_template!(expand_cortical_dynamics_quantization);
