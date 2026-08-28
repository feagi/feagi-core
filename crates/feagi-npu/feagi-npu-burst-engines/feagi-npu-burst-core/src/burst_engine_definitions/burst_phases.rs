
// TODO data exchanges?

#[derive(Debug, Clone, Copy, Default)]
pub enum RunBurstPhase{
    /// Runs starting from Neuron Burst Phase
    #[default]
    Full,
    // TODO multiple?
    SpecificBurstPhase(SpecificBurstPhase)
}

#[derive(Debug, Clone, Copy)]
pub enum SpecificBurstPhase{
    NeuronPhase(NeuronBurstPhase),
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