use std::fmt::Formatter;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelTransmitter;
use crate::engine::engine_change_command::{EngineChangeCommand, EngineChangeCommandError, EngineChangeCommandReturn};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::visualization::BurstVisualizationData;

pub mod engine_change_command;



/// Some implementation meant to process neurons and synapses in real time. The point of FEAGI
pub trait BurstEngine<FIQ: FeagiIndexQuantization, Vis>: Sized
where
    FIQ: FeagiIndexQuantization,
{
    /// Whatever is needed for the burst engine to init
    type InitParameters: Send;
    /// The error returned if the burst engine has an error initializing
    type InitError: Send + core::fmt::Debug;
    /// Which channel (heap allocated, static) visualization is published through
    type VisualizationTransmitter: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>;

    /// Init the burst engine, creating it in the process. The engine keeps
    /// `visualization_transmitter` to publish visualization through
    async fn init_burst_engine(
        parameters: Self::InitParameters,
        engine_index: BurstEngineIndex,
        visualization_transmitter: Self::VisualizationTransmitter,
    ) -> Result<Self, Self::InitError>;

    /// Run a full burst, from neuron firing to neurons getting their inputs combined. If
    /// `send_visualization`, the burst's visualization is written into a claimed slot of the
    /// visualization transmitter. If no slot is free (or nothing is listening) it is skipped, a
    /// burst never waits on visualization
    async fn run_burst(&mut self, send_visualization: bool);

    async fn process_change_command(&mut self, command: EngineChangeCommand<FIQ>)
                                    -> Result<EngineChangeCommandReturn<FIQ>, EngineChangeCommandError>;
}

// TODO designer trait with commands to add genome objects, channels, errors

/// Defines the index of a given burst engine
#[derive(Clone, Copy, Hash, Debug)]
pub struct BurstEngineIndex(pub u8);

impl core::fmt::Display for BurstEngineIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}