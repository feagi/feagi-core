use feagi_structures::quantization::CorticalAreaNeuronQuantization;

/// This trait is used as a factory to generate cortical areas
pub trait FeagiNeuronModelDefinition<CANQ: CorticalAreaNeuronQuantization> {
    // TODO neuron model neuron parameters

    // TODO neuron model cortical area parameters

    // TODO generate cortical area (types?)

}

/// Some metadata for a neuronal model, only applied in cpu builds
pub trait FeagiNeuronModelMetadata<CANQ: CorticalAreaNeuronQuantization>
: FeagiNeuronModelDefinition<CANQ>
{
    /// The name of the neuronal model
    const MODEL_NAME: &'static str;
    /// A rapid description of how the model works
    const SHORT_DESCRIPTION: &'static str;
}

