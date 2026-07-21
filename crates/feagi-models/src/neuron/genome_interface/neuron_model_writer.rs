use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron::common_structs::cortical_area_layout::CorticalAreaLayout;
use crate::neuron::common_structs::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use crate::neuron::models_shared_traits::data::{NeuronModelCorticalData, NeuronModelNeuronData};



/// Describes a struct capable of writing / updating some or all of a cortical area properties. Used
/// for editing cortical areas and also initializing them
pub trait NeuronModelCorticalDataWriter {

    const QUANT_AND_MODEL: NestedNeuronModelTypeAndQuantization;

    type ModelQuantization: CorticalPotentialQuantization;
    type CorticalData: NeuronModelCorticalData<Self::ModelQuantization>;

    fn write_cortical_data<FIQ: FeagiIndexQuantization>(&self, overwriting: &mut Self::CorticalData);
}

/// Describes a struct capable of writing / updating some or all of the neurons of a cortical area.
/// Used for editing cortical areas and also initializing them
pub trait NeuronModelNeuronDataWriter {

    const QUANT_AND_MODEL: NestedNeuronModelTypeAndQuantization;

    type NeuronLayout: (); // TODO
    type ModelQuantization: CorticalPotentialQuantization;
    type NeuronData: NeuronModelNeuronData<Self::ModelQuantization>;

    fn write_neuron_data<FIQ: FeagiIndexQuantization>(&self, overwriting: &mut [Self::NeuronData], layout: &Self::NeuronLayout);
}
