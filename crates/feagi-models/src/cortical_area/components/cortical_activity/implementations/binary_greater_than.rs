use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{PercentageUnsigned, QuantizedDecimalTrait};
use crate::cortical_area::components::cortical_activity::CorticalActivity;

/// Returns 100 Percent Cortical Activity if the number is below / above a threshold, and
/// 0 percent if it is above / below (inverse the setting) if not
pub struct BinaryThresholdCorticalActivity<FIQ: FeagiIndexQuantization, POut: QuantizedDecimalTrait>
{
    threshold: PercentageUnsigned<POut>,
    is_checking_greater_than: bool,
    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization, POut: QuantizedDecimalTrait> CorticalActivity<FIQ, POut> for BinaryThresholdCorticalActivity<FIQ, POut>
{
    type CorticalContext = ();

    fn calculate_cortical_activity(&self, context: &Self::CorticalContext, write_to: &mut PercentageUnsigned<POut>) {
        todo!()
    }
}