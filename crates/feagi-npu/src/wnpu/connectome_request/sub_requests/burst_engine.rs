use serde::{Deserialize, Serialize};
use crate::npu_3::npu_target_frequency::NPUTargetFrequency;
use crate::wnpu::connectome_request::connectome_request::{ConnectomeRequest, ConnectomeRequestEnum};

#[doc(hidden)]
/// Allows selecting between burst engine options
pub struct BurstEngineRequestBuilder;

impl BurstEngineRequestBuilder
{
    /// Wipes all connectome data and pauses on the initial burst
    pub fn reset_and_pause() -> ConnectomeRequest {
        BurstEngineRequestEnum::ResetAndPause.into()
    }

    /// Wipes all connectome data and starts at a given frequency
    pub fn reset_and_resume(target_frequency: NPUTargetFrequency) -> ConnectomeRequest {
        BurstEngineRequestEnum::ResetAndResume(target_frequency).into()
    }

    /// pauses the burst engine and freezes in place
    pub fn pause() -> ConnectomeRequest {
        BurstEngineRequestEnum::Pause.into()
    }

    /// Sets the frequency without trying to interrupt the data flow in general, resuming if needed
    pub fn run_at_frequency(target_frequency: NPUTargetFrequency) -> ConnectomeRequest {
        BurstEngineRequestEnum::RunAtFrequency(target_frequency).into()
    }

    // TODO Shift Burst
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum BurstEngineRequestEnum {

    ResetAndPause,
    ResetAndResume(NPUTargetFrequency),
    Pause,
    RunAtFrequency(NPUTargetFrequency),
    // TODO
    // /// While paused, allow moving through burst frames by a specific number of bursts
    // ShiftBurst(i32) // yes, there is a reason this is signed!
}

impl Into<ConnectomeRequest> for BurstEngineRequestEnum
{
    fn into(self) -> ConnectomeRequest {
        let a = ConnectomeRequestEnum::BurstEngine(self);
        a.into()
    }
}