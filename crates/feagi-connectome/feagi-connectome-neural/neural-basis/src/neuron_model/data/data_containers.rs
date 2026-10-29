use crate::neuron_model::neuron_model::{LayoutDataOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStore;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantIndexed;
use serde::{Deserialize, Serialize};

// TODO Move away from pub properties and instead use getters and setters with proper checks!
// We dont want to be able to set the layout to something that wont agree with the neuron count!

//region Cortical Model

/// The full data of a given cortical model to be stored (everything except scratch data)
pub struct CorticalModelFullStorageData<FIQ, NM, PDS>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    PDS: QuantContiguousStore<Elem=NM::NeuronDataInternal>
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: LayoutDataOf<NM, FIQ>,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: NM::CorticalDataInternal,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: NM::CorticalFunctionContext,
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
    /// The stored neuron data of all neurons
    pub neuron_internal: QuantIndexed<CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, PDS>,
}



/// Cortical level data (properties and neuron variance) as would be edited by a genome developer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")] // Every field is serializable through the bounds of NeuronModel and NeuronLayout
pub struct CorticalModelGenomePropertiesData<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: LayoutDataOf<NM, FIQ>,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: NM::CorticalFunctionContext,
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
}

/// Cortical level data (properties and internal) needed to load a cortical area into the NPU
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")] // Every field is serializable through the bounds of NeuronModel and NeuronLayout
pub struct CorticalModelNPUInterfaceData<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: LayoutDataOf<NM, FIQ>,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: NM::CorticalDataInternal,
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
}

//endregion