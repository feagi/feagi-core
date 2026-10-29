use crate::cortical_area::CorticalDynamicsModel;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::cortical_area::cortical_dynamics_quantization::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDynamicsQuantization, CorticalFunctionSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification};
use feagi_connectome_neural::neural_basis::cortical_area::data::neuron_layout::implementations::voxels_default::NeuronLayoutVoxelDefault;
use feagi_connectome_neural::neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Feagi Advanced type
    FeagiAdvanced

    NeuronLayout: NeuronLayoutVoxelDefault<FIQ>

    Quantizables: {
        /// For handling timing
        Period: QDECIMAL,
    }

    QuantizationLevels: {
        DefaultQuant: {
            MembranePotentialQuant: f32,
            Period: f32,
        }
    }

    DataSpecifications: {
        CorticalDataProperties: {
            /// How long till a neuron of the cortical area can fire again
            refractory_period: Period,
            /// At what threshold do we fire
            firing_threshold: MembranePotentialQuant,
            /// Amount of leak
            leak_coefficient: MembranePotentialQuant,
            /// How variable a leak is
            leak_variability: MembranePotentialQuant,
            /// Current post-synaptic
            postsynaptic_current: MembranePotentialQuant,
            /// max postsynaptic current
            postsynaptic_current_max: MembranePotentialQuant,
            /// neuron degeneration TODO shouldnt this be in memory?
            degeneration: MembranePotentialQuant,
        }

        CorticalFunctionContext: {
            /// How much fire threshold increases in a cortical area on the X direction
            firing_threshold_increment_x: MembranePotentialQuant,
            /// How much fire threshold increases in a cortical area on the Y direction
            firing_threshold_increment_y: MembranePotentialQuant,
            /// How much fire threshold increases in a cortical area on the Z direction
            firing_threshold_increment_z: MembranePotentialQuant,
        }
    }
}

cortical_dynamics_quantization_feagi_advanced_template!(expand_cortical_dynamics_quantization);
