use crate::neuron_model::interfacing::cortical_area_spec::NeuronInitializer;
use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use core::fmt::Debug;
use model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};

pub mod interfacing;
pub mod neuron_layout;
pub mod model_data;

/// Describes a specific neuron model for some usecase at a specific quantization level, as well
/// as the data its dynamics use
pub trait NeuronModel: MembranePotentialQuantization {
    // This trait is to be expanded on per model, which may restrict `LayoutTag` on its sub trait

    /// How the neurons of this model are laid out, for any [`FeagiIndexQuantization`]
    type LayoutTag: NeuronLayoutTag;

    /// A way of initializing neurons specific to this model. Use `NoCustomNeuronInit` if there is
    /// none
    type CustomNeuronInit: NeuronInitializer<Self>;

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

    /// The cortical level data that [`Self::NeuronDataVariance`] is derived from
    type CorticalDataVariance: CorticalDataVarianceSpecification<Self>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: NeuronDataInternalSpecification<Self>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: NeuronDataScratchSpecification<Self>;

    /// The per neuron level data derived from [`Self::CorticalDataVariance`]. Read only to neuron
    /// dynamics and never saved, as it can be generated again
    type NeuronDataVariance: NeuronDataVarianceSpecification<Self>;

    //endregion

}

/// Names one neuron model as a whole, across its quantization levels, and the layout it is fixed
/// to. Lets something differ per model, without needing to name each quantization level
pub trait NeuronModelTag {
    /// How the neurons of this model are laid out
    type LayoutTag: NeuronLayoutTag;
}


/// Returns the Neuron Layout of the given Neuron Model at the given index quantization
pub type NeuronLayoutOf<NM, FIQ> = <<NM as NeuronModel>::LayoutTag as NeuronLayoutTag>::Layout<FIQ>;

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

pub mod wrapped_indexing {
    use feagi_basis::create_wrapped_quantized_unsigned_integer;

    create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a cortical area within a burst engine
    pub CorticalAreaEngineIndex
);

    create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a cortical area within a model and quantization
    pub NeuronModelCorticalQuantIndex
);

    create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a neuron within a burst engine
    pub NeuronEngineIndex
);

    create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a neuron within a model and quantization
    pub NeuronModelNeuronQuantIndex
);
}



