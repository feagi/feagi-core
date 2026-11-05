use crate::neuron_model::neuron_layout::NeuronLayout;
use crate::neuron_model::model_data::{CorticalDataVarianceSpecification, NeuronModelUniversalCorticalProperties};
use crate::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;

/// Decides the data a neuron starts with, from where it is in its cortical area
pub trait NeuronInitializer<NM: NeuronModel>: Clone + core::fmt::Debug + PartialEq + Send {
    /// What the neuron data should be at the given index
    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch);
}

/// The [`NeuronModel::CustomNeuronInit`] of models without a custom way to initialize neurons.
/// Cannot be constructed, so [`NeuronInit::Custom`] is unreachable for them
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoCustomNeuronInit {}

impl<NM: NeuronModel> NeuronInitializer<NM> for NoCustomNeuronInit {
    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        match *self {}
    }
}


/// How the neurons of a new cortical area get their starting data. The neuron variance data is not
/// part of this, as it is always derived from the cortical variance
#[derive(Debug, Clone, PartialEq)]
pub enum NeuronInit<NM: NeuronModel> {
    /// Every neuron starts with the same data
    Uniform {
        internal: NM::NeuronDataInternal,
        scratch: NM::NeuronDataScratch,
    },
    /// Every neuron has its own data, such as when loaded from a connectome
    #[cfg(feature = "alloc")]
    Stored {
        internal: Vec<NM::NeuronDataInternal>,
        scratch: Vec<NM::NeuronDataScratch>,
    },
    /// The way specific to the neuron model
    Custom(NM::CustomNeuronInit),
}

impl<NM: NeuronModel> NeuronInitializer<NM> for NeuronInit<NM> {
    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch) {
        match self {
            Self::Uniform { internal, scratch } => (internal.clone(), scratch.clone()),
            #[cfg(feature = "alloc")]
            Self::Stored { internal, scratch } => (internal[index.quant_to_usize()].clone(), scratch[index.quant_to_usize()].clone()),
            Self::Custom(custom) => custom.neuron_data_at::<FIQ>(layout, index),
        }
    }
}

/// Every piece of data needed to spawn a new cortical area, at the given model quantization
#[derive(Debug, Clone, PartialEq)] // No need to serialize this, this is transient
pub struct CorticalAreaSpec<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Details on how neurons are laid out (length, dimensions, etc)
    neuron_layout: NeuronLayoutOf<NM, FIQ>,
    universal_properties: NeuronModelUniversalCorticalProperties<NM>,
    /// Cortical level properties of this model at the given model quantization
    cortical_properties: NM::CorticalDataProperties,
    /// Stateful data that must be kept but not exposed to the genome developer for editing.
    /// May update every burst
    cortical_internal: NM::CorticalDataInternal,
    /// Runtime cortical level data, not saved
    cortical_scratch: NM::CorticalDataScratch,
    /// What the neuron variance data of the area is derived from
    cortical_variance: NM::CorticalDataVariance,
    neurons: NeuronInit<NM>,
}

impl<FIQ, NM> CorticalAreaSpec<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// Returns an error if [`NeuronInit::Stored`] does not hold exactly one element per neuron of
    /// the layout
    pub fn new(
        neuron_layout: NeuronLayoutOf<NM, FIQ>,
        universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        cortical_properties: NM::CorticalDataProperties,
        cortical_internal: NM::CorticalDataInternal,
        cortical_scratch: NM::CorticalDataScratch,
        cortical_variance: NM::CorticalDataVariance,
        neurons: NeuronInit<NM>,
    ) -> Result<Self, CorticalAreaSpecError> {
        #[cfg(feature = "alloc")]
        if let NeuronInit::Stored { internal, scratch } = &neurons {
            let neuron_count = neuron_layout.get_number_neurons_to_allocate_for().quant_to_usize();
            if internal.len() != neuron_count || scratch.len() != neuron_count {
                return Err(CorticalAreaSpecError::StoredNeuronCountMismatch);
            }
        }
        Ok(Self {
            neuron_layout,
            universal_properties,
            cortical_properties,
            cortical_internal,
            cortical_scratch,
            cortical_variance,
            neurons,
        })
    }

    /// Every neuron starts with the same data, which cannot fail
    pub fn new_uniform(
        neuron_layout: NeuronLayoutOf<NM, FIQ>,
        universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        cortical_properties: NM::CorticalDataProperties,
        cortical_internal: NM::CorticalDataInternal,
        cortical_scratch: NM::CorticalDataScratch,
        cortical_variance: NM::CorticalDataVariance,
        neuron_internal: NM::NeuronDataInternal,
        neuron_scratch: NM::NeuronDataScratch,
    ) -> Self {
        Self {
            neuron_layout,
            universal_properties,
            cortical_properties,
            cortical_internal,
            cortical_scratch,
            cortical_variance,
            neurons: NeuronInit::Uniform { internal: neuron_internal, scratch: neuron_scratch },
        }
    }

    /// Get the neuron layout that will be used
    pub fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ> {
        &self.neuron_layout
    }

    /// Get properties expected of all cortical areas
    pub fn get_cortical_universal_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM> {
        &self.universal_properties
    }

    /// Get the cortical level data needed for the cortical area
    pub fn get_cortical_level_data(&self) -> (
        &NM::CorticalDataProperties,
        &NM::CorticalDataInternal,
        &NM::CorticalDataScratch,
    ) {
        (&self.cortical_properties, &self.cortical_internal, &self.cortical_scratch)
    }

    /// Get what the neuron variance data of the area is derived from
    pub fn get_cortical_variance(&self) -> &NM::CorticalDataVariance {
        &self.cortical_variance
    }

    /// Number of neurons needed in the cortical area
    pub fn get_total_number_neurons_needed(&self) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        self.neuron_layout.get_number_neurons_to_allocate_for()
    }

    /// What the data of a neuron should be at the given index
    pub fn generate_neuron_data_at(
        &self,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch, NM::NeuronDataVariance) {
        let (internal, scratch) = self.neurons.neuron_data_at::<FIQ>(&self.neuron_layout, index);
        (internal, scratch, self.cortical_variance.neuron_variance_at::<FIQ>(&self.neuron_layout, index))
    }

    /// Use as an iterator to generate neuron values without needing to collect it all first
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (NM::NeuronDataInternal, NM::NeuronDataScratch, NM::NeuronDataVariance)> + DoubleEndedIterator + '_ {
        // NOTE: At this time rusts iter step trait is still nightly, hence this nonsense
        let range = 0 .. self.get_total_number_neurons_needed().quant_to_usize();
        range.map(move |i| self.generate_neuron_data_at(&CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i)))
    }

    /// Writes the generated neuron data into preallocated slices, one element per neuron
    pub fn write_into_slices(
        &self,
        internal: &mut [NM::NeuronDataInternal],
        scratch: &mut [NM::NeuronDataScratch],
        variance: &mut [NM::NeuronDataVariance],
    ) {
        let neuron_count = self.get_total_number_neurons_needed().quant_to_usize();
        assert_eq!(internal.len(), neuron_count);
        assert_eq!(scratch.len(), neuron_count);
        for (i, (internal_out, scratch_out)) in internal.iter_mut().zip(scratch.iter_mut()).enumerate() {
            let index = CorticalAreaNeuronLocalIndex::quant_from_usize_unchecked(i);
            (*internal_out, *scratch_out) = self.neurons.neuron_data_at::<FIQ>(&self.neuron_layout, &index);
        }
        self.cortical_variance.write_neuron_variance_into_slice::<FIQ>(&self.neuron_layout, variance);
    }

    // TODO parallel versions of the above
}

#[derive(Debug, thiserror::Error)]
pub enum CorticalAreaSpecError {
    #[error("Stored neuron data does not hold exactly one element per neuron of the layout!")]
    StoredNeuronCountMismatch,
}
