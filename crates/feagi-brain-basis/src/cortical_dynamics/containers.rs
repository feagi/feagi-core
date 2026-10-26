use crate::cortical_dynamics::model::data_structs::cortical_area_property_flags::CorticalAreaPropertyFlags;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::par_data::par_data::ParDataStore;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::prelude::ParData;
use crate::cortical_dynamics::model::cortical_dynamics_model::{CorticalDynamicsModel, CorticalAreaModelNeuronSpecification};
use crate::cortical_dynamics::model::cortical_dynamics_quantization::CorticalDynamicsQuantization;


//region Storage
/// Generic struct that contains the storable (AKA not scratch) data for a given cortical area
/// model type and quantization
pub struct CorticalDynamicsCorticalStorage<CDM, CDQ>
where
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<CDQ>
{
    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    pub cortical_area_properties: CDM::CorticalDataProperties,
    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    pub cortical_area_internal: CDM::CorticalDataInternal,
    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    pub cortical_area_function_context: CDM::CorticalFunctionContext,
    // No scratch data included
    /// If PSP is cortical area driven (not membrane potential driven), use this value
    pub cortical_level_psp: CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
    /// Boolean flags used by all cortical areas
    pub cortical_area_flags: CorticalAreaPropertyFlags,
}

pub struct CorticalDynamicsNeuronStorage<FIQ, CDM, CDQ, Storage>
where
    FIQ: FeagiIndexQuantization,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<CDQ>,
    Storage: ParDataStore<Elem=CDM::NeuronDataInternal>

{
    /// Per Neuron Data that should be saved
    pub neuron_data_internal: ParData<FIQ::NeuronIndexQuant, Storage>,
    _p: core::marker::PhantomData<(CDM, CDQ)>
}

//endregion









