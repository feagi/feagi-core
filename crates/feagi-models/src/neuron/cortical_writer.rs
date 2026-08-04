use crate::neuron::model_generated::cortical_layout::CorticalAreaLayoutNested;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::neuron::properties::{CorticalAreaProperties, NeuronProperties};
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;

pub trait NeuronModelCorticalWriter<NMQ, NMCD, NMND>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
{
    fn number_neurons_needed<FIQ: FeagiIndexQuantization>(&self) -> Result<FIQ::NeuronIndexQuant, ()>; // TODO error!

    fn write_to_cortical_area<FIQ: FeagiIndexQuantization>(
        self,
        cortical_data: &mut NMCD,
        neuron_data: &mut [NMND],
    ) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties, impl Iterator<Item = NeuronProperties>), ()>;
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
        cortical_properties: CorticalAreaProperties,
        neuron_data: Vec<NMND>, // len should match what layout defines and properties
        neuron_properties: Vec<NeuronProperties>,
        neuron_layout: CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>,
    },
    ModelSpecific(SE),
}

impl<NMQ, NMCD, NMND, SE> NeuronModelCorticalWriter<NMQ, NMCD, NMND>
    for RootNeuronModelCorticalWriter<NMQ, NMCD, NMND, SE>
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
    ) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties, impl Iterator<Item = NeuronProperties>), ()> { // TODO Error handling!
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
                Ok((neuron_layout, cortical_properties, neuron_properties.iter().cloned())) // TODO cloning nu bueno
            }
            RootNeuronModelCorticalWriter::ModelSpecific(model) => {
                model.write_to_cortical_area::<FIQ>(current_cortical_data, current_neuron_data)
            }
        }
    }
}
