//! Not really a neuron dynamics model, just a base set of structs used to create the transport
//! voxel structures. This is not included in NPU dynamics!

// PseuedoPotential doesnt have properties other than the built in neruon potential!

//region Neuron

crate::__internal_neuron_generate_base_neuron_structs_and_traits!{
    PseuedoPotential,
    PseuedoPotential,
    {

    }
}

//endregion