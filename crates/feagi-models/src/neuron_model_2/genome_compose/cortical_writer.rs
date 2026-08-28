use crate::neuron_model::cortical_area::cortical_area_properties::CorticalAreaProperties;
use crate::neuron_model::cortical_area::cortical_data::NeuronModelCorticalData;
use crate::neuron_model::neuron::neuron_data::NeuronModelNeuronData;
use crate::neuron_model::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::neuron_model::neuron::neuron_properties::NeuronProperties;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::neuron_model::NeuronModel;

/// Trait for writing the data of newly created cortical areas, used both by the root and model
/// specific enums
pub trait NeuronModelCorticalWriter<FIQ, NMQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
    NM: NeuronModel<FIQ, NMQ>,
{
    /// Number of neurons needed
    fn number_neurons_needed(&self) -> Result<FIQ::NeuronIndexQuant, ()>; // TODO error!

    /// Handles writing the per neuron data and creating the properties, overwriting existing data
    /// ALL MEMBERS are to be overwritten!
    fn write_to_cortical_area(
        self,
        cortical_data: &mut NM::CorticalData,

        neuron_data: &mut [NM::NeuronData],
        neuron_properties: &mut [NeuronProperties], // TODO this is messy, we should find a way to get the 'impl iterator' thing to work
    ) -> Result<(CorticalAreaProperties<NMQ>,   ), ()>;
}

/// Root enum used to defining how a cortical area can be created. Enforces some universal methods.
/// By constraining model specific implementations to a generic sub enum, we can statically
/// create this easily!
pub enum RootNeuronModelCorticalWriter<NMQ, NMCD, NMND, SE>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
    SE: NeuronModelCorticalWriter<NMQ, NMCD, NMND>,
{
    /// In the case that we have a full set of data (IE from connectome loading), load the full
    /// data directly! Useful for overwriting / creating a new area
    CompleteRawData {
        _p: core::marker::PhantomData<NMQ>,
        cortical_data: NMCD,
        cortical_properties: CorticalAreaProperties<NMQ>,
        neuron_data: Vec<NMND>, // len should match what layout defines and properties
        neuron_properties: Vec<NeuronProperties>,
        neuron_layout: CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>,
    },
    ModelSpecific(SE),
}

impl<NMQ, NMCD, NMND, SE> NeuronModelCorticalWriter<NMQ, NMCD, NMND> for RootNeuronModelCorticalWriter<NMQ, NMCD, NMND, SE>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
    SE: NeuronModelCorticalWriter<NMQ, NMCD, NMND>,
{
    fn number_neurons_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::NeuronIndexQuant, ()> {
        match self {
            RootNeuronModelCorticalWriter::CompleteRawData { neuron_layout, .. } => {
                let u = neuron_layout.get_total_number_neurons();
                let r: FIQ::NeuronIndexQuant = u.try_to_quantization().unwrap(); // TODO error handling!
                Ok(r)
            }
            RootNeuronModelCorticalWriter::ModelSpecific(SE) => SE.number_neurons_needed::<FIQ>(),
        }
    }

    fn write_to_cortical_area<FIQ: FeagiIndexQuantization>(
        self,
        current_cortical_data: &mut NMCD,
        current_neuron_data: &mut [NMND],
        neuron_properties_out: &mut [NeuronProperties],
    ) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties<NMQ>), ()> {
        // TODO Error handling!
        match self {
            RootNeuronModelCorticalWriter::CompleteRawData {
                _p,
                cortical_data,
                cortical_properties,
                neuron_data,
                neuron_properties,
                neuron_layout,
            } => {
                *current_cortical_data = cortical_data;
                current_neuron_data.copy_from_slice(neuron_data.as_slice());
                for (dst, src) in neuron_properties_out.iter_mut().zip(neuron_properties.into_iter()) {
                    *dst = src;
                }
                Ok((neuron_layout, cortical_properties))
            }
            RootNeuronModelCorticalWriter::ModelSpecific(model) => {
                model.write_to_cortical_area::<FIQ>(current_cortical_data, current_neuron_data, neuron_properties_out)
            }
        }
    }
}
