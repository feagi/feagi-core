use core::marker::PhantomData;
use feagi_basis::collections::par_data::par_data::ParDataStore;
use feagi_basis::prelude::*;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;

/// Common root trait shared by all Neuron Model Quantizations. This trait should be extended
/// by the given neuron model to add any quantization parameters for their given data. Contains
/// the typing information for the specification data structs as well
pub trait CorticalAreaQuantization: MembranePotentialQuantization {

    //region Cortical level Data

    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    type CorticalDataProperties: CorticalAreaQuantizationCorticalSpecification<Self>;

    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type CorticalDataInternal: CorticalAreaQuantizationCorticalSpecification<Self>;

    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type CorticalDataScratch: CorticalAreaQuantizationCorticalSpecification<Self>;

    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    type CorticalFunctionContext: CorticalAreaQuantizationCorticalSpecification<Self>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: CorticalAreaQuantizationNeuronSpecification<Self>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: CorticalAreaQuantizationNeuronSpecification<Self>;

    //endregion

}

/// A Cortical Area Model Specification at the cortical level, typed as singular elements
pub trait CorticalAreaQuantizationCorticalSpecification<MPQ: MembranePotentialQuantization> {
    // Any cortical area model data should be defined here (for the specification)
}

/// A Cortical Area Model Specification at the neuron level, typed as singular elements
pub trait CorticalAreaQuantizationNeuronSpecification<MPQ: MembranePotentialQuantization> {
    /// Instead of singular element types, get them as some form of linear collection
    type Parallelized<
        FIQ: FeagiIndexQuantization,
        Storage: ParDataStore
    >: CorticalAreaQuantizationNeuronParallelizedSpecification<MPQ, FIQ, Storage>;
}

/// Should have the same types as `CorticalAreaQuantizationNeuronSpecification` but as
/// `ParData` element types (a linear collection of them)
pub trait CorticalAreaQuantizationNeuronParallelizedSpecification<
    MPQ: MembranePotentialQuantization,
    FIQ: FeagiIndexQuantization,
    Storage: ParDataStore>
{
    // Define the same types here with the same name
    // but using ParData<FIQ::NeuronIndexQuant, CollectionType<Element>>
}




