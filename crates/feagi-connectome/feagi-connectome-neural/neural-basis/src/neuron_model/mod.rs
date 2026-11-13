use crate::neuron_model::neurogenesis::by_generator::NeurogenesisByGenerator;
use crate::neuron_model::neurogenesis::{NeurogenesisMethod, NeurogenesisStorageOrGenerator};
use crate::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutRestrictor, NeuronLayoutTag};
use crate::quantization_levels::FeagiIndexQuantization;
use crate::quantization_levels::MembranePotentialQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use model_data::{
    CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification,
    NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification,
};
use neurogenesis::scratch_variance_defaults::NeurogenesisScratchVarianceDefaults;
use neurogenesis::validation::{NeurogenesisValidationError, NeurogenesisValidator, ValidatedNeurogenesisMethod};
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;
use crate::neuron_model::neurogenesis::validation::ValidatedNeurogenesisStorageOrGenerator;

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
pub trait NeuronModel: MembranePotentialQuantization 
+ Into<<Self as NeuronModel>::QuantedKind> + TryFrom<<Self as NeuronModel>::QuantedKind> {

    /// What neuron layout this neuron model expects
    type NeuronLayoutTag: NeuronLayoutTag;


    /// Trait for producing default variance and scratch values during NeuroGenesis
    type NeurogenesisScratchVarianceDefaults<FIQ: FeagiIndexQuantization>: NeurogenesisScratchVarianceDefaults<FIQ, Self>;

    /// The genome saveable data a cortical area of this model is generated from
    type NeurogenesisByGenerator<FIQ: FeagiIndexQuantization>: NeurogenesisByGenerator<FIQ, Self>;

    // NOTE: NeurogenesisStoragePrecursor is a struct! 

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


    /// The Quant Kind enum for this neuron model
    type QuantKind;
    
    /// The Quanted Kind enum for this neuron model, actually wraps the marker
    type QuantedKind;

    
    /// The Quant of this model represented as the Quant Kind Enum wrapper
    const QUANT_KIND: Self::QuantKind;

    /// The Quanted Kind enum for this neuron model, actually wraps the marker
    const QUANTED_KIND: Self::QuantedKind;
    
    //region Default Impls

    // NOTE: Many of these are defined here due to uses in the enums

    // NOTE: Explicitly No spawning function for `NeurogenesisGeneratorPrecursor` as
    // different cortical types may wish to have some differing implementations (though the
    // NM does specify the generator and precursor type) IE different defaults

    
    /// Statically generate the cortical scratch data during NeuroGenesis. Proxies through
    /// Self::NeurogenesisScratchVarianceGeneratorType
    fn generate_cortical_scratch<FIQ: FeagiIndexQuantization>(
        layout: &<Self::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &Self::CorticalDataInternal,
    ) -> Self::CorticalDataScratch {
        Self::NeurogenesisScratchVarianceDefaults::static_get_cortical_scratch(
            layout,
            cortical_properties,
            cortical_internal
        )
    }

    /// Statically generate the neuron scratch data during NeuroGenesis. Proxies through
    /// Self::NeurogenesisScratchVarianceGeneratorType
    fn generate_neuron_scratch<FIQ: FeagiIndexQuantization>(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &<Self::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        internal_neuron_data: &Self::NeuronDataInternal
    ) -> Self::NeuronDataScratch {
        Self::NeurogenesisScratchVarianceDefaults::static_get_neuron_scratch(
            index,
            layout,
            internal_neuron_data
        )
    }

    /// Statically generate the neuron variance data during NeuroGenesis. Proxies through
    /// Self::NeurogenesisScratchVarianceGeneratorType
    fn generate_neuron_variance<FIQ: FeagiIndexQuantization>(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &<Self::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        variance: &Self::CorticalDataVariance
    ) -> Self::NeuronDataVariance {
        Self::NeurogenesisScratchVarianceDefaults::static_get_neuron_variance(
            index,
            layout,
            variance
        )
    }

    fn verify_and_wrap_generator<
        FIQ: FeagiIndexQuantization,
        Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>,
        V: NeurogenesisValidator<FIQ, Self>,
    > (generator: Self::NeurogenesisByGenerator<FIQ>)
        -> Result<ValidatedNeurogenesisMethod<FIQ, Self, Self::NeurogenesisByGenerator<FIQ>>, NeurogenesisValidationError> {
        
        ValidatedNeurogenesisMethod::validate_and_wrap::<V, Res>(generator)
    }

    fn verify_and_wrap_storage<
        FIQ: FeagiIndexQuantization,
        Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>,
        V: NeurogenesisValidator<FIQ, Self>,
    > (storage: NeurogenesisByStorage<FIQ, Self> )
        -> Result<ValidatedNeurogenesisMethod<FIQ, Self, NeurogenesisByStorage<FIQ, Self>>, NeurogenesisValidationError> {
        
        ValidatedNeurogenesisMethod::validate_and_wrap::<V, Res>(storage)
    }

    fn verify_and_wrap_storage_or_generator<
        FIQ: FeagiIndexQuantization,
        Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, Self>>,
        V: NeurogenesisValidator<FIQ, Self>,
    > (storage_or_generator: NeurogenesisStorageOrGenerator<FIQ, Self>)
        -> Result<ValidatedNeurogenesisStorageOrGenerator<FIQ, Self>, NeurogenesisValidationError> {
        
        match storage_or_generator {
            NeurogenesisStorageOrGenerator::Storage(s) => {
                Ok(ValidatedNeurogenesisStorageOrGenerator::Storage(Self::verify_and_wrap_storage(s)?))
            }
            NeurogenesisStorageOrGenerator::Generator(g) => {
                Ok(ValidatedNeurogenesisStorageOrGenerator::Generator(Self::verify_and_wrap_generator(g)?))
            }
        }

    }
    
    //endregion
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
