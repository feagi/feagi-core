use crate::parallel_burst_engines::neural_cpu_init_dispatch::CpuCorticalAreaInitDispatch;
use crate::parallel_burst_engines::rayon::errors::BurstEngineCPURayonInitError;
use crate::shared_structs::{EnginePerCorticalData, EnginePerNeuronData};
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelTransmitter;
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_basis::target_frequency::TargetFrequency;
use neural_basis::connectome_editing::{
    ConnectomeEditResultantCorticalAreaAdd, ConnectomeEditResultantEnum, ConnectomeEditStepEnum, ConnectomeVisitorDispatchWrapper,
};
use neural_basis::engine::engine_change_command::EngineChangeCommandOtherError;
use neural_basis::engine::{BurstEngine, BurstEngineIndex};
use neural_basis::neuron_model::wrapped_indexing::CorticalAreaEngineIndex;
use neural_basis::quantization_levels::FeagiIndexQuantization;
use neural_basis::visualization::{BurstVisualizationData, NeuronVisualizationWordIndex};

/// TODO placeholder. How far apart the visualization start indexes of added cortical areas are
const PLACEHOLDER_VISUALIZATION_WORDS_PER_AREA: usize = 64;

pub struct BurstEngineCPURayon<FIQ, VisTx>
where
    FIQ: FeagiIndexQuantization,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
{
    neuron_phase: NeuronPhase<FIQ>,
    synapse_phase: SynapsePhase,
    visualization_transmitter: VisTx,
    /// How long each burst is delayed for
    burst_rate: TargetFrequency,
    /// Stored only, nothing uses it yet
    visualization_rate: TargetFrequency,
    /// TODO placeholder. The cortical index the next added cortical area gets
    next_cortical_index: usize,
    /// TODO placeholder. The visualization start index the next added cortical area gets
    next_visualization_start_index: usize,
}

impl<FIQ, VisTx, CVDW> BurstEngine<FIQ, CVDW, ()> for BurstEngineCPURayon<FIQ, VisTx>
where
    FIQ: FeagiIndexQuantization,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
    CVDW: ConnectomeVisitorDispatchWrapper<FIQ>,
    CVDW::CorticalAreaInitDispatch: CpuCorticalAreaInitDispatch<FIQ>,
{
    type InitParameters = ();
    type InitError = BurstEngineCPURayonInitError;
    type VisualizationTransmitter = VisTx;

    async fn init_burst_engine(
        parameters: Self::InitParameters,
        engine_index: BurstEngineIndex,
        visualization_transmitter: Self::VisualizationTransmitter,
        burst_rate: TargetFrequency,
        visualization_rate: TargetFrequency,
    ) -> Result<Self, Self::InitError> {
        let engine = BurstEngineCPURayon {
            neuron_phase: NeuronPhase::new(),
            synapse_phase: SynapsePhase {},
            visualization_transmitter,
            burst_rate,
            visualization_rate,
            next_cortical_index: 0,
            next_visualization_start_index: 0,
        };
        Ok(engine)
    }

    /// TODO placeholder. Only waits out one tick of the burst rate
    async fn run_burst(&mut self, send_visualization: bool) {
        if send_visualization {
            // Full (the listener still holds both slots) or closed (nothing is listening) skips
            // this burst's visualization
            if let Ok(mut _visualization_slot) = self.visualization_transmitter.try_send_ref() {
                // TODO write this burst's visualization into the slot. It is published on drop
            }
        }
        std::thread::sleep(self.burst_rate.duration_between_ticks());
    }

    async fn set_burst_rate(&mut self, burst_rate: TargetFrequency) -> Result<(), EngineChangeCommandOtherError> {
        self.burst_rate = burst_rate;
        Ok(())
    }

    async fn set_visualization_rate(&mut self, visualization_rate: TargetFrequency) -> Result<(), EngineChangeCommandOtherError> {
        self.visualization_rate = visualization_rate;
        Ok(())
    }

    /// TODO placeholder. Allocates nothing, only hands out the next indexes
    async fn process_connectome_edit(
        &mut self,
        step: ConnectomeEditStepEnum<FIQ, CVDW>,
    ) -> Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError> {
        match step {
            ConnectomeEditStepEnum::Nothing => Ok(ConnectomeEditResultantEnum::Nothing),
            ConnectomeEditStepEnum::CorticalAreaAdd(_step) => {
                // Out of indexes at this quantization
                let new_cortical_index = CorticalAreaEngineIndex::quant_try_from_usize(self.next_cortical_index)
                    .map_err(|_| EngineChangeCommandOtherError::UnknownError)?;
                let visualization_start_index = NeuronVisualizationWordIndex::quant_try_from_usize(self.next_visualization_start_index)
                    .map_err(|_| EngineChangeCommandOtherError::UnknownError)?;
                self.next_cortical_index += 1;
                self.next_visualization_start_index += PLACEHOLDER_VISUALIZATION_WORDS_PER_AREA;
                Ok(ConnectomeEditResultantCorticalAreaAdd { new_cortical_index, visualization_start_index }.into())
            }
        }
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
