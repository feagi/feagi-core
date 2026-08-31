use crate::cortical_area::data_structs::bit_batch_activations::BitBatchActivation;
use crate::cortical_area::data_structs::per_neuron_flags::PerNeuronFlags;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// TODO BitBatch sizing?

/// Additional data that is generated from an area's direct activity.
pub trait CorticalAreaContext<NPUIQ, BEIQ>
where
    NPUIQ: FeagiIndexQuantization,
    BEIQ: FeagiIndexQuantization,
{
    
}

/// Runs on BitBatch indexing, compiles neuron flag / firing data into bitpacked neuron activations
pub trait PerNeuronActivityBitpackingCorticalAreaContext<NPUIQ, BEIQ>: CorticalAreaContext<NPUIQ, BEIQ>
where
    NPUIQ: FeagiIndexQuantization,
    BEIQ: FeagiIndexQuantization,
{
    fn pack_neuron_flags_to_activations(flags: &[PerNeuronFlags; 32], write_to: &mut BitBatchActivation) {
        for i in 0usize..32 {
            // TODO this is wrong!
            let mask = (flags[i].is_neuron_firing_bit() as u32) << i;
            *write_to |= mask
        }
    }
}

