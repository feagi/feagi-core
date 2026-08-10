use crate::engines::rayon::data::RayonEngineData;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedDecimalTrait, WrappedQuantizedDecimal, WrappedQuantizedIndexCount};
use feagi_models::cortical_area::components::neuron_history::implementations::none::NeuronModelNoNeuronHistory;
use feagi_models::cortical_mapping_entry::synapse::synapse_model::SynapseModel;
use feagi_models::cortical_mapping_entry::synapse::synapse_model_quantization::SynapseModelQuantization;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::generated_enums::SynapseModelTypeAndQuantizationPacked;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::model::UniformSynapseModel;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelStandardQuant;
use feagi_models::wrapped_index_collections::{MappingEntryModelIndex, NeuronMPIndex, SynapseEngineIndex};

pub(crate) fn process_synapses<FIQ: FeagiIndexQuantization>(data: &RayonEngineData<FIQ>) {
    // Walked serially: synapses converge, so several of them accumulate into the same destination
    // slot, and scattering those writes across threads would alias what `get_mut_par` requires to
    // be disjoint. Parallelising this needs a destination indexed reverse mapping so each thread
    // owns whole destinations rather than whole synapses.
    unsafe {
        for (synapse_index, &mapping_entry_index) in data.cortical_mapping_entry_indexes.as_slice().iter().enumerate() {
            let synapse_engine_index: SynapseEngineIndex<FIQ::SynapseIndexCountQuant> = SynapseEngineIndex::quant_from_usize(synapse_index);

            let mapping_entry_properties = data.cortical_mapping_entry_properties.get_par(mapping_entry_index);
            if mapping_entry_properties.flags.get_is_mapping_entry_disabled() {
                continue; // mapping entry is disabled, stop here
            }

            let mapping_entry_lookup_table = data.cortical_mapping_index_lookup_table.get_par(mapping_entry_index);
            let (source_mp_index, destination_mp_index) = *data.synapse_source_destination_mp_neuron_indexes.get_par(synapse_engine_index);

            synapse_dynamics::<FIQ>(
                data,
                mapping_entry_properties.source_destination_mp_quants.source_mp(),
                source_mp_index,
                mapping_entry_properties.source_destination_mp_quants.destination_mp(),
                destination_mp_index,
                mapping_entry_properties.model_and_quant,
                mapping_entry_lookup_table.mapping_entry_model_index,
                mapping_entry_properties.flags.polarity_sign(),
            );
        }
    }
}

#[inline(always)]
unsafe fn synapse_dynamics<FIQ: FeagiIndexQuantization>(
    data: &RayonEngineData<FIQ>,
    source_neuron_mp_quant: DecimalQuantizationLevel,
    source_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
    destination_neuron_mp_quant: DecimalQuantizationLevel,
    destination_neuron_mp_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
    synapse_model_and_quant: SynapseModelTypeAndQuantizationPacked,
    mapping_entry_index: MappingEntryModelIndex<FIQ::CorticalMappingEntryIndexCountQuant>,
    polarity_sign: f32,
) {
    match synapse_model_and_quant {
        SynapseModelTypeAndQuantizationPacked::Uniform_Standard => {
            let input = source_to_junction::<FIQ, <UniformSynapseModelStandardQuant as SynapseModelQuantization>::JunctionPotentialQuant>(
                data,
                &source_neuron_mp_quant,
                source_neuron_mp_index,
            );

            let mapping_entry_data = data
                .synapse_model_data
                .uniform
                .quantization_standard
                .mapping_entry_data
                .get_par(mapping_entry_index);

            // The Uniform model keeps no fire history, so the history argument is inert here.
            let output = UniformSynapseModel::<FIQ, UniformSynapseModelStandardQuant>::synapse_process_incoming_signal(
                &input,
                mapping_entry_data,
                &NeuronModelNoNeuronHistory(),
            );

            junction_to_destination::<FIQ, _>(data, output, &destination_neuron_mp_quant, destination_neuron_mp_index, polarity_sign);
        }
    }
}

unsafe fn source_to_junction<FIQ: FeagiIndexQuantization, JunctionQuant: QuantizedDecimalTrait>(
    data: &RayonEngineData<FIQ>,
    source_quant: &DecimalQuantizationLevel,
    source_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
) -> NeuronMembranePotential<JunctionQuant> {
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

unsafe fn junction_to_destination<FIQ: FeagiIndexQuantization, JunctionQuant: QuantizedDecimalTrait>(
    data: &RayonEngineData<FIQ>,
    value: NeuronMembranePotential<JunctionQuant>,
    destination_quant: &DecimalQuantizationLevel,
    destination_index: NeuronMPIndex<FIQ::NeuronIndexQuant>,
    polarity_sign: f32,
) {
    match destination_quant {
        DecimalQuantizationLevel::F32 => {
            let arriving: NeuronMembranePotential<f32> = NeuronMembranePotential::from_quantization(value.deref());
            let destination = data.neuron_membrane_data.fcl_f32.get_mut_par(destination_index);
            // Polarity is applied at this single point that every synapse model funnels through, so
            // no model can forget it, and as a multiply rather than a branch so wide SIMD and GPU
            // backends stay divergence free.
            *destination = NeuronMembranePotential::new(destination.deref() + (arriving.deref() * polarity_sign));
        }
        // `add_cortical_area` only allocates F32 membrane potential storage, so no other precision
        // has a backing vector to accumulate into. Reaching one means the allocator and the kernel
        // disagree, which must be loud rather than a silently dropped potential.
        DecimalQuantizationLevel::StorageF8 | DecimalQuantizationLevel::F16 | DecimalQuantizationLevel::BF16 | DecimalQuantizationLevel::F64 => {
            todo!("membrane potential storage for this precision is not allocated yet")
        }
    }
}
