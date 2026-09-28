use feagi_basis::prelude::*;
use crate::burst_phase_output::BurstPhaseOutput;
use crate::non_composable::non_composable_phase_notification::NonComposablePhaseNotification;

// quant level is irrelevant
const NOTIF_COUNT: usize = NonComposablePhaseNotification::<FeagiIndexQuantizationStandard>::NUMBER_NON_COMPOSABLE_PHASE_NOTIFICATIONS;

pub type NonComposableBurstPhaseOutput<FIQ: FeagiIndexQuantization> = BurstPhaseOutput<NonComposablePhaseNotification<FIQ>, NOTIF_COUNT>;