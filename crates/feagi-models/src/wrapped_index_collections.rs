// TODO the following macro should be common throughout

// TODO dissolve this whole subcrate and put it in neuron model

use feagi_data::{
    create_wrapped_contiguous_slice, create_wrapped_contiguous_slice_mut, create_wrapped_contiguous_vector, create_wrapped_quantized_unsigned_integer_index,
};



/// Given a name and level, creates a linear index type, and slice, slicemut, vector structs of it
macro_rules! make_index_and_linear_collections {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        ::paste::paste! {
            create_wrapped_quantized_unsigned_integer_index!(
                $(#[$meta])*
                pub [<$name Index>]
            );

            create_wrapped_contiguous_slice!(
                $(#[$meta])*
                pub [<$name IndexedSlice>],
                [<$name Index>]
            );

            create_wrapped_contiguous_slice_mut!(
                $(#[$meta])*
                pub [<$name IndexedSliceMut>],
                [<$name Index>],
                [<$name IndexedSlice>]
            );

            create_wrapped_contiguous_vector!(
                $(#[$meta])*
                pub [<$name IndexedVector>],
                [<$name Index>],
                [<$name IndexedSlice>],
                [<$name IndexedSliceMut>]
            );
        }
    };
}

// Neuron Indexing
make_index_and_linear_collections!(
    /// Neuron Indexed at the connectome level
    NeuronConnectome);
make_index_and_linear_collections!(
    /// Neuron Indexed at the current burst engine level
    NeuronEngine);
make_index_and_linear_collections!(
    /// Neuron Indexed at the activity byte (visualization) level
    NeuronEngineByte);
make_index_and_linear_collections!(
    /// Neuron Indexed at the history (all neurons in the engine that have a history) level
    NeuronHistory);
make_index_and_linear_collections!(
    /// Neuron Indexed at the membrane potential quantization level in the engine, of a given MP quantization level
    NeuronMP);
make_index_and_linear_collections!(
    /// Neuron Indexed at the array for psp uniformity divisor
    NeuronPSPUniform);
make_index_and_linear_collections!(
    /// Neuron indexed at the neuron model and quantization level within the engine
    NeuronModel);

// Cortical Areas
make_index_and_linear_collections!(
    /// Cortical area indexed at the connectome level
    CorticalConnectome);
make_index_and_linear_collections!(
    /// Cortical area indexed at the burst engine level
    CorticalEngine);
make_index_and_linear_collections!(
    /// Cortical area indexed at the neuron model and quantization level within the engine
    CorticalModel);
make_index_and_linear_collections!(
    /// Cortical area indexed for layout. Is typed depending on layout type
    CorticalLayout);

// Cortical Mapping Entry
make_index_and_linear_collections!(
    /// Cortical mapping entry indexed at the connectome level
    MappingEntryConnectome);
make_index_and_linear_collections!(
    /// Cortical mapping entry indexed at the burst engine level
    MappingEntryEngine);
make_index_and_linear_collections!(
    /// Cortical mapping entry indexed at the synapse model and quantization level within the engine
    MappingEntryModel);

// Synapse
make_index_and_linear_collections!(
    /// Synapse indexed at the connectome level
    SynapseConnectome);

make_index_and_linear_collections!(
    /// Synapse indexed at the current burst engine level
    SynapseEngine);

make_index_and_linear_collections!(
    /// Synapse indexed at the synapse model and quantization level in the burst engine
    SynapseModel);

make_index_and_linear_collections!(
    /// Synapse indexed at the level local to its owning Cortical Mapping Entry
    SynapseLocal);


 