use feagi_structures::quantization::CorticalAreaNeuronQuantization;

/// Defines any parameters you want per neuron for a given neuron model.
/// This is what models shall expand on!
pub trait NeuronModelParametersTrait<CANQ: CorticalAreaNeuronQuantization>:
{

}