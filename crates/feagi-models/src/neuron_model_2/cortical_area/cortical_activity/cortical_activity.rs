use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::feagi_data_quantization::values::quantizable::{PercentageUnsigned, QuantizedDecimalTrait};


pub trait CorticalActivity<FIQ: FeagiIndexQuantization, POut: QuantizedDecimalTrait> {
    /// What information do you need (a tuple if multiple) from a cortical
    /// area to make this calculation
    type CorticalContext;

    /// Using the cortical context, calculate as a percentage how active the cortical area is for this
    /// usecase
    fn calculate_cortical_activity(&self, context: &Self::CorticalContext, write_to: &mut PercentageUnsigned<POut>);
}