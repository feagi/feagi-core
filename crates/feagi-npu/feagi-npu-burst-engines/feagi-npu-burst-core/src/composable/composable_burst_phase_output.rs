use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use crate::BurstPhaseOutput;
use crate::composable::composable_phase_notification::ComposablePhaseNotification;

// quant level is irrelevant
const NOTIF_COUNT: usize = ComposablePhaseNotification::<FeagiIndexQuantizationStandard>::NUMBER_COMPOSABLE_PHASE_NOTIFICATIONS;

pub type ComposableBurstPhaseOutput<FIQ: FeagiIndexQuantization> = BurstPhaseOutput<ComposablePhaseNotification<FIQ>, NOTIF_COUNT>;