use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use crate::BurstPhaseOutput;
use crate::non_composable::non_composable_phase_notification::NonComposablePhaseNotification;

// quant level is irrelevant
const NOTIF_COUNT: usize = NonComposablePhaseNotification::<FeagiIndexQuantizationStandard>::NUMBER_NON_COMPOSABLE_PHASE_NOTIFICATIONS;

pub type NonComposableBurstPhaseOutput<FIQ: FeagiIndexQuantization> = BurstPhaseOutput<NonComposablePhaseNotification<FIQ>, NOTIF_COUNT>;