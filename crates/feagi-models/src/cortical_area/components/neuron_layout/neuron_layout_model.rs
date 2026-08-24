
/// Root trait for defining Neuron Layout
pub trait NeuronLayoutModelTrait
{
    /// What layout the neurons are using
    const NEURON_LAYOUT_MODEL: NeuronLayoutModelEnum;
}

pub enum NeuronLayoutModelEnum {
    Dimensional,
    Linear
}