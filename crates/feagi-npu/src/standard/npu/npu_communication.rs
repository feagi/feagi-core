use crate::NPUTargetFrequency;

pub enum NPUCommand {
    UpdateFrequency(NPUTargetFrequency),
    Pause,
    // TODO request edits to specific workers (by index)
}