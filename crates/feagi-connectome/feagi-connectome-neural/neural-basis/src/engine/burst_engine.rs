use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Some implementation meant to process neurons and synapses in real time. The point of FEAGI
pub trait BurstEngine<FIQ: FeagiIndexQuantization, Vis,>: Sized
where
    FIQ: FeagiIndexQuantization,
    // TODO Vis should handle visibility
{
    /// Whatever is needed for the burst engine to init
    type InitParameters: Send;
    /// The error returned if the burst engine has an error initializing
    type InitError; //: Send + core::fmt::Debug; // TODO

    /// Init the burst engine, creating it in the process
    async fn init_burst_engine(parameters: Self::InitParameters) -> Result<Self, Self::InitError>;

    /// Run a full burst, from neuron firing to neurons getting their inputs combined
    async fn run_burst(&mut self);
}



// TODO designer trait with commands to add genome objects, channels, errors