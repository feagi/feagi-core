use crate::engines::rayon::data::neuron::neuron_sub_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};
use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::kernels_neurons;
use crate::engines::rayon::kernels_synapses;
use crate::engines_common::EditableEngine::EditableEngine;
use crate::flags::cortical_runtime_flags::CorticalRuntimeFlags;
use crate::flags::neuron_runtime_flags::NeuronRuntimeFlags;
use feagi_data::neurons::NeuronMembranePotential;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_models::cortical_area::components::cortical_area_layout::enums::CorticalAreaLayoutNested;
use feagi_models::cortical_area::components::cortical_area_layout::implementations::dimensional::CorticalAreaLayoutDimensional;
use feagi_models::cortical_area::components::neuron_history::implementations::full::NeuronModelFullNeuronHistory;
use feagi_models::cortical_area::components::neuron_history::neuron_history::NeuronModelHistory;
use feagi_models::cortical_area::genome_compose::cortical_writer::NeuronModelCorticalWriter;
use feagi_models::cortical_area::neuron::neuron_model::NeuronModel;
use feagi_models::cortical_area::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_models::cortical_area::neuron::neuron_properties::NeuronProperties;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::data::{FeagiAdvancedModelCorticalData, FeagiAdvancedModelNeuronData};
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use feagi_models::cortical_area::neuron_model_implementations::generated_enums::NeuronModelTypeAndQuantizationPacked;
use feagi_models::cortical_mapping_entry::genome_compose::cortical_mapping_entry_writer_by_model_quant::UniformWriter;
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

    fn add_mapping_entry<SM>(&mut self, writer: UniformWriter) -> MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant> {
        /*

        let number_synapses = writer:


         */

        // TODO
        MappingEntryEngineIndex::QUANT_ZERO
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
