use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Describes the quantization level of a specific CorticalDynamicsModel data points
pub trait CorticalDynamicsQuantization: MembranePotentialQuantization {
    // This trait is to be expanded on per model

    // NOTE: The model using this quantization (`type Model`) is defined on the per model sub trait
    // instead, as models may fix or further restrict their NeuronLayout, which a generic
    // associated type here could not allow

    //region Cortical level Data

    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    type CorticalDataProperties: CorticalDataPropertiesSpecification<Self>;

    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type CorticalDataInternal: CorticalDataInternalSpecification<Self>;

    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type CorticalDataScratch: CorticalDataScratchSpecification<Self>;

    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    type CorticalFunctionContext: CorticalFunctionSpecification<Self>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: NeuronDataInternalSpecification<Self>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: NeuronDataScratchSpecification<Self>;

    //endregion
    
}

//region Specifications
// NOTE: These are each extended by a given model

/// The cortical level data that should be exposed to genome developers. Not mutable during
/// cortical processing
pub trait CorticalDataPropertiesSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that is for mutable internal processing, will not be
/// exposed to genome developers but is saved in the connectome
pub trait CorticalDataInternalSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that is used for runtime processing, is not exposed to
/// genome developers nor is it saved (starts clean with every init)
pub trait CorticalDataScratchSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}

/// Not actual cortical data, but rather represents various "values" that are actually the result
/// of running functions on the neuron data
pub trait CorticalFunctionSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}

/// The per neuron level data that should be saved to the connectome
pub trait NeuronDataInternalSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}

/// The per neuron level data that should not be saved to the connectome
/// (starts clean with every init)
pub trait NeuronDataScratchSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}



//endregion
