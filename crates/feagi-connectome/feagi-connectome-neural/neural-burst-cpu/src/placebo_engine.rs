//! This burst engine does nothing. It simply follows edit commands, and returns plausible responses.
//! No neuron or synapse dynamics are run on here

use neural_basis::connectome_editing::{ConnectomeEditResultantEnum, ConnectomeEditStepEnum, ConnectomeNeuronModelTypes};
use neural_basis::engine::{BurstEngine, BurstEngineIndex};
use neural_basis::engine::engine_change_command::EngineChangeCommandOtherError;
use neural_basis::quantization_levels::FeagiIndexQuantization;

pub struct BurstEngineCPUPlacebo<FIQ, CNMT, VIS>
where
    FIQ: FeagiIndexQuantization,
    CNMT: ConnectomeNeuronModelTypes<FIQ>,
    VIS: Send // TODO
{

}

impl<FIQ, CNMT, VIS> BurstEngine<FIQ, CNMT, VIS> for BurstEngineCPUPlacebo<FIQ, CNMT, VIS>
where
    FIQ: FeagiIndexQuantization,
    CNMT: ConnectomeNeuronModelTypes<FIQ>,
    VIS: Send // TODO
{
    type InitParameters = ();
    type InitError = PlaceboEngineInitError;
    type VisualizationTransmitter = NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>;

    async fn init_burst_engine(parameters: Self::InitParameters, engine_index: BurstEngineIndex, visualization_transmitter: Self::VisualizationTransmitter) -> Result<Self, Self::InitError> {
        todo!()
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        todo!()
    }

    async fn process_connectome_edit(&mut self, step: ConnectomeEditStepEnum<FIQ, CNMT>) -> Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError> {
        todo!()
    }
}




#[derive(Debug, thiserror::Error)]
pub enum PlaceboEngineInitError {
    #[error("Unknown")]
    Unknown
}

