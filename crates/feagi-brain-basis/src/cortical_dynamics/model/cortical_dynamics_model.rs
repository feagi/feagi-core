use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::cortical_dynamics::model::cortical_dynamics_quantization::CorticalDynamicsQuantization;
// NOTE: How this works is a sub trait is made FIRST per cortical area model, then
// there are implementations under that

/// Describes a specific cortical area model for some usecase, as well as its dynamics
pub trait CorticalDynamicsModel<CDQ>
where
    CDQ: CorticalDynamicsQuantization, // More restricted per model
{
    
    //region Cortical level Data

    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    type CorticalDataProperties: CorticalAreaModelCorticalSpecification<CDQ>;

    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type CorticalDataInternal: CorticalAreaModelCorticalSpecification<CDQ>;

    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type CorticalDataScratch: CorticalAreaModelCorticalSpecification<CDQ>;

    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    type CorticalFunctionContext: CorticalAreaModelCorticalSpecification<CDQ>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: CorticalAreaModelNeuronSpecification<CDQ>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: CorticalAreaModelNeuronSpecification<CDQ>;

    //endregion
    
}


//region Specifications

/// A Cortical Area Model Specification at the cortical level, typed as singular elements
pub trait CorticalAreaModelCorticalSpecification<CDQ: CorticalDynamicsQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}



/// A Cortical Area Model Specification at the neuron level, typed as singular elements
pub trait CorticalAreaModelNeuronSpecification<CDQ: CorticalDynamicsQuantization> {}




