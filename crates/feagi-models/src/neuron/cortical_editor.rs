use crate::neuron::model_generated::cortical_layout::CorticalAreaLayoutNested;
use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;
use crate::neuron::properties::{CorticalAreaProperties, NeuronProperties};
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationGenomic};

// TODO rethink interface a bit more

/// Trait used to define both the root and model specific implementations for editing a cortical
/// area
pub trait NeuronModelCorticalEditor<NMQ, NMCD, NMND>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
{
    fn edit_cortical_area_inplace<FIQ: FeagiIndexQuantization>(
        self,
        cortical_data: &mut NMCD,
        neuron_data: &mut [NMND],
        neuron_properties: &mut [NeuronProperties],
    ) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties), ()>;
}

/// Root enum used to defining how a cortical area can be created. Enforces some universal methods.
/// By constraining model specific implementations to a generic sub enum, we can statically
/// create this easily!
pub enum RootNeuronModelCorticalEditor<NMQ, NMCD, NMND, SE>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
    SE: NeuronModelCorticalEditor<NMQ, NMCD, NMND>,
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

impl<NMQ, NMCD, NMND, SE> NeuronModelCorticalEditor<NMQ, NMCD, NMND>
    for RootNeuronModelCorticalEditor<NMQ, NMCD, NMND, SE>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
    SE: NeuronModelCorticalEditor<NMQ, NMCD, NMND>,
{


    fn edit_cortical_area_inplace<FIQ: FeagiIndexQuantization>(
        self,
        current_cortical_data: &mut NMCD,
        current_neuron_data: &mut [NMND],
        neuron_properties_out: &mut [NeuronProperties],
    ) -> Result<(CorticalAreaLayoutNested<FeagiIndexQuantizationGenomic>, CorticalAreaProperties), ()> { // TODO Error handling!
        match self {
            RootNeuronModelCorticalEditor::CompleteRawData {
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
            RootNeuronModelCorticalEditor::ModelSpecific(model) => {
                model.edit_cortical_area_inplace::<FIQ>(current_cortical_data, current_neuron_data, neuron_properties_out)
            }
        }
    }
}
