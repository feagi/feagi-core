//! Covers the Uniform synapse model's genome-to-engine writer path: how a mapping request is
//! lifted into a writer, how that writer sizes its synapse region, and what it stamps into the
//! cortical mapping entry once the engine has carved that region out.

use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantizationGenomic, FeagiIndexQuantizationStandard};
use feagi_data::values::quantizable::WrappedQuantizedDecimal;
use feagi_models::cortical_mapping_entry::components::doublet::doublet_iterator_type::DoubletIteratorDimensionalTypeGenomic;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer::SynapseModelCorticalWriter;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::UniformWriter;
use feagi_models::cortical_mapping_entry::synapse::synapse_data::EmptyPerSynapseData;
use feagi_models::cortical_mapping_entry::synapse::synapse_properties::SynapseProperties;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::composers::UniformSynapseWriter;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::data::{
    UniformSynapseModelCorticalMappingEntryData, UniformSynapseMultiplier,
};
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelStandardQuant;

type StandardEntryData = UniformSynapseModelCorticalMappingEntryData<UniformSynapseModelStandardQuant>;
type StandardWriter = UniformSynapseWriter<UniformSynapseModelStandardQuant>;

/// A OneToOne mapping request carrying the given weight, delay and polarity.
fn genomic_request(weight: f32, propagation_delay: u16, is_inhibitory: bool) -> UniformWriter {
    UniformWriter::Standard {
        doublet: DoubletIteratorDimensionalTypeGenomic::OneToOne {
            source: NeuronVoxelCoordinate::new_from_usizes_unchecked(0, 0, 0),
            destination: NeuronVoxelCoordinate::new_from_usizes_unchecked(0, 0, 0),
        },
        uniform_weight: UniformSynapseMultiplier::new(weight),
        propagation_delay,
        is_inhibitory,
    }
}

#[test]
fn genomic_request_lifts_into_writer_preserving_every_field() {
    let request = genomic_request(0.75, 3, true);

    let writer = StandardWriter::from_genomic_writer(&request, 4);

    let UniformSynapseWriter::Default {
        number_synapses,
        uniform_weight,
        propagation_delay,
        is_inhibitory,
        ..
    } = writer;
    assert_eq!(number_synapses, 4);
    assert_eq!(uniform_weight.deref(), 0.75);
    assert_eq!(propagation_delay, 3);
    assert!(is_inhibitory);
}

#[test]
fn synapse_count_is_reported_in_the_requested_quantization() {
    let writer = StandardWriter::from_genomic_writer(&genomic_request(1.0, 0, false), 12);

    let standard: u32 = writer.number_synapses_needed::<FeagiIndexQuantizationStandard>().unwrap();
    let genomic: u64 = writer.number_synapses_needed::<FeagiIndexQuantizationGenomic>().unwrap();

    assert_eq!(standard, 12);
    assert_eq!(genomic, 12);
}

#[test]
fn synapse_count_that_overflows_the_quantization_is_rejected() {
    // Standard indexes synapses with a u32, so a count past that must fail rather than wrap and
    // leave the engine sizing a region far smaller than the mapping needs.
    let writer = StandardWriter::from_genomic_writer(&genomic_request(1.0, 0, false), usize::MAX);

    assert!(writer.number_synapses_needed::<FeagiIndexQuantizationStandard>().is_err());
}

#[test]
fn writing_the_region_stamps_weight_properties_and_entry_metadata() {
    let writer = StandardWriter::from_genomic_writer(&genomic_request(0.5, 7, false), 3);
    let mut entry_data = StandardEntryData::default();
    let mut synapse_data = [EmptyPerSynapseData; 3];
    // Seeded with probes on, so a writer that silently skipped them would be caught.
    let mut synapse_properties = [SynapseProperties {
        probe_force_disabled: true,
        probe_force_firing: true,
    }; 3];

    let entry_properties = writer
        .write_to_synapse_region::<FeagiIndexQuantizationStandard>(&mut entry_data, &mut synapse_data, &mut synapse_properties)
        .unwrap();

    assert_eq!(entry_data.post_synaptic_multiplier.deref(), 0.5);
    assert_eq!(entry_properties.propagation_delay, 7);
    assert!(!entry_properties.is_inhibitory);
    for properties in synapse_properties.iter() {
        assert!(!properties.probe_force_disabled);
        assert!(!properties.probe_force_firing);
    }
}

#[test]
fn writing_a_region_sized_against_a_different_count_is_rejected() {
    let writer = StandardWriter::from_genomic_writer(&genomic_request(0.5, 0, false), 3);
    let mut entry_data = StandardEntryData::default();
    let mut synapse_data = [EmptyPerSynapseData; 2];
    let mut synapse_properties = [SynapseProperties::default(); 2];

    let result = writer.write_to_synapse_region::<FeagiIndexQuantizationStandard>(&mut entry_data, &mut synapse_data, &mut synapse_properties);

    assert!(result.is_err());
    // A rejected write must not have partially applied the weight.
    assert_eq!(
        entry_data.post_synaptic_multiplier.deref(),
        StandardEntryData::default().post_synaptic_multiplier.deref()
    );
}

#[test]
fn inhibitory_polarity_reaches_the_entry_properties() {
    let writer = StandardWriter::from_genomic_writer(&genomic_request(1.0, 0, true), 1);
    let mut entry_data = StandardEntryData::default();
    let mut synapse_data = [EmptyPerSynapseData; 1];
    let mut synapse_properties = [SynapseProperties::default(); 1];

    let entry_properties = writer
        .write_to_synapse_region::<FeagiIndexQuantizationStandard>(&mut entry_data, &mut synapse_data, &mut synapse_properties)
        .unwrap();

    assert!(entry_properties.is_inhibitory);
}
