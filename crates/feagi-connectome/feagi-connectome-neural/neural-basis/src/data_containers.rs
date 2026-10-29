use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use feagi_basis::collections::par_data::par_data::ParDataStore;
use feagi_basis::neurons::CorticalAreaNeuronPotential;
use feagi_basis::prelude::ParData;
use crate::cortical_area::cortical_dynamics_model::CorticalDynamicsModel;
use crate::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
use crate::cortical_area::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// TODO Move away from pub properties and instead use getters and setters with proper checks!
// We dont want to be able to set the layout to something that wont agree with the neuron count!

//region Cortical Model

/// The full data of a given cortical model to be stored (everything except scratch data)
pub struct CorticalModelFullStorageData<FIQ, NL, CDM, CDQ, Par>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>,
    Par: ParDataStore<Elem=CDQ::NeuronDataInternal>
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: NL,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: CDQ::CorticalDataProperties,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: CDQ::CorticalFunctionContext,
    pub cortical_level_psp: CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
    /// The stored neuron data of all neurons
    pub neuron_internal: ParData<FIQ, Par>,
    _p: PhantomData<(FIQ, CDM)>
}



/// Cortical level data (properties and neuron variance) as would be edited by a genome developer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorticalModelGenomePropertiesData<FIQ, NL, CDM, CDQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: NL,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: CDQ::CorticalDataProperties,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: CDQ::CorticalFunctionContext,
    pub cortical_level_psp: CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
    _p: PhantomData<(FIQ, CDM)>
}

/// Cortical level data (properties and internal) needed to load a cortical area into the NPU
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorticalModelNPUInterfaceData<FIQ, NL, CDM, CDQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CDQ: CorticalDynamicsQuantization,
    CDM: CorticalDynamicsModel<FIQ, NL, CDQ>
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: NL,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: CDQ::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: CDQ::CorticalDataInternal,
    pub cortical_level_psp: CorticalAreaNeuronPotential<CDQ::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
    _p: PhantomData<(FIQ, CDM)>
}

//endregion