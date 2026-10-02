use serde::{Deserialize, Serialize};

pub type SimulationTimestep = f64;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BurstEnginesMessage {
    SimulationTimestepRequest(),
    SimulationTimeStepResponse(SimulationTimestep),
    SimulationTimestepChangeRequest(SimulationTimestep),
    SimulationTimestepChangeRequestResponse(bool),
}
