use serde::{Deserialize, Serialize};

pub type SimulationTimestep = f64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BurstEnginesMessage {
    SimulationTimestepRequest(),
    SimulationTimeStepResponse(SimulationTimestep),
    SimulationTimestepChangeRequest(SimulationTimestep),
    SimulationTimestepChangeRequestResponse(bool),
}
