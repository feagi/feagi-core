use crate::npu_3::npu_target_frequency::NPUTargetFrequency;

pub enum NPUCommand {
    UpdateFrequency(NPUTargetFrequency),
    Pause,
    // TODO request edits to specific workers (by index)
}