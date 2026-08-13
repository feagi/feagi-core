
#[derive(Clone, Copy, PartialEq)]
pub enum BurstEngineWorkerCommand {
    /// Runs a full burst, Neuron phase then synapse phase
    RunFullBurst,
    
    // TODO edit command
    
    // TODO exchange visual, motor, sensor data
    
    // TODO force fire
    
    // TODO probe
    
    /// Hard terminate this worker https://www.youtube.com/watch?v=x012BnKWi3g&t=97s
    CommitSudoku
}