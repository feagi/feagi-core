use crate::parallel_burst_engines::rayon::errors::BurstEngineCPURayonInitError;
use crate::parallel_burst_engines::rayon::neuron_model_store::CpuNeuronModelStores;
use crate::shared_structs::CpuEngineNeuronData;
use ::rayon::prelude::*;
use core::ops::DerefMut;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelTransmitter;
use feagi_basis::prelude::*;
use neural_basis::connectome_editing::{
    ConnectomeEditErrorCorticalAreaAdd, ConnectomeEditResultantCorticalAreaAdd, ConnectomeEditResultantEnum, ConnectomeEditStepEnum,
    ConnectomeVisitorDispatchWrapper,
};
use neural_basis::engine::engine_change_command::EngineChangeCommandOtherError;
use neural_basis::engine::{BurstEngine, BurstEngineIndex};
use neural_basis::quantization_levels::FeagiIndexQuantization;
use neural_basis::visualization::BurstVisualizationData;

/// Bursts on the CPU, the neurons of a cortical area in parallel through rayon. `NeuronModels`
/// lists the store of every neuron model and quantization the engine can host
pub struct BurstEngineCPURayon<FIQ, NeuronModels, VisTx>
where
    FIQ: FeagiIndexQuantization,
    NeuronModels: CpuNeuronModelStores<FIQ>,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
{
    /// The burst about to run
    burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    neuron_data: CpuEngineNeuronData<FIQ>,
    neuron_models: NeuronModels,
    #[allow(dead_code)] // TODO synapses
    synapse_phase: SynapsePhase,
    visualization_transmitter: VisTx,
}

impl<FIQ, NeuronModels, VisTx> BurstEngineCPURayon<FIQ, NeuronModels, VisTx>
where
    FIQ: FeagiIndexQuantization,
    NeuronModels: CpuNeuronModelStores<FIQ>,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
{
    /// The burst about to run
    pub fn burst_index(&self) -> BurstIndex<FIQ::BurstIndexQuant> {
        self.burst_index
    }

    /// Packs the firing flag of every neuron into the visualization of the engine, one bit per
    /// neuron. A separate pass from the neuron phase, as that is parallel over neurons and
    /// several neurons share a word. This is parallel over words instead, each written whole, so
    /// the bits of the previous burst are cleared by the same write
    fn pack_visualization(neuron_data: &mut CpuEngineNeuronData<FIQ>) {
        let neuron_flags = neuron_data.per_neuron.neuron_flags.store().as_slice();
        let words = neuron_data.visualization.bit_data.store_mut().as_mut_slice();
        let neurons_per_word = CpuEngineNeuronData::<FIQ>::NEURONS_PER_VISUALIZATION_WORD;

        for area in 0..neuron_data.per_cortical.neuron_count.store().len() {
            let neuron_count = neuron_data.per_cortical.neuron_count.store()[area].quant_to_usize();
            let first_neuron = neuron_data.per_cortical.first_neuron_engine_index.store()[area].quant_to_usize();
            let first_word = neuron_data.per_cortical.visualization_start_index.store()[area].quant_to_usize();
            let word_count = neuron_count.div_ceil(neurons_per_word);

            // The last word of an area is only partly filled if its neuron count does not divide
            // evenly. Its other bits stay zero
            words[first_word..first_word + word_count]
                .par_iter_mut()
                .zip(neuron_flags[first_neuron..first_neuron + neuron_count].par_chunks(neurons_per_word))
                .for_each(|(word, flags_of_word)| {
                    let mut packed = 0;
                    for (bit, flags) in flags_of_word.iter().enumerate() {
                        if flags.is_firing() {
                            packed |= 1 << bit;
                        }
                    }
                    *word = packed;
                });
        }
    }
}

impl<FIQ, NeuronModels, VisTx, CVDW> BurstEngine<FIQ, CVDW, ()> for BurstEngineCPURayon<FIQ, NeuronModels, VisTx>
where
    FIQ: FeagiIndexQuantization,
    NeuronModels: CpuNeuronModelStores<FIQ>,
    VisTx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>,
    CVDW: ConnectomeVisitorDispatchWrapper<FIQ, CorticalAreaNeurogenesisDispatch = NeuronModels::CorticalAreaNeurogenesisDispatch>,
{
    type InitParameters = ();
    type InitError = BurstEngineCPURayonInitError;
    type VisualizationTransmitter = VisTx;

    async fn init_burst_engine(
        _parameters: Self::InitParameters,
        _engine_index: BurstEngineIndex,
        visualization_transmitter: Self::VisualizationTransmitter,
    ) -> Result<Self, Self::InitError> {
        Ok(BurstEngineCPURayon {
            burst_index: BurstIndex::QUANT_ZERO,
            neuron_data: CpuEngineNeuronData::new(),
            neuron_models: NeuronModels::new(),
            synapse_phase: SynapsePhase {},
            visualization_transmitter,
        })
    }

    async fn run_burst(&mut self, send_visualization: bool) {
        self.neuron_models.run_neuron_phase(&self.burst_index, &mut self.neuron_data);

        if send_visualization {
            // Full (the listener still holds both slots) or closed (nothing is listening) skips
            // this burst's visualization
            if let Ok(mut visualization_slot) = self.visualization_transmitter.try_send_ref() {
                Self::pack_visualization(&mut self.neuron_data);
                // The slot keeps its allocation between bursts, and is only allocated again when
                // the structure of the visualization changed. It is published on drop
                let source = &self.neuron_data.visualization;
                let slot: &mut BurstVisualizationData<FIQ> = visualization_slot.deref_mut();
                slot.version = source.version;
                slot.change_counter = source.change_counter;
                if slot.bit_data.store().len() == source.bit_data.store().len() {
                    slot.bit_data.store_mut().copy_from_slice(source.bit_data.store());
                } else {
                    slot.bit_data = source.bit_data.clone();
                }
            }
        }

        // TODO synapse phase: every synapse whose source neuron is firing writes into its slot of
        // the incoming staging of its destination

        // Rolls over past the last index of its quantization
        self.burst_index = match self.burst_index.quant_to_usize() == FIQ::BurstIndexQuant::QUANT_MAX_USIZE {
            true => BurstIndex::QUANT_ZERO,
            false => BurstIndex::quant_from_usize_unchecked(self.burst_index.quant_to_usize() + 1),
        };
    }

    async fn process_connectome_edit(
        &mut self,
        step: ConnectomeEditStepEnum<FIQ, CVDW>,
    ) -> Result<ConnectomeEditResultantEnum<FIQ>, EngineChangeCommandOtherError> {
        match step {
            ConnectomeEditStepEnum::Nothing => Ok(ConnectomeEditResultantEnum::Nothing),
            ConnectomeEditStepEnum::CorticalAreaAdd(step) => {
                match self.neuron_models.add_cortical_area(step.cortical_area_neurogenesis_dispatch, &mut self.neuron_data) {
                    Ok(added) => Ok(ConnectomeEditResultantCorticalAreaAdd {
                        new_cortical_index: added.cortical_index,
                        visualization_start_index: added.visualization_start_index,
                    }
                    .into()),
                    // Every limit is checked before anything is allocated, so nothing was changed
                    Err(_) => Ok(ConnectomeEditErrorCorticalAreaAdd::ExceedsIndexQuantization.into()),
                }
            }
        }
    }
}

struct SynapsePhase {}
