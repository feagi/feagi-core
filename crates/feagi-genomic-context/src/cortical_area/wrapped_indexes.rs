use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};

type StandardNeuronQuantization = <FeagiIndexQuantizationStandard as FeagiIndexQuantization>::NeuronIndexQuant;

/// Used for counting and indexing specific channels within an I/O cortical area.
pub type CorticalChannelIndexCount = generics::CorticalChannelIndexCountQuant<StandardNeuronQuantization>;

/// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
pub type CorticalChannelNeuronDepth = generics::CorticalChannelNeuronDepthQuant<StandardNeuronQuantization>;

pub mod generics {
    use feagi_data::create_wrapped_quantized_index;

    create_wrapped_quantized_index!(
    /// Used for counting and indexing specific channels within an I/O cortical area.
    pub CorticalChannelIndexCountQuant
    );

    create_wrapped_quantized_index!(
        /// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
        pub CorticalChannelNeuronDepthQuant
    );
}
