use crate::cortical_area::components::cortical_area_quantization::{CorticalAreaQuantization, CorticalAreaQuantizationNeuronSpecification};
use crate::cortical_area::data_structs::cortical_area_property_flags::CorticalAreaPropertyFlags;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::par_data::par_data::ParDataStore;
use feagi_basis::neurons::CorticalAreaNeuronPotential;


/// Generic struct that contains the storable (AKA not scratch) data for a given cortical area
/// model type and quantization
pub struct CorticalAreaModelCorticalStorage<CAQ: CorticalAreaQuantization> {
    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    pub cortical_area_properties: CAQ::CorticalDataProperties,
    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    pub cortical_area_internal: CAQ::CorticalDataInternal,
    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    pub cortical_area_function_context: CAQ::CorticalFunctionContext,
    // No scratch data included
    /// If PSP is cortical area driven (not membrane potential driven), use this value
    pub cortical_level_psp: CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>,
    /// Boolean flags used by all cortical areas
    pub cortical_area_flags: CorticalAreaPropertyFlags,
}

pub struct CorticalAreaModelNeuronStorage<FIQ, CAQ, Storage>
where
    FIQ: FeagiIndexQuantization,
    CAQ: CorticalAreaQuantization,
    Storage: ParDataStore
{
    /// The per neuron level data that should be saved to the connectome
    pub neuron_data_internal:
        <CAQ::NeuronDataInternal as CorticalAreaQuantizationNeuronSpecification<CAQ>>::Parallelized<FIQ, Storage>
}











