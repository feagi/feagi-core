
/// Used for counting and indexing specific channels within an I/O cortical area.
pub type CorticalChannelIndexCount = generics::CorticalChannelIndexCountQuant<u32>;

/// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
pub type CorticalChannelNeuronDepth = generics::CorticalChannelNeuronDepthQuant<u32>;

pub mod generics {
    use feagi_data::create_wrapped_quantized_unsigned_integer;

    create_wrapped_quantized_unsigned_integer!(
    /// Used for counting and indexing specific channels within an I/O cortical area.
    pub CorticalChannelIndexCountQuant
    );

    create_wrapped_quantized_unsigned_integer!(
        /// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
        pub CorticalChannelNeuronDepthQuant
    );
}
