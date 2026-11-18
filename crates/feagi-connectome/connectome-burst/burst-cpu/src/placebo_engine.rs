//! This burst engine does nothing. It simply follows edit commands, and returns plausible responses.
//! No neuron or synapse dynamics are run on here

use crate::NeuronProcessorSingleEngineVisualizationTransmitter;
use burst_basis::burst_engine::{BurstEngine, BurstEngineIndex};
use burst_basis::burst_engine_commands::EngineChangeCommandOtherError;
use burst_basis::connectome_reshaping::{ConnectomeEditResultantCorticalAreaAdd, ConnectomeEditResultantEnum, ConnectomeEditStepEnum};
use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use connectome_basis::visualization::NeuronVisualizationVoxelByteIndex;

pub struct BurstEngineCPUPlacebo<FIQ, Vis>
where
    FIQ: FeagiIndexQuantization,
    Vis: Send // TODO
{
    cortical_indexes: Vec<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    next_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    _t: <BurstEngineCPUPlacebo<FIQ, Vis> as BurstEngine<FIQ, Vis>>::VisualizationTransmitter,
    _p: core::marker::PhantomData<(Vis)>
}

impl<FIQ, VIS> BurstEngine<FIQ, VIS> for BurstEngineCPUPlacebo<FIQ, VIS>
where
    FIQ: FeagiIndexQuantization,
    VIS: Send // TODO
{
    type InitParameters = ();
    type InitError = PlaceboEngineInitError;
    type VisualizationTransmitter = NeuronProcessorSingleEngineVisualizationTransmitter<FIQ>;

    async fn init_burst_engine(parameters: Self::InitParameters, engine_index: BurstEngineIndex, visualization_transmitter: Self::VisualizationTransmitter) -> Result<Self, Self::InitError> {
        Ok(Self {
            cortical_indexes: vec![],
            next_cortical_index: CorticalAreaEngineIndex::QUANT_ZERO,
            _t: visualization_transmitter,
            _p: Default::default()
        })
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        // Do nothing lol
    }

    async fn process_connectome_edit(&mut self, step: ConnectomeEditStepEnum<FIQ>) -> Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError> {
        let res = match step {
            ConnectomeEditStepEnum::Nothing => {
                ConnectomeEditResultantEnum::Nothing
            }
            ConnectomeEditStepEnum::CorticalAreaAdd(a) => {
                
                self.next_cortical_index += CorticalAreaEngineIndex::QUANT_ONE;
                self.cortical_indexes.push(self.next_cortical_index);
                
                ConnectomeEditResultantEnum::CorticalAreaAdd(ConnectomeEditResultantCorticalAreaAdd {
                    new_cortical_index: self.next_cortical_index,
                    visualization_start_index: NeuronVisualizationVoxelByteIndex::QUANT_ZERO,
                })
            }
        };
        Ok(res)
    }
}



#[derive(Debug, thiserror::Error)]
pub enum PlaceboEngineInitError {
    #[error("Unknown")]
    Unknown
}

