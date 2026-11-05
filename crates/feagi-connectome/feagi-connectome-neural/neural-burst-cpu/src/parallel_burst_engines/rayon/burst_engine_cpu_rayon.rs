use crate::parallel_burst_engines::rayon::errors::BurstEngineCPURayonInitError;
use crate::shared_structs::{EnginePerCorticalData, EnginePerNeuronData};
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelTransmitter;
use neural_basis::engine::engine_change_command::{EngineChangeCommand, EngineChangeCommandError, EngineChangeCommandReturn};
use neural_basis::engine::{BurstEngine, BurstEngineIndex};
use neural_basis::quantization_levels::FeagiIndexQuantization;
use neural_basis::visualization::BurstVisualizationData;

pub struct BurstEngineCPURayon<FIQ, VisTx>
where
    FIQ: FeagiIndexQuantization,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
{
    neuron_phase: NeuronPhase<FIQ>,
    synapse_phase: SynapsePhase,
    visualization_transmitter: VisTx,
}

impl<FIQ, VisTx> BurstEngine<FIQ, ()> for BurstEngineCPURayon<FIQ, VisTx>
where
    FIQ: FeagiIndexQuantization,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
{
    type InitParameters = ();
    type InitError = BurstEngineCPURayonInitError;
    type VisualizationTransmitter = VisTx;

    async fn init_burst_engine(
        parameters: Self::InitParameters,
        engine_index: BurstEngineIndex,
        visualization_transmitter: Self::VisualizationTransmitter,
    ) -> Result<Self, Self::InitError> {
        let engine = BurstEngineCPURayon {
            neuron_phase: NeuronPhase::new(),
            synapse_phase: SynapsePhase {},
            visualization_transmitter,
        };
        Ok(engine)
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        if send_visualization {
            // Full (the listener still holds both slots) or closed (nothing is listening) skips
            // this burst's visualization
            if let Ok(mut _visualization_slot) = self.visualization_transmitter.try_send_ref() {
                // TODO write this burst's visualization into the slot. It is published on drop
            }
        }
    }

    async fn process_change_command(
        &mut self,
        command: EngineChangeCommand<FIQ>,
    ) -> Result<EngineChangeCommandReturn<FIQ>, EngineChangeCommandError> {
        Err(EngineChangeCommandError::UnknownError)
    }
}

struct NeuronPhase<FIQ: FeagiIndexQuantization> {
    pub engine_per_cortical_data: EnginePerCorticalData<FIQ>,
    // quant
    pub engine_per_neuron: EnginePerNeuronData<FIQ>,
    //quant
    pub visualizations: BurstVisualizationData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NeuronPhase<FIQ> {
    pub fn new() -> Self {
        Self {
            engine_per_cortical_data: EnginePerCorticalData::<FIQ>::new(),
            engine_per_neuron: EnginePerNeuronData::<FIQ>::new(),
            visualizations: BurstVisualizationData::new(),
        }
    }
}

struct SynapsePhase {}
