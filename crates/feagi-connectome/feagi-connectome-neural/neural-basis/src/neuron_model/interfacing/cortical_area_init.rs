use crate::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutOf};
use crate::neuron_model::model_data::{CorticalDataVarianceSpecification, NeuronModelUniversalCorticalProperties};
use crate::neuron_model::{NeuronModel};
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use feagi_basis::quantization::quantizable::DataValueQuantizationError;

/// Decides the data a neuron starts with, from where it is in its cortical area. Every way of
/// initializing neurons implements this, and each neuron model lists the ones it supports, which
/// together become its [`NeuronModel::NeuronInit`]. Implement it generically over the quantization
/// levels of the models it serves, converting to the level as needed
pub trait NeuronInitializer<NM: NeuronModel>: Clone + core::fmt::Debug + PartialEq + Send {
    /// Why this initializer cannot initialize the neurons of some layout
    type Error: core::error::Error;

    /// Checks that this initializer can give data to every neuron of `layout`. Called once when a
    /// [`CorticalAreaInit`] is created, so [`Self::neuron_data_at`] may assume it passed
    fn check_against_layout<FIQ: FeagiIndexQuantization>(&self, _layout: &NeuronLayoutOf<NM, FIQ>) -> Result<(), Self::Error> {
        Ok(())
    }

    /// What the neuron data should be at the given index
    fn neuron_data_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> (NM::NeuronDataInternal, NM::NeuronDataScratch);
}

/// Why the neuron initialization of model `NM` cannot initialize the neurons of some layout
pub type NeuronInitErrorOf<NM> = <<NM as NeuronModel>::NeuronInit as NeuronInitializer<NM>>::Error;

/// Every piece of data needed to spawn a new cortical area, at the given model quantization
#[derive(Debug, Clone, PartialEq)] // No need to serialize this, this is transient
pub struct CorticalAreaInit<FIQ, NM>
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
    /// How the neurons get their starting data. The neuron variance data is not part of this, as
    /// it is always derived from the cortical variance
    neurons: NM::NeuronInit,
}

impl<FIQ, NM> CorticalAreaInit<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel,
{
    /// `neurons` is any of the initializers the model supports. Returns an error if it cannot
    /// initialize the neurons of `neuron_layout`
    pub fn new(
        neuron_layout: NeuronLayoutOf<NM, FIQ>,
        universal_properties: NeuronModelUniversalCorticalProperties<NM>,
        cortical_properties: NM::CorticalDataProperties,
        cortical_internal: NM::CorticalDataInternal,
        cortical_scratch: NM::CorticalDataScratch,
        cortical_variance: NM::CorticalDataVariance,
        neurons: impl Into<NM::NeuronInit>,
    ) -> Result<Self, NeuronInitErrorOf<NM>> {
        let neurons = neurons.into();
        neurons.check_against_layout::<FIQ>(&neuron_layout)?;
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

    /// Get how the neurons get their starting data
    pub fn get_neuron_init(&self) -> &NM::NeuronInit {
        &self.neurons
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

/// Why a cortical area init could not be generated from the defaults of its cortical type
#[derive(Debug, thiserror::Error)]
pub enum CorticalAreaInitGenerationError<NeuronInitError: core::error::Error> {
    /// A default could not be converted to the quantization of the model
    #[error(transparent)]
    Quantization(#[from] DataValueQuantizationError),
    /// The neuron initializer cannot initialize the neurons of the layout
    #[error(transparent)]
    NeuronInit(NeuronInitError),
}
