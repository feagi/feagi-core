use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use core::fmt::Debug;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use serde::de::DeserializeOwned;
use serde::Serialize;
// NOTE: How this works is a sub trait is made FIRST per neuron model, then there are
// implementations under that, one per quantization level of the model

/// Describes a specific neuron model for some usecase at a specific quantization level, as well
/// as the data its dynamics use
pub trait NeuronModel: MembranePotentialQuantization {
    // This trait is to be expanded on per model, which may restrict `LayoutType` on its sub trait

    /// How the neurons of this model are laid out, for any [`FeagiIndexQuantization`]
    type LayoutType: NeuronLayoutType;

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

/// Selects a [`NeuronLayout`] independently of [`FeagiIndexQuantization`], so a model sub trait
/// can restrict its layout without an `FIQ` in scope (which stable Rust cannot express on a
/// generic associated type directly)
pub trait NeuronLayoutType: Clone + Copy + Debug + PartialEq + Sync + Send + Serialize + DeserializeOwned + 'static {
    /// The layout at a given [`FeagiIndexQuantization`]
    type Layout<FIQ: FeagiIndexQuantization>: NeuronLayout<FIQ>;
}







//region Specifications
// NOTE: These are each extended by a given model

/// The cortical level data that should be exposed to genome developers. Not mutable during
/// cortical processing
pub trait CorticalDataPropertiesSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that is for mutable internal processing, will not be
/// exposed to genome developers but is saved in the connectome
pub trait CorticalDataInternalSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that is used for runtime processing, is not exposed to
/// genome developers nor is it saved (starts clean with every init)
pub trait CorticalDataScratchSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// Not actual cortical data, but rather represents various "values" that are actually the result
/// of running functions on the neuron data
pub trait CorticalFunctionSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}


/// The per neuron level data that should be saved to the connectome
pub trait NeuronDataInternalSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The per neuron level data that should not be saved to the connectome
/// (starts clean with every init)
pub trait NeuronDataScratchSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}



//endregion














/// The [`NeuronLayout`] of neuron model `NM` at [`FeagiIndexQuantization`] `FIQ`
pub type NeuronLayoutOf<NM, FIQ> = <<NM as NeuronModel>::LayoutType as NeuronLayoutType>::Layout<FIQ>;



/// This marker trait is used to implement per neuron model dynamics per burst engine
/// backend. Each burst engine has an extension of this trait, then a concrete implementation
/// per neuron model to handle its math in the way that burst engine does so. Since
/// Burst Engines differ so wildly in implementation, there are no functions on this trait.
/// For a general reference for how a specific neuron model operates its math, see the CPU
/// implementation
pub trait NeuralBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{}



/// Defaults of the cortical level data every neuron model has, defined at the quantization of
/// [`Self::Reference`]. Meant to be implemented on a zero sized struct, which then implements the
/// defaults trait of its model as well (such as `FeagiAdvancedDefaults`), which assembles them
pub trait CorticalUniversalDefaults {
    /// The neuron model (and so quantization level) the defaults are defined at
    type Reference: NeuronModel;

    const CORTICAL_LEVEL_PSP: CorticalAreaNeuronPotential<<Self::Reference as MembranePotentialQuantization>::MembranePotentialQuant>;
    const IS_PSP_UNIFORM: bool;
    const IS_PSP_USING_MEMBRANE_POTENTIAL: bool;
}