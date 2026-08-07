use rayon::prelude::*;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait, WrappedQuantizedDecimal, WrappedQuantizedIndexCount};
use feagi_models::cortical_mapping_entry::synapse::synapse_model::SynapseModel;
use feagi_models::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::generated_enums::SynapseModelTypeAndQuantizationPacked;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::model::UniformSynapseModel;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelStandardQuant;
use feagi_models::wrapped_index_collections::{MappingEntryModelIndex, NeuronMPIndex, SynapseEngineIndex};
use crate::engines::rayon::data::RayonEngineData;

pub(crate) fn process_synapses<FIQ: FeagiIndexQuantization>(data: &RayonEngineData<FIQ>)
{
    let burst_index = data.burst_index;

    // We access `data` through a shared `&` and mutate disjoint slots via the
    // `get_mut_par` accessors
    unsafe {

        // no clustering with synapses, since the way they access data *may* be sporadic.

        data.cortical_mapping_entry_indexes
            .as_slice()
            .par_iter()
            .enumerate()
            .for_each(|(synapse_index, &mapping_entry_index)| {

                let synapse_engine_index: SynapseEngineIndex<FIQ::SynapseIndexCountQuant> = SynapseEngineIndex::quant_from_usize(synapse_index);
                
                let mapping_entry_properties = data.cortical_mapping_entry_properties.get_par(mapping_entry_index);
                
                if mapping_entry_properties.flags.get_is_mapping_entry_disabled() {
                    return; // mapping entry is disabled, stop here
                }
                
                let mapping_entry_lookup_table = data.cortical_mapping_index_lookup_table.get_par(mapping_entry_index);
                
                let synapse_neuron_indexes = data.synapse_source_destination_mp_neuron_indexes.get_par(synapse_engine_index);

                let mapping_entry_lookup_table = data.cortical_mapping_index_lookup_table.get_par(mapping_entry_index);
                let synapse_neuron_indexes = data.synapse_source_destination_mp_neuron_indexes.get_par(synapse_engine_index);




        })


    }




}


#[inline(always)]
unsafe fn synapse_dynamics<FIQ: FeagiIndexQuantization> (
    data: &RayonEngineData<FIQ>,
    source_neuron_mp_quant: DecimalQuantizationLevel,
    source_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
    destination_neuron_mp_quant: DecimalQuantizationLevel,
    destination_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
    synapse_model_and_quant: SynapseModelTypeAndQuantizationPacked,
    mapping_entry_index: MappingEntryModelIndex<FIQ::CorticalMappingEntryIndexCountQuant>,
)
{
    
    match synapse_model_and_quant 
    { 
        SynapseModelTypeAndQuantizationPacked::Uniform_Standard => {
            let input = source_to_junction::<FIQ, <UniformSynapseModelStandardQuant as SynapseModelQuantization>::JunctionPotentialQuant>(
                data,
                &source_neuron_mp_quant,
                source_neuron_mp_index,
            );
            
        }

        _ => {}
    }
}
unsafe fn source_to_junction<FIQ: FeagiIndexQuantization, JunctionQuant: QuantizedDecimalTrait>(data: &RayonEngineData<FIQ>, source_quant: &DecimalQuantizationLevel, source_index: NeuronMPIndex<FIQ::NeuronIndexQuant>) -> NeuronMembranePotential<JunctionQuant> {
    match source_quant {
        DecimalQuantizationLevel::StorageF8 => {
            let potential = data.neuron_membrane_data.mp_storage_f8.get_par(source_index).deref();
            NeuronMembranePotential::from_quantization(potential)
        }
        DecimalQuantizationLevel::F16 => {
            let potential = data.neuron_membrane_data.mp_f16.get_par(source_index).deref();
            NeuronMembranePotential::from_quantization(potential)
        }
        DecimalQuantizationLevel::BF16 => {
            let potential = data.neuron_membrane_data.mp_bf16.get_par(source_index).deref();
            NeuronMembranePotential::from_quantization(potential)
        }
        DecimalQuantizationLevel::F32 => {
            let potential = data.neuron_membrane_data.mp_f32.get_par(source_index).deref();
            NeuronMembranePotential::from_quantization(potential)
        }
        DecimalQuantizationLevel::F64 => {
            let potential = data.neuron_membrane_data.mp_f64.get_par(source_index).deref();
            NeuronMembranePotential::from_quantization(potential)
        }
    }
}

unsafe fn junction_to_destination<FIQ: FeagiIndexQuantization, JunctionQuant: QuantizedDecimalTrait>(data: &RayonEngineData<FIQ>, value: NeuronMembranePotential<JunctionQuant>, destination_quant: &DecimalQuantizationLevel, destination_index: FIQ::NeuronIndexQuant, fclc_index: u8) {
    match destination_quant {
        DecimalQuantizationLevel::StorageF8 => {}
        DecimalQuantizationLevel::F16 => {}
        DecimalQuantizationLevel::BF16 => {}
        DecimalQuantizationLevel::F32 => {}
        DecimalQuantizationLevel::F64 => {}
    }
}

