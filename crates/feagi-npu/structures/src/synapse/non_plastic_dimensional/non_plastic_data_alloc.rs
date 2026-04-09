use ahash::AHashMap;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::npu_neuron_type::NPUNeuronType;
use crate::quantizables::{NPUNeuronIndex, NPUSynapseIndex, SynapseCount};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::{Dim2DimSynapseAllocStorageTrait, Dim2DimSynapseBaseStorageTrait};
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::{NonPlasticSynapseFull, NonplasticSynapseProperties};
use crate::synapse::non_plastic_dimensional::traits::{NonplasticSynapseAllocStorageTrait, NonplasticSynapseBaseStorageTrait};
// NOTE: since deletions are going to be generally uncommon (and done in blocks0 and since this
// synapse is very numerous, we are not going to store the neuron indexes in the synaptic data struct.
// This does mean there is no "easy" way to look up the source / destination neurons from a synapse
// index itself, and thus means their deletion is a bit more involved computationally. But I believe
// this is worth the other gains when designed carefully


pub struct NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // Data
    synapses_data: Vec<NonPlasticSynapseFull<ValueQuant, BurstDeltaQuant, ValueQuant>>,
    source_to_destination: AHashMap<NPUNeuronIndex<NeuronIndexQuant>, Vec<NPUSynapseIndex<SynapseIndexQuant>>>,
    destination_to_source: AHashMap<NPUNeuronIndex<NeuronIndexQuant>, Vec<NPUSynapseIndex<SynapseIndexQuant>>>,

    // Cached Data
    cache_valid_synapse_count: SynapseCount<SynapseIndexQuant>,
    cache_invalid_synapse_count: SynapseCount<SynapseIndexQuant>,
    /// Includes ranges of entire valid synapse blocks mapped to their cortical mapping. MAY INCLUDE individual dead synapses
    cache_valid_synapse_blocks: AHashMap<(
        CorticalAreaIndex<CorticalIndexQuant>, CorticalAreaIndex<CorticalIndexQuant>),
        Vec<core::ops::Range<SynapseIndexQuant>>>,
    /// Includes ranges of entire invalid synapse blocks. Does NOT include singular dead synapses
    cache_invalid_synapse_blocks: Vec<core::ops::Range<SynapseIndexQuant>>,
}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    pub fn new(number_synapses_to_preallocate: SynapseCount<SynapseIndexQuant>) -> Self {
        let count = number_synapses_to_preallocate.to_usize();
        NonplasticDimensionalSynapseAllocRAMStorage {
            synapses_data: Vec::with_capacity(count),
            source_to_destination: AHashMap::with_capacity(count),
            destination_to_source: AHashMap::with_capacity(count),
            cache_valid_synapse_count: SynapseCount(0),
            cache_invalid_synapse_count: SynapseCount(0),
            cache_valid_synapse_blocks: AHashMap::new(),
            cache_invalid_synapse_blocks: Vec::new(),
        }
    }

    /// Tries to get synapse at given index. Errors if something is wrong. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_at_synapse_index(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: self.get_total_number_of_synapses().to_usize() as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if valid
    fn get_valid_synapse_at_synapse_index(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let possible_valid = self.get_synapse_at_synapse_index(synapse_index)?;
        if !possible_valid.is_valid() {
            return Err(FeagiNPUSynapseError::SynapseIndexIsInvalid {
                context: "Expected valid synapse at index!",
                given_synapse_index: synapse_index.to_usize() as u32 })
        }
        Ok(possible_valid)
    }

    /// Tries to get synapse at given index. Errors if something is wrong. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_at_synapse_index_mut(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get_mut(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: self.get_total_number_of_synapses().to_usize() as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if valid
    fn get_valid_synapse_at_synapse_index_mut(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let possible_valid = self.get_synapse_at_synapse_index_mut(synapse_index)?;
        if !possible_valid.is_valid() {
            return Err(FeagiNPUSynapseError::SynapseIndexIsInvalid {
                context: "Expected valid synapse at index!",
                given_synapse_index: synapse_index.to_usize() as u32 })
        }
        Ok(possible_valid)
    }

}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
NonplasticSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    fn add_synapses_mapping_between_cortical_areas(&mut self, source_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   destination_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   number_of_synapses: SynapseCount<SynapseIndexQuant>,
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>) {

        // TODO check length is ok

        // actual data

        let (synapse_index_range, is_allocation_at_end_needed): (core::ops::Range<NPUSynapseIndex<NeuronIndexQuant>>, bool) = {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            //if self.cache_number_invalid_neurons.to_usize() > number_of_neurons {
            //
            //}
            // TODO size checks (not debug only, we need to be careful)
            (NPUSynapseIndex::from_usize(self.synapses_data.len())..(self.synapses_data.len() + number_of_synapses), true)
        };

        // TODO this iterator is getting consumed by these functions, or we have to call them multiple times
        // This isnt acceptable, we will need to find a way to do the following:
        // - iterate over the iterator
        // - for each value, open the produced struct, set the hash lookups with the neuron maps
        // - close the struct and shove it in the overall index at the right spot
        // There are some ways to do this naively but I am concerne about performance

        if is_allocation_at_end_needed {
            // TODO expand capacity
            self.synapses_data.extend(neuron_mapping_executor)
        } else {
            self.synapses_data[&synapse_index_range].fill(neuron_mapping_executor)
        }

        // cache
        let mut synapse_ranges: Vec<core::ops::Range<SynapseIndexQuant>> = self.cache_valid_synapse_blocks.get_mut((source_area_index, destination_area_index)).ok_or_else(
            self.cache_valid_synapse_blocks.push(Vec::new())
        );

        synapse_ranges.push(synapse_index_range)



    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
NonplasticSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
Dim2DimSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    fn remove_all_synapses_mappings_to_and_from_cortical_area(&mut self, area_index: CorticalAreaIndex<CorticalIndexQuant>) {
        todo!()
    }

    fn remove_all_synaptic_mappings_between_cortical_areas(&mut self, source_area_index: CorticalAreaIndex<CorticalIndexQuant>, destination_area_index: CorticalAreaIndex<CorticalIndexQuant>) {
        todo!()
    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
Dim2DimSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&[NPUNeuronIndex<NeuronIndexQuant>], FeagiNPUSynapseError> {
        todo!()
    }

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&[NPUNeuronIndex<NeuronIndexQuant>], FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapse_by_synapse_index(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_by_synapse_indexes(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_source_neuron_index(&mut self, source_neurons_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_source_neuron_indexes(&mut self, source_neurons_indexes: &[NPUNeuronIndex<NeuronIndexQuant>]) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_destination_neuron_index(&mut self, destination_neurons_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_destination_neuron_indexes(&mut self, destination_neurons_indexes: &[NPUNeuronIndex<NeuronIndexQuant>]) -> Result<SynapseCount<SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
BaseSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    fn free_unused_synapse_capacity(&mut self, spare_capacity_to_maintain: SynapseCount<SynapseIndexQuant>) -> SynapseCount<SynapseIndexQuant> {
        self.synapses_data.shrink_to(self.get_total_number_of_synapses() + spare_capacity_to_maintain);
        self.source_to_destination.shrink_to_fit();
        self.destination_to_source.shrink_to_fit();
        // TODO delete empty vec keys?
        *self.get_total_number_of_synapses()
    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
BaseSynapseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    const NUMBER_BYTES_PER_SYNAPSE: usize = 0; // TODO

    fn get_max_possible_synapse_index(&self) -> NPUSynapseIndex<SynapseIndexQuant> {
        NPUSynapseIndex::MAX_VALUE
    }

    fn get_total_number_of_synapses(&self) -> &SynapseCount<SynapseIndexQuant> {
        &(self.cache_valid_synapse_count + self.cache_invalid_synapse_count)
    }

    fn get_total_number_of_valid_synapses(&self) -> &SynapseCount<SynapseIndexQuant> {
        &self.cache_valid_synapse_count
    }

    fn get_total_number_of_invalid_synapses(&self) -> &SynapseCount<SynapseIndexQuant> {
        &self.cache_invalid_synapse_count
    }
}

