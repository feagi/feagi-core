use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::neuron_model::data::data_containers::{
    CorticalModelFullStorageData, CorticalModelGenomePropertiesData, CorticalModelNPUInterfaceData, CorticalModelSpawnData,
};
use feagi_connectome_neural::neural_basis::neuron_model::data::neuron_layout::implementations::voxels::NeuronLayoutVoxelDefaultType;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::CorticalUniversalDefaults;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::{
    CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalFunctionSpecification,
    NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronLayoutOf, NeuronModel,
};
use feagi_connectome_neural::neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use half::{bf16, f16};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Core Power type
    CorePower

    NeuronLayout: NeuronLayoutVoxelDefaultType

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

cortical_dynamics_quantization_core_power_template!(expand_cortical_dynamics_quantization);
