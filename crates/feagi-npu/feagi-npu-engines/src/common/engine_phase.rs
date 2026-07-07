
/// TODO macro to verify phase

#[derive(Copy, Clone)]
pub enum EnginePhase {
    BetweenBurst(BetweenBurstPhase),
    MembranePotentialsImport(MembranePotentialsImportPhase),
    SynapseDynamics(SynapseDynamicsPhase),
    FCLExport(FCLExportPhase),
    FCLImport(FCLImportPhase),
    NeuronDynamics(NeuronDynamicsPhase),
}


/// About to start a burst as neurons have just updated their potentials. 
/// Can export motor, visualization and membrane potential data, and import sensor data
#[derive(Copy, Clone)]
pub struct BetweenBurstPhase;

/// Expecting to import Membrane Potential data
#[derive(Copy, Clone)]
pub struct MembranePotentialsImportPhase;

/// Execute Synapse dynamics from neuron values, potentially consolidating neurons first, and
/// merge synaptic output to FCL
#[derive(Copy, Clone)]
pub struct SynapseDynamicsPhase;

/// Expecting to export FCL (synapse) potentials
#[derive(Copy, Clone)]
pub struct FCLExportPhase;

/// Expecting to import FCL (synapse) potentials
#[derive(Copy, Clone)]
pub struct FCLImportPhase;

/// Runs through neuron dynamics, and potentially updates visualization data (this can optionally
/// be forced). Also increments the burst counter index
#[derive(Copy, Clone)]
pub struct NeuronDynamicsPhase;
