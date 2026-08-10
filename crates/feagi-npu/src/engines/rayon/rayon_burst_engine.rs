use crate::engines::rayon::data::neuron::neuron_sub_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};
use crate::engines::rayon::data::synapse::synapse_sub_data::{
    CorticalMappingEntryIndexLookupTable, CorticalMappingEntryProperties, SynapseMappingMPQuants,
};
use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::kernels_neurons;
use crate::engines::rayon::kernels_synapses;
use crate::engines_common::EditableEngine::EditableEngine;
use crate::flags::cortical_mapping_entry_runtime_flags::CorticalMappingEntryRuntimeFlags;
use crate::flags::cortical_runtime_flags::CorticalRuntimeFlags;
use crate::flags::neuron_runtime_flags::NeuronRuntimeFlags;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use feagi_data::values::quantizable::{DecimalQuantizationLevel, QuantizedUnsignedIntegerTrait};
use feagi_models::cortical_area::components::cortical_area_layout::enums::CorticalAreaLayoutNested;
use feagi_models::cortical_area::components::cortical_area_layout::implementations::dimensional::CorticalAreaLayoutDimensional;
use feagi_models::cortical_area::components::neuron_history::implementations::full::NeuronModelFullNeuronHistory;
use feagi_models::cortical_area::components::neuron_history::neuron_history::NeuronModelHistory;
use feagi_models::cortical_area::genome_compose::cortical_writer::NeuronModelCorticalWriter;
use feagi_models::cortical_area::neuron::neuron_model::NeuronModel;
use feagi_models::cortical_area::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_models::cortical_area::neuron::neuron_model_quantization_level::NeuronModelQuantizationLevel;
use feagi_models::cortical_area::neuron::neuron_properties::NeuronProperties;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::cortical_area::neuron_model_implementations::generated_enums::{
    NeuronModelTypeAndQuantizationNested, NeuronModelTypeAndQuantizationPacked,
};
use feagi_models::cortical_mapping_entry::components::doublet::doublet_iterator::DoubletIterator;
use feagi_models::cortical_mapping_entry::components::doublet::doublet_iterator_type::DoubletIteratorDimensionalTypeGenomic;
use feagi_models::cortical_mapping_entry::components::doublet::implementations::all_to_one::DoubletIteratorAllToOne;
use feagi_models::cortical_mapping_entry::components::doublet::implementations::one_to_all::DoubletIteratorOneToAll;
use feagi_models::cortical_mapping_entry::components::doublet::implementations::one_to_one::DoubletIteratorOneToOne;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer::SynapseModelCorticalWriter;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::UniformWriter;
use feagi_models::cortical_mapping_entry::synapse::synapse_data::EmptyPerSynapseData;
use feagi_models::cortical_mapping_entry::synapse::synapse_model::SynapseModel;
use feagi_models::cortical_mapping_entry::synapse::synapse_properties::SynapseProperties;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::generated_enums::SynapseModelTypeAndQuantizationPacked;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::composers::UniformSynapseWriter;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::data::UniformSynapseModelCorticalMappingEntryData;
use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::quantizations::UniformSynapseModelStandardQuant;
use feagi_models::wrapped_index_collections::{
    CorticalEngineIndex, CorticalLayoutIndex, CorticalModelIndex, MappingEntryEngineIndex, NeuronEngineIndex, NeuronEngineIndexedVector,
    NeuronHistoryIndex, NeuronMPIndex, NeuronModelIndex,
};
use feagi_models::wrapped_indexes::BurstIndex;

pub struct RayonBurstEngine<FIQ: FeagiIndexQuantization> {
    data: RayonEngineData<FIQ>,
    // dyn stuff
    latest_cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    latest_mapping_entry_index: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>,
}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            latest_cortical_index: CorticalEngineIndex::QUANT_ZERO,
            latest_mapping_entry_index: MappingEntryEngineIndex::QUANT_ZERO,
        }
    }

    pub fn set_sensor_data(&mut self, data: ()) {
        todo!()
    }

    pub fn get_motor_data(&self) {
        todo!()
    }

    pub fn get_visualization_data(&self) -> &NeuronEngineIndexedVector<FIQ::NeuronIndexQuant, NeuronRuntimeFlags> {
        &self.data.neuron_runtime_flags
    }

    /// Read access to the engine's storage for sibling modules that project it into other shapes,
    /// such as the visualization snapshot.
    pub(crate) fn engine_data(&self) -> &RayonEngineData<FIQ> {
        &self.data
    }

    pub fn execute_single_burst(&mut self) {
        kernels_neurons::process_neurons(&self.data);
        // Firing state is settled by neuron dynamics and is what visualization reads, so it is
        // packed before synapses run and start consuming it.
        kernels_neurons::pack_firing_bitmap(&self.data);
        kernels_synapses::process_synapses(&self.data);
        self.data.burst_index += BurstIndex::QUANT_ONE;
    }

    /// The dimensional layout of a cortical area, resolved through its lookup table.
    fn dimensional_layout_of(&self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) -> CorticalAreaLayoutDimensional<FIQ> {
        let lookup = self
            .data
            .cortical_index_lookup_table
            .get(cortical_index)
            .expect("mapping entry references a cortical area the engine does not host"); // TODO ERROR HANDLING
        self.data
            .cortical_layout_dimensional_data
            .get(lookup.cortical_layout_index)
            .expect("cortical area has no dimensional layout allocated") // TODO ERROR HANDLING
    }

    /// The membrane potential precision a cortical area's neurons are stored at. Read back from the
    /// neuron model quantization the area was allocated with rather than assumed, so it stays
    /// correct once more than one precision is allocated.
    fn membrane_potential_quantization_of(&self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) -> DecimalQuantizationLevel {
        let (model_and_quant, _) = self
            .data
            .cortical_neuron_model_and_quant_and_neuron_properties
            .get(cortical_index)
            .expect("mapping entry references a cortical area the engine does not host"); // TODO ERROR HANDLING
        match model_and_quant.to_unpacked() {
            NeuronModelTypeAndQuantizationNested::FeagiAdvanced(level) => level.get_membrane_potential_level(),
        }
    }

    /// First membrane potential index of a cortical area, used to lift area local neuron indexes
    /// into the engine wide indexing the synapse kernel addresses.
    fn first_membrane_potential_index_of(
        &self,
        cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    ) -> NeuronMPIndex<FIQ::NeuronIndexQuant> {
        self.data
            .cortical_neuron_index_lookup_table
            .get(cortical_index)
            .expect("mapping entry references a cortical area the engine does not host") // TODO ERROR HANDLING
            .cortical_first_neuron_mp_index
    }

    /// Appends one Uniform mapping entry along with the synapses its doublet resolves to.
    ///
    /// Generic over the resolved doublet so every pairing kind shares a single allocation path. The
    /// caller builds the concrete iterator once both cortical layouts are known.
    fn append_uniform_mapping_entry<D>(
        &mut self,
        doublet_iterator: D,
        writer: &UniformWriter,
        source_cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
        destination_cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    ) -> MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>
    where
        D: DoubletIterator<FIQ, CorticalAreaLayoutDimensional<FIQ>, CorticalAreaLayoutDimensional<FIQ>>,
    {
        let number_synapses_quant = doublet_iterator.get_number_of_synapses();
        let number_synapses = number_synapses_quant.quant_to_usize();

        let source_first_mp_index = self.first_membrane_potential_index_of(source_cortical_index);
        let destination_first_mp_index = self.first_membrane_potential_index_of(destination_cortical_index);
        let source_destination_mp_quants = SynapseMappingMPQuants::new(
            self.membrane_potential_quantization_of(source_cortical_index),
            self.membrane_potential_quantization_of(destination_cortical_index),
        );

        let mapping_entry_index = self.latest_mapping_entry_index;
        // Same append-only discipline as `add_cortical_area`: captured before any append so it
        // describes where this entry's model data starts rather than where it ends.
        let mapping_entry_model_index = self.data.synapse_model_data.uniform.quantization_standard.mapping_entry_data.len();

        // Appended first so the writer can fill the entry's model data in place.
        let mapping_entry_data = self
            .data
            .synapse_model_data
            .uniform
            .quantization_standard
            .mapping_entry_data
            .append_single_mut(Default::default());

        let synapse_writer = UniformSynapseWriter::<UniformSynapseModelStandardQuant>::from_genomic_writer(writer, number_synapses);

        // The Uniform model keeps nothing per synapse, but the writer contract still takes a slice.
        // `EmptyPerSynapseData` is zero sized, so this buffer occupies no memory.
        let mut synapse_data = vec![EmptyPerSynapseData; number_synapses];
        let mut synapse_properties = vec![SynapseProperties::default(); number_synapses];

        let entry_properties = synapse_writer
            .write_to_synapse_region::<FIQ>(mapping_entry_data, synapse_data.as_mut_slice(), synapse_properties.as_mut_slice())
            .expect("uniform synapse writer must fill the region the engine sized from its doublet"); // TODO ERROR HANDLING

        // Per mapping entry rows, both indexed by `MappingEntryEngineIndex`.
        self.data
            .cortical_mapping_entry_properties
            .append_single_mut(CorticalMappingEntryProperties {
                flags: CorticalMappingEntryRuntimeFlags::new(false, entry_properties.is_inhibitory),
                model_and_quant: SynapseModelTypeAndQuantizationPacked::Uniform_Standard,
                source_destination_mp_quants,
                delay: entry_properties.propagation_delay,
            });
        self.data
            .cortical_mapping_index_lookup_table
            .append_single_mut(CorticalMappingEntryIndexLookupTable { mapping_entry_model_index });

        // Per synapse rows. `cortical_mapping_entry_indexes` is what the synapse kernel iterates,
        // so it has to grow by exactly this entry's synapse count for the mapping to run at all.
        self.data
            .cortical_mapping_entry_indexes
            .extend_mut(number_synapses_quant.into(), mapping_entry_index);
        let synapse_neuron_indexes = self
            .data
            .synapse_source_destination_mp_neuron_indexes
            .extend_mut(number_synapses_quant.into(), (NeuronMPIndex::QUANT_ZERO, NeuronMPIndex::QUANT_ZERO));

        // The doublet yields area local neuron indexes while the kernel addresses membrane
        // potentials engine wide, so each end is offset by its area's first membrane potential index.
        for (slot, (source_local, destination_local)) in synapse_neuron_indexes.iter_mut().zip(doublet_iterator) {
            *slot = (
                NeuronMPIndex::new(source_first_mp_index.deref() + source_local.deref()),
                NeuronMPIndex::new(destination_first_mp_index.deref() + destination_local.deref()),
            );
        }

        debug_assert_eq!(
            self.data.cortical_mapping_entry_indexes.len().deref(),
            self.data.synapse_source_destination_mp_neuron_indexes.len().deref(),
            "synapse indexed vectors must grow in lockstep"
        );

        self.latest_mapping_entry_index += MappingEntryEngineIndex::QUANT_ONE;
        mapping_entry_index
    }
}

/// Doublet coordinates arrive at genomic quantization; the engine resolves them at its own. A
/// coordinate that does not fit describes a genome the engine cannot host.
fn to_engine_voxel_coordinate<FIQ: FeagiIndexQuantization>(
    coordinate: NeuronVoxelCoordinate<<FeagiIndexQuantizationStandard as FeagiIndexQuantization>::NeuronIndexQuant>,
) -> NeuronVoxelCoordinate<FIQ::NeuronIndexQuant> {
    coordinate
        .try_to_quantization::<FIQ::NeuronIndexQuant>()
        .expect("doublet voxel coordinate must fit the engine's index quantization")
    // TODO ERROR HANDLING
}

impl<FIQ: FeagiIndexQuantization> EditableEngine<FIQ> for RayonBurstEngine<FIQ> {
    fn add_cortical_area<NM>(
        &mut self,
        neuron_data_writer: impl NeuronModelCorticalWriter<FeagiAdvancedModelStandardQuant, NM::CorticalData, NM::NeuronData>,
    ) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>
    where
        NM: NeuronModel<
            FIQ,
            FeagiAdvancedModelStandardQuant,
            CorticalData = FeagiAdvancedModelCorticalData<FeagiAdvancedModelStandardQuant>,
            NeuronData = FeagiAdvancedModelNeuronData<FeagiAdvancedModelStandardQuant>,
        >,
    {
        let number_neurons: FIQ::NeuronIndexQuant = neuron_data_writer.number_neurons_needed::<FIQ>().unwrap(); // TODO ERROR CHECKING
        let mut neuron_properties = vec![NeuronProperties::default(); number_neurons.quant_to_usize()];

        // Every per-neuron vector is appended to in lockstep, so the first index of this area is
        // wherever each vector currently ends. Captured before any append so they describe this
        // area's start rather than its end.
        let first_neuron_engine_index = NeuronEngineIndex::new(self.data.cortical_engine_indexes.len().deref());
        let first_neuron_mp_index = NeuronMPIndex::new(self.data.neuron_membrane_data.mp_f32.len().deref());
        let first_neuron_model_index =
            NeuronModelIndex::new(self.data.neuron_model_data.feagi_advanced.quantization_standard.neuron_data.len().deref());
        let first_neuron_history_index = NeuronHistoryIndex::new(self.data.neuron_history_data.len().deref());
        let cortical_model_index = CorticalModelIndex::new(
            self.data
                .neuron_model_data
                .feagi_advanced
                .quantization_standard
                .cortical_data
                .len()
                .deref(),
        );
        let cortical_layout_index = CorticalLayoutIndex::new(self.data.cortical_layout_dimensional_data.len().deref());

        // TODO for now fixate ona specific quantization
        let cortical_data = self
            .data
            .neuron_model_data
            .feagi_advanced
            .quantization_standard
            .cortical_data
            .append_single_mut(Default::default());
        let neuron_data = self
            .data
            .neuron_model_data
            .feagi_advanced
            .quantization_standard
            .neuron_data
            .extend_mut(number_neurons.into(), Default::default());

        let (layout, cortical_properties) = neuron_data_writer
            .write_to_cortical_area::<FIQ>(cortical_data, neuron_data, neuron_properties.as_mut_slice())
            .unwrap(); // TODO ERROR HANDLING

        let cortical_index = self.latest_cortical_index;

        // Layout. Only dimensional areas are allocated for now, matching the single writer the
        // engine accepts; a formless area would need its own indexed vector.
        match layout {
            CorticalAreaLayoutNested::Dimensional(dimensional) => {
                // Writers describe layouts at genomic quantization; the engine stores them at its
                // own. Dimensions that do not fit are a genome the engine cannot host.
                let dimensions = dimensional
                    .dimensions
                    .try_to_quantization::<FIQ::NeuronIndexQuant>()
                    .expect("cortical area dimensions must fit the engine's index quantization"); // TODO ERROR HANDLING
                self.data
                    .cortical_layout_dimensional_data
                    .append_single_mut(CorticalAreaLayoutDimensional { dimensions });
            }
            CorticalAreaLayoutNested::Formless(_) => {
                todo!("formless cortical area layouts are not allocated by this engine yet")
            }
        }

        // Per-area entries, all indexed by `CorticalEngineIndex`.
        self.data.cortical_neuron_model_and_quant_and_neuron_properties.append_single_mut((
            NeuronModelTypeAndQuantizationPacked::FeagiAdvanced_Standard,
            CorticalRuntimeFlags::from_cortical_area_properties(&cortical_properties),
        ));
        self.data
            .cortical_index_lookup_table
            .append_single_mut(CorticalIndexLookupTable::new(cortical_model_index, cortical_layout_index));
        self.data.cortical_neuron_count.append_single_mut(number_neurons);

        // One firing bit per neuron. Allocated in the same order as every other per-area vector,
        // so the run id the manager hands back is this area's `CorticalEngineIndex`.
        let firing_bitmap_index = self.data.neuron_voxel_is_firing.get_new_range(number_neurons);
        debug_assert_eq!(
            firing_bitmap_index,
            cortical_index.deref(),
            "firing bitmap runs must stay aligned with cortical engine indexes"
        );
        self.data
            .cortical_neuron_index_lookup_table
            .append_single_mut(NeuronIndexLookupTable::new(
                first_neuron_engine_index,
                first_neuron_mp_index,
                first_neuron_model_index,
                first_neuron_history_index,
            ));

        // Per-neuron entries. `cortical_engine_indexes` is what the burst kernel iterates, so it
        // has to grow by exactly this area's neuron count for the area to be processed at all.
        self.data.cortical_engine_indexes.extend_mut(number_neurons.into(), cortical_index);
        self.data
            .neuron_membrane_data
            .mp_f32
            .extend_mut(number_neurons.into(), NeuronMembranePotential::QUANT_ZERO);
        self.data
            .neuron_membrane_data
            .fcl_f32
            .extend_mut(number_neurons.into(), NeuronMembranePotential::QUANT_ZERO);
        self.data.neuron_history_data.extend_mut(
            number_neurons.into(),
            NeuronModelFullNeuronHistory::new(self.data.burst_index, BurstIndex::QUANT_ZERO),
        );

        // Runtime flags carry the writer's per-neuron probe settings into the engine.
        let neuron_runtime_flags = self
            .data
            .neuron_runtime_flags
            .extend_mut(number_neurons.into(), NeuronRuntimeFlags::new(false, false));
        for (flags, properties) in neuron_runtime_flags.iter_mut().zip(neuron_properties.iter()) {
            *flags = NeuronRuntimeFlags::new(properties.probe_force_disabled, properties.probe_force_firing);
        }

        self.latest_cortical_index += CorticalEngineIndex::QUANT_ONE;
        cortical_index
    }

    fn edit_cortical_area_cortical_flags(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) {
        todo!()
    }

    fn edit_cortical_area_cortical_data<NMQ: NeuronModelQuantization, NM: NeuronModel<FIQ, NMQ>>(&mut self, new_cortical_data: NM::CorticalData) {
        todo!()
    }

    fn remove_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) {
        todo!()
    }

    fn resize_dimensional_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>) {
        todo!()
    }

    fn add_mapping_entry<SM>(
        &mut self,
        source_cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
        destination_cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
        writer: UniformWriter,
    ) -> MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>
    where
        SM: SynapseModel<
            FIQ,
            UniformSynapseModelStandardQuant,
            CorticalMappingEntryData = UniformSynapseModelCorticalMappingEntryData<UniformSynapseModelStandardQuant>,
            SynapseData = EmptyPerSynapseData,
        >,
    {
        // A doublet only becomes a concrete pairing once both ends' layouts are known, which is why
        // it stays genomic until it reaches the engine.
        let source_layout = self.dimensional_layout_of(source_cortical_index);
        let destination_layout = self.dimensional_layout_of(destination_cortical_index);

        let UniformWriter::Standard { doublet, .. } = &writer;
        match doublet {
            DoubletIteratorDimensionalTypeGenomic::OneToOne { source, destination } => {
                let iterator = DoubletIteratorOneToOne::new(
                    to_engine_voxel_coordinate::<FIQ>(*source),
                    to_engine_voxel_coordinate::<FIQ>(*destination),
                    &source_layout,
                    &destination_layout,
                );
                self.append_uniform_mapping_entry(iterator, &writer, source_cortical_index, destination_cortical_index)
            }
            DoubletIteratorDimensionalTypeGenomic::OneToAll { source } => {
                let iterator = DoubletIteratorOneToAll::new(to_engine_voxel_coordinate::<FIQ>(*source), &source_layout, &destination_layout);
                self.append_uniform_mapping_entry(iterator, &writer, source_cortical_index, destination_cortical_index)
            }
            DoubletIteratorDimensionalTypeGenomic::AllToOne { destination } => {
                let iterator = DoubletIteratorAllToOne::new(to_engine_voxel_coordinate::<FIQ>(*destination), &source_layout, &destination_layout);
                self.append_uniform_mapping_entry(iterator, &writer, source_cortical_index, destination_cortical_index)
            }
        }
    }

    fn remap_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>) {
        todo!()
    }

    fn remove_mapping_entry(&mut self, mapping_entry: MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>) {
        todo!()
    }

    /*
    fn probe_cortical_area(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, _: flags) {
        todo!()
    }

    fn probe_neurons(&mut self, cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>, _: iterator) {
        todo!()
    }

    fn probe_mapping_entries(&mut self) {
        todo!()
    }

     */
}

/*
Synaptogenesis: a mapping entry allocated by `add_mapping_entry` must actually carry potential from
its source neuron to its destination when a burst runs.

These live inside the crate rather than in `tests/` because the path has no public stimulus or
readout yet: the placeholder `FeagiAdvancedModel` dynamics never write a membrane potential, so the
source has to be seeded directly, and there is no accessor to read the destination's back out.
*/
#[cfg(test)]
mod tests {
    use super::*;
    use core::marker::PhantomData;
    use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
    use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
    use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationGenomic;
    use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::composers::FeagiAdvancedModelCorticalWriter;
    use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::model::FeagiAdvancedModel;
    use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::data::UniformSynapseMultiplier;
    use feagi_models::cortical_mapping_entry::synapse_model_implementations::uniform::model::UniformSynapseModel;
    use feagi_models::wrapped_index_collections::SynapseEngineIndex;

    type TestFIQ = FeagiIndexQuantizationGenomic;
    type TestEngine = RayonBurstEngine<TestFIQ>;
    type TestNeuronModel = FeagiAdvancedModel<TestFIQ, FeagiAdvancedModelStandardQuant>;
    type TestSynapseModel = UniformSynapseModel<TestFIQ, UniformSynapseModelStandardQuant>;

    /// Adds a one neuron dimensional area, the smallest allocation that still has a real layout.
    fn add_single_neuron_area(engine: &mut TestEngine) -> CorticalEngineIndex<<TestFIQ as FeagiIndexQuantization>::CorticalAreaIndexCountQuant> {
        let dimensions = DimensionalCorticalArea4DDimensions::try_new_from_usizes(1, 1, 1, 1).expect("unit dimensions should be representable");
        let writer = FeagiAdvancedModelCorticalWriter::DefaultNewDimensional {
            dimensions,
            _p: PhantomData::<FeagiAdvancedModelStandardQuant>,
        };
        engine.add_cortical_area::<TestNeuronModel>(writer)
    }

    /// A OneToOne mapping between the origin voxel of each area.
    fn one_to_one_writer(weight: f32, is_inhibitory: bool) -> UniformWriter {
        UniformWriter::Standard {
            doublet: DoubletIteratorDimensionalTypeGenomic::OneToOne {
                source: NeuronVoxelCoordinate::new_from_usizes_unchecked(0, 0, 0),
                destination: NeuronVoxelCoordinate::new_from_usizes_unchecked(0, 0, 0),
            },
            uniform_weight: UniformSynapseMultiplier::new(weight),
            propagation_delay: 0,
            is_inhibitory,
        }
    }

    /// Two single neuron areas joined by one mapping, with the source's membrane potential seeded.
    /// The placeholder neuron dynamics leave membrane potentials untouched, so whatever is seeded
    /// here is exactly what the synapse reads.
    fn engine_with_one_synapse(source_potential: f32, weight: f32, is_inhibitory: bool) -> TestEngine {
        let mut engine = TestEngine::new();
        let source = add_single_neuron_area(&mut engine);
        let destination = add_single_neuron_area(&mut engine);

        engine.add_mapping_entry::<TestSynapseModel>(source, destination, one_to_one_writer(weight, is_inhibitory));

        let source_mp_index = engine.first_membrane_potential_index_of(source);
        engine
            .data
            .neuron_membrane_data
            .mp_f32
            .set(source_mp_index, NeuronMembranePotential::new(source_potential))
            .expect("source neuron must have membrane potential storage");

        engine
    }

    /// The destination's accumulated potential, which is what a synapse deposits into.
    fn destination_potential(engine: &TestEngine) -> f32 {
        let destination = CorticalEngineIndex::new(1);
        let destination_mp_index = engine.first_membrane_potential_index_of(destination);
        engine
            .data
            .neuron_membrane_data
            .fcl_f32
            .get(destination_mp_index)
            .expect("destination neuron must have accumulation storage")
            .deref()
    }

    #[test]
    fn mapping_entry_allocates_one_synapse_per_neuron_pairing() {
        let engine = engine_with_one_synapse(0.0, 1.0, false);

        assert_eq!(engine.data.cortical_mapping_entry_indexes.len().deref(), 1);
        assert_eq!(engine.data.synapse_source_destination_mp_neuron_indexes.len().deref(), 1);
        assert_eq!(engine.data.cortical_mapping_entry_properties.len().deref(), 1);
    }

    #[test]
    fn synapse_endpoints_are_stored_as_engine_wide_membrane_potential_indexes() {
        let engine = engine_with_one_synapse(0.0, 1.0, false);

        // The two areas hold one neuron each, so the second area's neuron sits at membrane
        // potential index one. A synapse still holding the area local index would read zero here.
        let (source, destination) = engine
            .data
            .synapse_source_destination_mp_neuron_indexes
            .get(SynapseEngineIndex::QUANT_ZERO)
            .expect("the mapping should have allocated one synapse");
        assert_eq!(source.deref(), 0);
        assert_eq!(destination.deref(), 1);
    }

    #[test]
    fn an_excitatory_synapse_carries_scaled_potential_to_its_destination() {
        let mut engine = engine_with_one_synapse(2.0, 0.5, false);
        assert_eq!(destination_potential(&engine), 0.0, "nothing should have moved before a burst");

        engine.execute_single_burst();

        assert_eq!(
            destination_potential(&engine),
            1.0,
            "destination should receive source potential times weight"
        );
    }

    #[test]
    fn an_inhibitory_synapse_subtracts_from_its_destination() {
        let mut engine = engine_with_one_synapse(2.0, 0.5, true);

        engine.execute_single_burst();

        assert_eq!(
            destination_potential(&engine),
            -1.0,
            "inhibitory polarity should flip the sign of the arriving potential"
        );
    }

    #[test]
    fn a_disabled_mapping_entry_carries_nothing() {
        let mut engine = engine_with_one_synapse(2.0, 0.5, false);
        engine
            .data
            .cortical_mapping_entry_properties
            .get_mut(MappingEntryEngineIndex::QUANT_ZERO)
            .expect("the mapping entry should exist")
            .flags = CorticalMappingEntryRuntimeFlags::new(true, false);

        engine.execute_single_burst();

        assert_eq!(destination_potential(&engine), 0.0);
    }

    #[test]
    fn repeated_bursts_accumulate_at_the_destination() {
        let mut engine = engine_with_one_synapse(2.0, 0.5, false);

        engine.execute_single_burst();
        engine.execute_single_burst();
        engine.execute_single_burst();

        // Nothing drains the accumulation buffer yet, so each burst deposits again. This pins the
        // current behaviour so that adding a drain is a deliberate change rather than a silent one.
        assert_eq!(destination_potential(&engine), 3.0);
    }
}
