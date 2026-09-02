use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::enclosed_burst_engine::noncomposable_burst_engine::NonComposableBurstEngine;
#[cfg(feature = "composable")]
use crate::enclosed_burst_engine::composable_burst_engine::ComposableBurstEngine;


// NOTE: for a constant API surface, keep this enum. In the case of a single variant, release mode
// will likely optimize this away anyways

/// Can contain a burst engine of composable or noncomposable type
pub enum EnclosedBurstEngine<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "composable")]
    Composable(ComposableBurstEngine<FIQ>),
    NonComposable(NonComposableBurstEngine<FIQ>)
}


