
/// Instructs what the burst engine should do this upcoming cycle
#[derive(Clone, Copy, PartialEq)]
pub enum BurstEngineWorkerCommand {
    /// Runs a kernel
    RunKernel(BurstEngineWorkerKernelCommand),

    /// Requests the loaded connectome be edited in some manner. Drains any incoming data first
    EditConnectome(), // TODO, also this should be feature gated somehow

    // TODO Force Fire

    // TODO probe

    /// Hard terminate this worker https://www.youtube.com/watch?v=x012BnKWi3g&t=97s
    CommitSudoku
}

/// Instructs the burst engine to perform some sort of computation
#[derive(Clone, Copy, PartialEq)]
pub enum BurstEngineWorkerKernelCommand {
    FullNeuronSynapseBurst,
    MultipleFullNeuronSynapseBurst{number_bursts: u16},
    // TODO half bursts
    // TODO conditional data generation?
}

