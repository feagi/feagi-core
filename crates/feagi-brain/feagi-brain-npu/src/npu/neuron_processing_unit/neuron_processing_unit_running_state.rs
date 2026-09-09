use crate::npu::npu_target_frequency::NPUTargetFrequency;

/// Defines what state the NPU is in
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NeuronProcessingUnitRunningState {
    #[default]
    Paused,
    RunningAtFrequency(NPUTargetFrequency),
    RunningAtMaxSpeed,
    Crashed
}

