use crate::neuron_model::neurogenesis::NeurogenesisScratchVarianceGenerator;
use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::quantization_levels::MembranePotentialQuantization;
use crate::quantization_levels::FeagiIndexQuantization;
use model_data::{
    CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification,
    NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification,
};
use neurogenesis::by_generator::NeurogenesisByGenerator;

//pub mod interfacing;
pub mod model_data;
pub mod neuron_layout;
pub mod neurogenesis;
pub mod metamorphosis;
pub mod neuron_pattern;

/// Describes a specific neuron model for some usecase at a specific quantization level, as well
/// as the data its dynamics use and the neuron layout it expects. Note that this trait
/// is extended to a certain models trait before being implemented, this is not implemented
/// directly
pub trait NeuronModel: MembranePotentialQuantization {
    
    /// What neuron layout this neuron model expects
    type NeuronLayoutTag: NeuronLayoutTag;
    
    type NeurogenesisDefaults<FIQ: FeagiIndexQuantization>: NeurogenesisScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisGenerator<FIQ: FeagiIndexQuantization>: NeurogenesisByGenerator<FIQ, Self>;


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
    
    
    
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: NeuronDataInternalSpecification<Self>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: NeuronDataScratchSpecification<Self>;

    /// The per neuron level data derived from [`Self::CorticalDataVariance`]. Read only to neuron
    /// dynamics and never saved, as it can be generated again
    type NeuronDataVariance: NeuronDataVarianceSpecification<Self>;
    
}



/// This marker trait is used to implement per neuron model dynamics per burst engine
/// backend. Each burst engine has an extension of this trait, then a concrete implementation
/// per neuron model to handle its math in the way that burst engine does so. Since
/// Burst Engines differ so wildly in implementation, there are no functions on this trait.
/// For a general reference for how a specific neuron model operates its math, see the CPU
/// implementation `CorticalCPUParallelDynamics`
pub trait NeuralBurstEngineDynamics<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
}

pub mod wrapped_indexing {
    use feagi_basis::create_wrapped_quantized_unsigned_integer;

    create_wrapped_quantized_unsigned_integer!(
        /// Represents the index of a cortical area within a burst engine
        pub CorticalAreaEngineIndex
    );

    create_wrapped_quantized_unsigned_integer!(
        /// Represents the index of a cortical area within a model and quantization
        pub CorticalAreaQuantIndex
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
