//! This burst engine does nothing. It simply follows edit commands, and returns plausible responses.
//! No neuron or synapse dynamics are run on here

use burst_basis::BurstEngine;
use burst_basis::connectome_editing::{ConnectomeEditEnumHandler, ConnectomeEditError, ConnectomeEditHandler, ConnectomeEditNothing};
use burst_basis::multi_engine::BurstEngineIndex;
use burst_basis::visualization::BurstEngineVisualizationSupport;
use connectome_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use connectome_basis::quantization_levels::FeagiIndexQuantization;


pub struct BurstEngineCPUPlacebo<FIQ, VIS>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport
{
    cortical_indexes: Vec<CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    next_cortical_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    _t: VIS::VisualizationTransmitter,
    _p: core::marker::PhantomData<VIS>
}

impl<FIQ, VIS> BurstEngine<FIQ, VIS> for BurstEngineCPUPlacebo<FIQ, VIS>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport
{
    type InitParameters = ();
    type InitError = PlaceboEngineInitError;

    async fn init_burst_engine(parameters: Self::InitParameters, engine_index: BurstEngineIndex, visualization_transmitter: VIS::VisualizationTransmitter) -> Result<Self, Self::InitError> {
        todo!()
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        todo!()
    }

    
    /*
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

     */
}



impl<FIQ, VIS> ConnectomeEditEnumHandler<FIQ> for BurstEngineCPUPlacebo<FIQ, VIS>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport
{

}


impl<FIQ, VIS> ConnectomeEditHandler<FIQ, ConnectomeEditNothing<FIQ>> for BurstEngineCPUPlacebo<FIQ, VIS>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport
{
    async fn handle_edit(&mut self, edit: ConnectomeEditNothing<FIQ>) -> Result<<burst_basis::connectome_editing::ConnectomeEditNothing<FIQ> as burst_basis::connectome_editing::ConnectomeEditStep<FIQ>>::Resultant, ConnectomeEditError<FIQ>> {
        todo!()
    }
}

#[cfg(feature = "allow-reshape-edits")]
impl<FIQ, VIS> ConnectomeEditHandler<FIQ, burst_basis::connectome_editing::ConnectomeEditReshapeStep<FIQ>> for BurstEngineCPUPlacebo<FIQ, VIS>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport
{
    async fn handle_edit(&mut self, edit: burst_basis::connectome_editing::ConnectomeEditReshapeStep<FIQ>) -> Result<burst_basis::connectome_editing::ConnectomeEditReshapeStep<FIQ>::Resultant, ConnectomeEditError<FIQ>> {
        todo!()
    }
}







#[derive(Debug, thiserror::Error)]
pub enum PlaceboEngineInitError {
    #[error("Unknown")]
    Unknown
}

