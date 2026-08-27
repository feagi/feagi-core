use crate::cortical_area::data_structs::bit_batch_activations::BitBatchActivation;
use crate::cortical_area::data_structs::per_neuron_flags::PerNeuronFlags;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

// TODO BitBatch sizing?

/// Additional data that is generated from an area's direct activity.
pub trait CorticalAreaContext<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: NeuronProcessingUnitIndexQuantization,
{
    
}

/// Runs on BitBatch indexing, compiles neuron flag / firing data into bitpacked neuron activations
pub trait PerNeuronActivityBitpackingCorticalAreaContext<NPUIQ, BEIQ>: CorticalAreaContext<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: NeuronProcessingUnitIndexQuantization,
{
    fn pack_neuron_flags_to_activations(flags: &[PerNeuronFlags; 32], write_to: &mut BitBatchActivation) {
        for i in 0usize..32 {
            // TODO this is wrong!
            let mask = (flags[i].is_neuron_firing_bit() as u32) << i;
            *write_to |= mask
        }
    }
}

