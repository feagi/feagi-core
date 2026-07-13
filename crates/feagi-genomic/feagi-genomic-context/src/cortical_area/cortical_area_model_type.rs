#[derive(Debug, Default)]
pub enum CorticalAreaModelType {
    // Only option for non-interneuron areas, Basically LIF with some additions
    #[default]
    FeagiStandard,
    // More performant, least accurate
    LIF,
    // Medium performance and accuracy
    Izhikevich,
    // Low performance but most biologically accurate
    HodgkinHuxley,
}
