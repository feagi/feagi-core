// Stored neuron data is heap allocated
#[cfg(feature = "alloc")]
pub mod from_storage;
pub mod by_generator;
pub mod scratch_variance_defaults;
pub mod validation;

use crate::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
#[cfg(feature = "alloc")]
use crate::neuron_model::neurogenesis::by_generator::NeurogenesisOtherDataForStorageOf;
#[cfg(feature = "alloc")]
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;
use crate::neuron_model::neurogenesis::validation::NeurogenesisValidationError;
use crate::neuron_model::neuron_layout::{CorticalLayoutContextOf, NeuronLayoutOf};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// How a cortical area gets spawned, implemented by every precursor (the saveable form)
pub trait NeurogenesisMethod<FIQ, NM>: Clone + core::fmt::Debug + Send + Sync + Serialize + DeserializeOwned
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// The layout context, not yet validated
    fn get_cortical_layout_context(&self) -> &CorticalLayoutContextOf<FIQ, NM>;

    /// Cortical level data used by all models
    fn get_universal_cortical_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM>;

    fn get_model_cortical_properties(&self) -> &NM::CorticalDataProperties;

    fn get_model_cortical_internal(&self) -> &NM::CorticalDataInternal;

    fn get_model_cortical_variance(&self) -> &NM::CorticalDataVariance;

    fn into_model_cortical_data(self) -> (NM::CorticalDataProperties, NM::CorticalDataInternal);

    /// Structural checks against the built layout, run on every validation. Accepts everything by default
    fn check_against_layout(&self, _layout: &NeuronLayoutOf<FIQ, NM>) -> Result<(), NeurogenesisValidationError> {
        Ok(())
    }

    fn spawn_neuron_data_at(
        &self, // NOTE: make sure the implementation makes use of variance!
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch);

}

/// Either kind of saveable neurogenesis data of a cortical area
///
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Serialize, ::serde::Deserialize)]
#[serde(bound = "")]
pub enum NeurogenesisStorageOrGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    Storage(NeurogenesisByStorage<FIQ, NM>),
    Generator(NM::NeurogenesisByGenerator<FIQ>),
}

#[cfg(feature = "alloc")]
impl<FIQ, NM> NeurogenesisStorageOrGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Copies this as a generator. Storage is converted using `other_data`, a generator is cloned
    pub fn copy_as_generator(&self, other_data: NeurogenesisOtherDataForStorageOf<FIQ, NM>) -> NM::NeurogenesisByGenerator<FIQ> {
        match self {
            Self::Storage(storage) => storage.to_generator(other_data),
            Self::Generator(generator) => generator.clone(),
        }
    }
}