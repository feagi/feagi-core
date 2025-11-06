

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PercentageNeuronPositioning {
    Linear,
    Fractional,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameChangeHandling {
    Absolute,
    Incremental,
}