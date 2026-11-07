//! The neuron initializers commonly listed by neuron models. Nothing sets them apart from an
//! initializer specific to one model, they are only shared

use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
#[cfg(feature = "alloc")]
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_connectome_neural::neural_basis::neuron_model::interfacing::cortical_area_init::{GenomeNeuronInitializer, NeuronInitializer};
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutData;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

//region Uniform

/// Every neuron starts with the same data
// Clone, Debug and PartialEq are written out, as deriving them would require them of `NM` too
pub struct UniformNeuronInit<NM: NeuronModel> {
    pub internal: NM::NeuronDataInternal,
    pub scratch: NM::NeuronDataScratch,
}

impl<NM: NeuronModel> UniformNeuronInit<NM> {
    pub fn new(internal: NM::NeuronDataInternal, scratch: NM::NeuronDataScratch) -> Self {
        Self { internal, scratch }
    }
}

impl<NM: NeuronModel> NeuronInitializer<NM> for UniformNeuronInit<NM> {
    /// Fits any layout
    type Error = core::convert::Infallible;
    type GenomeLevelNeuronIntoType = Self;

    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        (self.internal.clone(), self.scratch.clone())
    }

    /// Small enough to be stored in a genome as is
    fn as_genome_level_neuron_initializer(&self) -> Self::GenomeLevelNeuronIntoType {
        self.clone()
    }
}

impl<NM: NeuronModel> GenomeNeuronInitializer for UniformNeuronInit<NM> {}

impl<NM: NeuronModel> Clone for UniformNeuronInit<NM> {
    fn clone(&self) -> Self {
        Self { internal: self.internal.clone(), scratch: self.scratch.clone() }
    }
}

impl<NM: NeuronModel> core::fmt::Debug for UniformNeuronInit<NM> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UniformNeuronInit").field("internal", &self.internal).field("scratch", &self.scratch).finish()
    }
}

impl<NM: NeuronModel> PartialEq for UniformNeuronInit<NM> {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal && self.scratch == other.scratch
    }
}

//endregion

//region Stored

/// Every neuron has its own data, such as when loaded from a connectome. Holds one element per
/// neuron, in neuron index order
// Clone, Debug and PartialEq are written out, as deriving them would require them of `NM` too
#[cfg(feature = "alloc")]
pub struct StoredNeuronInit<NM: NeuronModel> {
    pub internal: Vec<NM::NeuronDataInternal>,
    pub scratch: Vec<NM::NeuronDataScratch>,
    default_neuron: (NM::NeuronDataInternal, NM::NeuronDataScratch)
}

#[cfg(feature = "alloc")]
impl<NM: NeuronModel> StoredNeuronInit<NM> {
    pub fn new(
        internal: Vec<NM::NeuronDataInternal>,
        scratch: Vec<NM::NeuronDataScratch>,
        default_neuron: (NM::NeuronDataInternal, NM::NeuronDataScratch)
    ) -> Self {
        Self { internal, scratch, default_neuron }
    }
}

#[cfg(feature = "alloc")]
impl<NM: NeuronModel> NeuronInitializer<NM> for StoredNeuronInit<NM> {
    type Error = StoredNeuronInitError;
    type GenomeLevelNeuronIntoType = UniformNeuronInit<NM>;

    /// Errors unless exactly one element per neuron of the layout is held
    fn check_against_layout<FIQ: FeagiIndexQuantization>(&self, layout: &NeuronLayoutOf<NM, FIQ>) -> Result<(), Self::Error> {
        let expected = layout.get_number_neurons_to_allocate_for().quant_to_usize();
        if self.internal.len() != expected || self.scratch.len() != expected {
            return Err(StoredNeuronInitError::NeuronCountMismatch {
                expected,
                internal: self.internal.len(),
                scratch: self.scratch.len(),
            });
        }
        Ok(())
    }

    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        let index = index.quant_to_usize();
        (self.internal[index].clone(), self.scratch[index].clone())
    }

    fn as_genome_level_neuron_initializer(&self) -> Self::GenomeLevelNeuronIntoType
    {
        UniformNeuronInit::<NM> {
            internal: self.default_neuron.0.clone(),
            scratch: self.default_neuron.1.clone(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<NM: NeuronModel> Clone for StoredNeuronInit<NM> {
    fn clone(&self) -> Self {
        Self { internal: self.internal.clone(), scratch: self.scratch.clone(), default_neuron: self.default_neuron.clone() }
    }
}

#[cfg(feature = "alloc")]
impl<NM: NeuronModel> core::fmt::Debug for StoredNeuronInit<NM> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StoredNeuronInit").field("internal", &self.internal).field("scratch", &self.scratch).finish()
    }
}

#[cfg(feature = "alloc")]
impl<NM: NeuronModel> PartialEq for StoredNeuronInit<NM> {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal && self.scratch == other.scratch
    }
}

/// Why stored neuron data cannot initialize the neurons of a layout
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StoredNeuronInitError {
    #[error("Stored neuron data holds {internal} internal and {scratch} scratch elements, but the layout has {expected} neurons")]
    NeuronCountMismatch {
        expected: usize,
        internal: usize,
        scratch: usize,
    },
}

//endregion
