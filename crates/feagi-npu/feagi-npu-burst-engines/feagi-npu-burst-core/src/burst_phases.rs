//! Descriptors for what processing step a burst engine is doing / should do


#[derive(Debug, Clone, Copy, Default)]
pub enum RunBurstPhase{
    /// Runs starting from Neuron Burst Phase, the entire burst
    #[default]
    Full,
    // TODO multiple?
    SpecificBurstPhase(SpecificBurstPhase)
}

#[derive(Debug, Clone, Copy)]
pub enum SpecificBurstPhase{
    /// Individual neuron / cortical processing step
    NeuronPhase(NeuronBurstPhase),
    /// Synaptic propagation processing step
    SynapsePhase(SynapseBurstPhase),
}

#[derive(Debug, Clone, Copy)]
pub enum NeuronBurstPhase {
    CorticalAreaDynamics,
    CorticalAreaContextDynamics,
    VisualMotorSensorCrossEngineCorticalDataExchange
}


#[derive(Debug, Clone, Copy)]
pub enum SynapseBurstPhase {
    CorticalMappingDynamics,
    SynapseDelayMigration,
    SynapseConsolidation,
    CrossEngineCorticalDataExchange
}