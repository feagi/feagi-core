use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use core::fmt::{Debug, Formatter};
use feagi_basis::collections::quant_indexed::quant_indexed::{QuantContiguousStore, QuantContiguousStoreBackend, QuantIndexedBackend};
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use serde::{Deserialize, Serialize};
use crate::neuron_model::data::neuron_layout::NeuronLayout;

#[cfg(feature = "alloc")]
use feagi_basis::collections::quant_indexed::quant_indexed::QuantContiguousStoreBackendVec;

// TODO Move away from pub properties and instead use getters and setters with proper checks!
// We dont want to be able to set the layout to something that wont agree with the neuron count!


/// The full data of a given cortical model to be stored (everything except scratch data), with
/// the neuron data held in a store of the backend `SB`. Serializable when the store of `SB` is
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "SB::Store<NM::NeuronDataInternal>: Serialize",
    deserialize = "SB::Store<NM::NeuronDataInternal>: Deserialize<'de>",
))] // Every other field is serializable through the bounds of NeuronModel and NeuronLayout
pub struct CorticalModelFullStorageData<FIQ, NM, SB>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    SB: QuantContiguousStoreBackend
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: NeuronLayoutOf<NM, FIQ>,
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
    pub neuron_internal: QuantIndexedBackend<CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>, SB, NM::NeuronDataInternal>,
}

impl<FIQ, NM, SB> Debug for CorticalModelFullStorageData<FIQ, NM, SB>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    SB: QuantContiguousStoreBackend
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CorticalModelFullStorageData")
            .field("layout", &self.layout)
            .field("cortical_properties", &self.cortical_properties)
            .field("cortical_internal", &self.cortical_internal)
            .field("neuron_variance_properties", &self.neuron_variance_properties)
            .field("cortical_level_psp", &self.cortical_level_psp)
            .field("is_psp_uniform", &self.is_psp_uniform)
            .field("is_psp_using_membrane_potential", &self.is_psp_using_membrane_potential)
            .field("neuron_internal", &self.neuron_internal.store().store_as_slice())
            .finish()
    }
}

impl<FIQ, NM, SB> Clone for CorticalModelFullStorageData<FIQ, NM, SB>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    SB: QuantContiguousStoreBackend,
    SB::Store<NM::NeuronDataInternal>: Clone
{
    fn clone(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            cortical_properties: self.cortical_properties.clone(),
            cortical_internal: self.cortical_internal.clone(),
            neuron_variance_properties: self.neuron_variance_properties.clone(),
            cortical_level_psp: self.cortical_level_psp.clone(),
            is_psp_uniform: self.is_psp_uniform,
            is_psp_using_membrane_potential: self.is_psp_using_membrane_potential,
            neuron_internal: self.neuron_internal.clone(),
        }
    }
}

/// Neuron data is compared by its elements, regardless of the capacity of its store
impl<FIQ, NM, SB> PartialEq for CorticalModelFullStorageData<FIQ, NM, SB>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
    SB: QuantContiguousStoreBackend
{
    fn eq(&self, other: &Self) -> bool {
        self.layout == other.layout
            && self.cortical_properties == other.cortical_properties
            && self.cortical_internal == other.cortical_internal
            && self.neuron_variance_properties == other.neuron_variance_properties
            && self.cortical_level_psp == other.cortical_level_psp
            && self.is_psp_uniform == other.is_psp_uniform
            && self.is_psp_using_membrane_potential == other.is_psp_using_membrane_potential
            && self.neuron_internal.store().store_as_slice() == other.neuron_internal.store().store_as_slice()
    }
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
    pub layout: NeuronLayoutOf<NM, FIQ>,
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
    pub layout: NeuronLayoutOf<NM, FIQ>,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: NM::CorticalDataInternal,
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
}

/// Every piece of data needed to spawn a new cortical area, at the given model quantization via 
/// the defaults spawner. Built from a set of model defaults and the given cortical properties
#[derive(Debug, Clone, PartialEq)] // No need to serialize this, this is transient
pub struct CorticalModelSpawnData<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    pub layout: NeuronLayoutOf<NM, FIQ>,
    /// Cortical level properties of this model at the given model quantization
    pub cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    pub cortical_internal: NM::CorticalDataInternal,
    /// Runtime cortical level data, not saved
    pub cortical_scratch: NM::CorticalDataScratch,
    /// Not direct properties, but rather parameters used by functions to set per neuron variance
    pub neuron_variance_properties: NM::CorticalFunctionContext,
    /// The data every neuron starts with that is saved
    pub neuron_internal: NM::NeuronDataInternal,
    /// The data every neuron starts with that is not saved
    pub neuron_scratch: NM::NeuronDataScratch,
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_membrane_potential: bool,
}

