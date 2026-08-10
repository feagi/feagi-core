/// Per neuron properties that all neurons have that can be configured
#[derive(Clone, Copy, Debug)]
pub struct NeuronProperties {
    pub probe_force_disabled: bool,
    pub probe_force_firing: bool,
}

impl Default for NeuronProperties {
    fn default() -> Self {
        Self {
            probe_force_disabled: false,
            probe_force_firing: false,
        }
    }
}
