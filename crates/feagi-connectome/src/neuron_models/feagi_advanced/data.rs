use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedDecimalTrait;
use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_spec::{CorticalAreaSpec, NoCustomNeuronInit};
use feagi_connectome_neural::neural_basis::neuron_model::interfacing::NeuronModelUniversalCorticalProperties;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayout;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTypeVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::CorticalUniversalDefaults;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_model::{
    CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification,
    NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification, NeuronLayoutOf, NeuronModel,
};
use feagi_connectome_neural::neural_basis::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Feagi Advanced type
    FeagiAdvanced

    NeuronLayout: NeuronLayoutTypeVoxel

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

        CorticalDataVariance: {
            /// How much fire threshold increases in a cortical area on the X direction
            firing_threshold_increment_x: MembranePotentialQuant,
            /// How much fire threshold increases in a cortical area on the Y direction
            firing_threshold_increment_y: MembranePotentialQuant,
            /// How much fire threshold increases in a cortical area on the Z direction
            firing_threshold_increment_z: MembranePotentialQuant,
        }

        NeuronDataVariance: {
            /// Added to the firing threshold of the cortical area, for this neuron
            firing_threshold_offset: MembranePotentialQuant,
        }
    }
}

cortical_dynamics_quantization_feagi_advanced_template!(expand_cortical_dynamics_quantization);

impl<NM: NeuronModelFeagiAdvanced> CorticalDataVarianceSpecification<NM> for FeagiAdvancedCorticalDataVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> FeagiAdvancedNeuronDataVariance<NM> {
        let coordinates = layout.get_neuron_layout_context(index);
        let [x, y, z, _d] = coordinates.as_slice();
        FeagiAdvancedNeuronDataVariance::new(
            self.firing_threshold_increment_x * NM::MembranePotentialQuant::from_quantized_unsigned_integer(*x)
                + self.firing_threshold_increment_y * NM::MembranePotentialQuant::from_quantized_unsigned_integer(*y)
                + self.firing_threshold_increment_z * NM::MembranePotentialQuant::from_quantized_unsigned_integer(*z),
        )
    }
}

