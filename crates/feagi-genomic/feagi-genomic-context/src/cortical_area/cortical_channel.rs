use feagi_data::create_wrapped_index;

create_wrapped_index!(
    /// Index for addressing specific channels within an I/O cortical area area.
    ///
    /// Cortical areas can contain multiple channels for processing different
    /// aspects of data. This index addresses individual channels within a
    /// specific cortical_area area for fine-grained data routing.
    pub CorticalChannelIndex, u32
);

create_wrapped_index!(
    /// The number of cortical area channels within a IO cortical area
    pub CorticalChannelCount, u32
);

create_wrapped_index!(
    /// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
    pub CorticalChannelNeuronDepth, u32
);

// TODO spatial