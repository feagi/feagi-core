use crate::cortical_area::model::data_structs::cortical_area_property_flags::CorticalAreaPropertyFlags;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::par_data::par_data::ParDataStore;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use crate::cortical_area::model::cortical_area_model::{CorticalAreaModel, CorticalAreaModelNeuronSpecification};

/// Generic struct that contains the storable (AKA not scratch) data for a given cortical area
/// model type and quantization
pub struct CorticalAreaModelCorticalStorage<CAM: CorticalAreaModel> {
    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    pub cortical_area_properties: CAM::CorticalDataProperties,
    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    pub cortical_area_internal: CAM::CorticalDataInternal,
    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    pub cortical_area_function_context: CAM::CorticalFunctionContext,
    // No scratch data included
    /// If PSP is cortical area driven (not membrane potential driven), use this value
    pub cortical_level_psp: CorticalAreaNeuronPotential<CAM::MembranePotentialQuant>,
    /// Boolean flags used by all cortical areas
    pub cortical_area_flags: CorticalAreaPropertyFlags,
}

pub struct CorticalAreaModelNeuronStorage<FIQ, CAM, Storage>
where
    FIQ: FeagiIndexQuantization,
    CAM: CorticalAreaModel,
    Storage: ParDataStore
{
    /// The per neuron level data that should be saved to the connectome
    pub neuron_data_internal:
        <CAM::NeuronDataInternal as CorticalAreaModelNeuronSpecification<CAM>>::Parallelized<FIQ, Storage>
}











