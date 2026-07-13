
/// Controls aspects of the burst engine(s) themselves
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPURequestParametersBurstEngine {
    RunSingleBurst,
    // TODO Start / Stop / Frequency
}