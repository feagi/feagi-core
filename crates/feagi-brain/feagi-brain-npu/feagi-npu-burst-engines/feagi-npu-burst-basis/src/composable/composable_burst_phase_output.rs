use feagi_basis::prelude::*;
use crate::burst_phase_output::BurstPhaseOutput;
use crate::composable::composable_phase_notification::ComposablePhaseNotification;

// quant level is irrelevant
const NOTIF_COUNT: usize = ComposablePhaseNotification::<FeagiIndexQuantizationStandard>::NUMBER_COMPOSABLE_PHASE_NOTIFICATIONS;

pub type ComposableBurstPhaseOutput<FIQ: FeagiIndexQuantization> = BurstPhaseOutput<ComposablePhaseNotification<FIQ>, NOTIF_COUNT>;