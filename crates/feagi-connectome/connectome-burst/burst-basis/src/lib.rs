use crate::visualization::BurstEngineVisualizationSupport;
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::connectome_editing::ConnectomeEditEnumHandler;
use crate::multi_engine::BurstEngineIndex;

pub mod connectome_reshaping;
pub mod connectome_editing;
pub mod visualization;
pub mod multi_engine;

/// Some implementation meant to process neurons and synapses in real time. The point of FEAGI
pub trait BurstEngine<FIQ, VIS>: ConnectomeEditEnumHandler<FIQ>
where
    FIQ: FeagiIndexQuantization,
    VIS: BurstEngineVisualizationSupport
// Vis TODO
{
    /// Whatever is needed for the burst engine to init
    type InitParameters: Send + 'static;
    /// The error returned if the burst engine has an error initializing
    type InitError: Send + 'static + core::error::Error;
    

    /// Init the burst engine, creating it in the process. The engine keeps
    /// `visualization_transmitter` to publish visualization through (if relevant).
    /// Timing is not the engine's
    /// concern: whatever runs it decides when each burst runs and when visualization is sent
    async fn init_burst_engine(
        parameters: Self::InitParameters,
        engine_index: BurstEngineIndex,
        visualization_transmitter: VIS::VisualizationTransmitter,
    ) -> Result<Self, Self::InitError>;

    /// Run a full burst, from neuron firing to neurons getting their inputs combined. If
    /// `send_visualization`, the burst's visualization is written into a claimed slot of the
    /// visualization transmitter. If no slot is free (or nothing is listening) it is skipped, a
    /// burst never waits on visualization. . Note if visualization is disabled via trait, the 
    /// send_visualization is ignored entirely
    async fn run_burst(&mut self, send_visualization: bool);


    //NOTE: extended trait brings in `handle_burst_engine_command_enum`
}
