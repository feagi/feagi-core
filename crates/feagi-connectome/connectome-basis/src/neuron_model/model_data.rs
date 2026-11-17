use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutOf};
use crate::neuron_model::{NeuronModel};
use crate::quantization_levels::FeagiIndexQuantization;
use core::fmt::Debug;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Data used by all Neuron Models
#[derive(Debug, Clone, Copy, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
// Serializable through the quantization of NM, not the bounds serde infers
#[serde(bound = "")]
pub struct NeuronModelUniversalCorticalProperties<NM>
where
    NM: NeuronModel
{
    /// The psp to use on neuron fire if psp_using_neuron_potential is disabled
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_neuron_potential: bool
}


/// The cortical level data that should be exposed to genome developers. Not mutable during
/// cortical processing
pub trait CorticalDataPropertiesSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that is for mutable internal processing, will not be
/// exposed to genome developers but is saved in the connectome
pub trait CorticalDataInternalSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that is used for runtime processing, is not exposed to
/// genome developers nor is it saved (starts clean with every init)
pub trait CorticalDataScratchSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The cortical level data that the per neuron variance data is derived from. Exposed to genome
/// developers, and editable during runtime by writing the per neuron variance data again
pub trait CorticalDataVarianceSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Serialize + DeserializeOwned + Sync + Send {
    /// What the variance data of a neuron should be at the given index
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NM::NeuronDataVariance;

    /// Writes the variance data of every neuron into a preallocated slice, one element per neuron.
    /// Overwrites what was there, so no previous variance needs to be undone
    fn write_neuron_variance_into_slice<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<FIQ, NM>,
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
pub trait NeuronDataInternalSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Serialize
+ DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The per neuron level data that should not be saved to the connectome
/// (starts clean with every init)
pub trait NeuronDataScratchSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Serialize
+ DeserializeOwned + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}

/// The per neuron level data derived from the cortical variance. Read only to neuron dynamics, and
/// not saved to the connectome
pub trait NeuronDataVarianceSpecification<NM: NeuronModel>: Clone + Debug + PartialEq + Sync + Send {
    // Any cortical area model data should be defined here (for the specification)
}