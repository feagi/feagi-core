use feagi_data::neurons::wrapped_types::NeuronCount;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_values::EngineCorticalIndex;

#[cfg(feature = "alloc")]
/// Change requests for the connectome
pub enum EngineConnectomeChangeRequest<FIQ: FeagiIndexQuantization> {
    AddCorticalArea { } , // writer enum
    RemoveCorticalArea(EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>),
    /// Edit Cortical Area Data in place
    EditPropertiesCorticalArea { },
    /// resize a cortical area, which would also require a mapping reset
    ResizeCorticalArea { },
    /// Increase space given to growing cortical areas without breaking mappings
    IncreaseCorticalBufferSizes { },
}

#[cfg(feature = "alloc")]
pub enum EngineConnectomeChangeResponse<FIQ: FeagiIndexQuantization> {
    CorticalAreaAdded{
        cortical_index: EngineCorticalIndex <FIQ::CorticalAreaIndexCountQuant>,
        new_engine_neuron_count: NeuronCount<FIQ::NeuronIndexQuant>
    },
    CorticalAreaRemoved {
        new_engine_neuron_count: NeuronCount<FIQ::NeuronIndexQuant>,
        new_engine_synapse_count: (), // TODO
    },
    CorticalAreaPropertiesEdited {
        new_engine_neuron_count: NeuronCount<FIQ::NeuronIndexQuant>,
        new_engine_synapse_count: (), // TODO // things like excitability may result in mappings being redone
    }
}


