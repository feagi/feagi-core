use core::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::interfacing::cortical_area_spec::NeuronInitializer;
use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutType};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

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

    /// A way of initializing neurons specific to this model. Use `NoCustomNeuronInit` if there is
    /// none
    type CustomNeuronInit: NeuronInitializer<Self>;

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

/// The cortical level data that the per neuron variance data is derived from. Exposed to genome
/// developers, and editable during runtime by writing the per neuron variance data again
pub trait CorticalDataVarianceSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    /// What the variance data of a neuron should be at the given index
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NM::NeuronDataVariance;

    /// Writes the variance data of every neuron into a preallocated slice, one element per neuron.
    /// Overwrites what was there, so no previous variance needs to be undone
    fn write_neuron_variance_into_slice<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        variance: &mut [NM::NeuronDataVariance],
    ) {
        assert_eq!(variance.len(), layout.get_number_neurons_to_allocate_for().quant_to_usize());
        for (i, variance_out) in variance.iter_mut().enumerate() {
            *variance_out = self.neuron_variance_at::<FIQ>(layout, &CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i));
        }
    }
    
    // TODO rayon
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

/// The per neuron level data derived from the cortical variance. Read only to neuron dynamics, and
/// not saved to the connectome
pub trait NeuronDataVarianceSpecification<NM: NeuronModel>:
Clone + Debug + PartialEq + Sync + Send {
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
    const IS_PSP_USING_NEURON_POTENTIAL: bool;
}