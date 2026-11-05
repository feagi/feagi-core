use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedDecimalTrait;
use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_spec::NoCustomNeuronInit;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayout;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::{NeuronLayoutOf, NeuronModel, NeuronModelTag};
use feagi_connectome_neural::neural_basis::quantization_levels::MembranePotentialQuantization;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Feagi Simple type
    FeagiSimple

    NeuronLayout: NeuronLayoutTagVoxel

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
            /// Current post-synaptic
            postsynaptic_current: MembranePotentialQuant,
            /// max postsynaptic current
            postsynaptic_current_max: MembranePotentialQuant,
        }

        CorticalDataVariance: {
            
        }

        NeuronDataInternal: {
            /// How many more bursts the neuron is refractory for
            refractory_countdown: Period,
        }

        NeuronDataVariance: {
        }
    }
}

cortical_dynamics_quantization_feagi_simple_template!(expand_cortical_dynamics_quantization);

impl<NM: NeuronModelFeagiSimple> CorticalDataVarianceSpecification<NM> for FeagiSimpleCorticalDataVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> FeagiSimpleNeuronDataVariance<NM> {
        let coordinates = layout.get_neuron_layout_context(index);
        let [x, y, z, _d] = coordinates.as_slice();
        FeagiSimpleNeuronDataVariance::new()
    }
}

