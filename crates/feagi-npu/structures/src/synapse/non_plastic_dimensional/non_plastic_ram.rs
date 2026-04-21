use core::marker::PhantomData;
use ahash::AHashMap;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::{IndexTracker, IndexedDataTracker, RangeUintVector};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalTypedCorticalIndex, DimensionalTypedNeuronIndex, NPUDimensionalAreaType};
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUQuantization, NPUNeuronIndex, NPUSynapseIndex, SynapseBundleIndex, SynapseCount};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStorageTrait};
use crate::synapse::dimension_to_dimension_traits::{Dim2DimSynapseAllocStorageTrait, Dim2DimSynapseBaseStorageTrait};
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;
use crate::synapse::non_plastic_dimensional::NonPlasticSynapseFull;
use crate::synapse::non_plastic_dimensional::traits::{NonplasticSynapseAllocStorageTrait, NonplasticSynapseBaseStorageTrait};
// NOTE: since deletions are going to be generally uncommon (and done in blocks0 and since this
// synapse is very numerous, we are not going to store the neuron indexes in the synaptic data struct.
// This does mean there is no "easy" way to look up the source / destination neurons from a synapse
// index itself, and thus means their deletion is a bit more involved computationally. But I believe
// this is worth the other gains when designed carefully

// TODO we can optimize this by shoving the cortical types into the flags

pub struct NonplasticDimensionalSynapseAllocRAMStorage<Q: NPUQuantization>
{
    // Data
    synapses_data: Vec<NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>,
    source_to_synapse: AHashMap<DimensionalTypedNeuronIndex<Q::NeuronIndex>, Vec<NPUSynapseIndex<Q::SynapseIndex>>>,
    destination_to_synapse: AHashMap<DimensionalTypedNeuronIndex<Q::NeuronIndex>, Vec<NPUSynapseIndex<Q::SynapseIndex>>>,

    // Cached Data
    cache_valid_synapse_count: SynapseCount<Q::SynapseIndex>,
    cache_invalid_synapse_count: SynapseCount<Q::SynapseIndex>,
    /// Includes ranges of entire valid synapse blocks mapped to their cortical mapping. MAY INCLUDE individual dead synapses
    cache_valid_synapse_blocks: AHashMap<
        (CorticalAreaIndex<Q::CorticalIndex>, CorticalAreaIndex<Q::CorticalIndex>),
        IndexedDataTracker<core::ops::Range<NPUSynapseIndex<Q::SynapseIndex>>>
    >,
    /// Includes ranges of entire invalid synapse blocks. Does NOT notate singular dead synapses
    cache_invalid_synapse_blocks: RangeUintVector<NPUSynapseIndex<Q::SynapseIndex>>,

    _phantom: PhantomData<Q>,
}

impl<Q: NPUQuantization>
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    pub fn new(number_synapses_to_preallocate: SynapseCount<Q::SynapseIndex>) -> Self {
        let count = number_synapses_to_preallocate.to_usize();
        Self {
            synapses_data: Vec::with_capacity(count),
            source_to_synapse: AHashMap::with_capacity(count),
            destination_to_synapse: AHashMap::with_capacity(count),
            cache_total_synapse_count: SynapseCount::ZERO,
            cache_valid_synapse_count: SynapseCount::ZERO,
            cache_invalid_synapse_count: SynapseCount::ZERO,
            cache_valid_synapse_blocks: AHashMap::new(),
            cache_invalid_synapse_blocks: Vec::new(),
            _phantom: PhantomData,
        }
    }

    fn insert_valid_synapse_block_and_get_index(&mut self,
                                                synapse_block: core::ops::Range<NPUSynapseIndex<Q::SynapseIndex>>,
                                                source_area: CorticalAreaIndex<Q::CorticalIndex>,
                                                destination_area: CorticalAreaIndex<Q::CorticalIndex>)
        -> SynapseBundleIndex<Q::SynapseBundleIndex>
    {
        let key = (source_area, destination_area);
        if !self.cache_valid_synapse_blocks.contains_key(&key) {
            self.cache_valid_synapse_blocks.insert(key, IndexedDataTracker::new());
        }
        let block_vec = self.cache_valid_synapse_blocks.get_mut(&key).unwrap();
        let index = block_vec.insert(synapse_block);
        SynapseBundleIndex::from_usize(index)
    }

    //region Get Synapse Data

    /// Tries to get synapse at given index. Errors if index is invalid. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_data_at_synapse_index(&self, synapse_index: NPUSynapseIndex<Q::SynapseIndex>)
        -> Result<&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: self.cache_total_synapse_count.to_usize() as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if index and synapse valid
    fn get_valid_synapse_data_at_synapse_index(&self, synapse_index: NPUSynapseIndex<Q::SynapseIndex>)
        -> Result<&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>, FeagiNPUSynapseError>
    {
        let possible_valid = self.get_synapse_data_at_synapse_index(synapse_index)?;
        if !possible_valid.is_valid() {
            return Err(FeagiNPUSynapseError::SynapseIndexIsInvalid {
                context: "Expected valid synapse at index!",
                given_synapse_index: synapse_index.to_usize() as u32 })
        }
        Ok(possible_valid)
    }

    /// Tries to get synapse at given index. Errors if index is invalid. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_data_at_synapse_index_mut(&mut self, synapse_index: NPUSynapseIndex<Q::SynapseIndex>)
        -> Result<&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get_mut(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: self.cache_total_synapse_count.to_usize() as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if index and synapse valid
    fn get_valid_synapse_data_at_synapse_index_mut(&mut self, synapse_index: NPUSynapseIndex<Q::SynapseIndex>)
        -> Result<&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>, FeagiNPUSynapseError>
    {
        let possible_valid = self.get_synapse_data_at_synapse_index_mut(synapse_index)?;
        if !possible_valid.is_valid() {
            return Err(FeagiNPUSynapseError::SynapseIndexIsInvalid {
                context: "Expected valid synapse at index!",
                given_synapse_index: synapse_index.to_usize() as u32 })
        }
        Ok(possible_valid)
    }

    /// Gets all synapse indexes that have the following source_neuron_index. Returns empty if nothing found
    fn get_synapse_indexes_from_source_neuron_index(&self, source_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> &[NPUSynapseIndex<Q::SynapseIndex>] {
        if let Some(val) = self.source_to_synapse.get(source_neuron_index) {
            return val.as_slice()
        }
        &[]
    }

    /// Gets all synapse indexes that have the following source_neuron_index. Returns empty if nothing found
    fn get_synapse_indexes_from_destination_neuron_index(&self, destination_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        ->  &[NPUSynapseIndex<Q::SynapseIndex>] {
        if let Some(val) = self.destination_to_synapse.get(destination_neuron_index) {
            return val.as_slice()
        }
        &[]
    }

    /// Gets synapse data from a source neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_source_neuron_index(&self, source_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_source_neuron_index(source_neuron_index);
        let data_len = self.synapses_data.len();
        for synapse_index in synapse_indexes {
            if synapse_index.to_usize() >= data_len {
                return Err(FeagiNPUSynapseError::InternalError {
                    context: "Source neuron index pointed to invalid synapse index! Internal state corrupted!"
                });
            }
        }
        let iterator = synapse_indexes.iter().map(|synapse_index|
            &self.synapses_data[synapse_index.to_usize()]
        );
        Ok((iterator, NeuronCount::from_usize(synapse_indexes.len())))
    }

    /// Gets synapse data from a destination neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_destination_neuron_index(destination_neuron_index);
        let data_len = self.synapses_data.len();
        for synapse_index in synapse_indexes {
            if synapse_index.to_usize() >= data_len {
                return Err(FeagiNPUSynapseError::InternalError {
                    context: "Destination neuron index pointed to invalid synapse index! Internal state corrupted!"
                });
            }
        }
        let iterator = synapse_indexes.iter().map(|synapse_index|
            &self.synapses_data[synapse_index.to_usize()]
        );
        Ok((iterator, NeuronCount::from_usize(synapse_indexes.len())))
    }

    /// Gets mut synapse data from a source neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_source_neuron_index_mut(&mut self, source_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        let indexes: Vec<usize> = self.get_synapse_indexes_from_source_neuron_index(source_neuron_index)
            .iter()
            .map(|idx| idx.to_usize())
            .collect();
        let data_len = self.synapses_data.len();
        for &idx in &indexes {
            if idx >= data_len {
                return Err(FeagiNPUSynapseError::InternalError {
                    context: "Source neuron index pointed to invalid synapse index! Internal state corrupted!"
                });
            }
        }
        let count = indexes.len();
        let iterator = self.synapses_data.iter_mut()
            .enumerate()
            .filter_map(move |(i, synapse)| {
                if indexes.contains(&i) { Some(synapse) } else { None }
            });
        Ok((iterator, NeuronCount::from_usize(count)))
    }

    /// Gets mut synapse data from a destination neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_destination_neuron_index_mut(&mut self, destination_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndex>)
        -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        let indexes: Vec<usize> = self.get_synapse_indexes_from_destination_neuron_index(destination_neuron_index)
            .iter()
            .map(|idx| idx.to_usize())
            .collect();
        let data_len = self.synapses_data.len();
        for &idx in &indexes {
            if idx >= data_len {
                return Err(FeagiNPUSynapseError::InternalError {
                    context: "Destination neuron index pointed to invalid synapse index! Internal state corrupted!"
                });
            }
        }
        let count = indexes.len();
        let iterator = self.synapses_data.iter_mut()
            .enumerate()
            .filter_map(move |(i, synapse)| {
                if indexes.contains(&i) { Some(synapse) } else { None }
            });
        Ok((iterator, NeuronCount::from_usize(count)))
    }

    //endregion


}

impl<Q: NPUQuantization>
NonplasticSynapseAllocStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>,
                                                   source_neuron_indexes: core::ops::Range<NPUNeuronIndex<Q::NeuronIndex>>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   source_cortical_dimensions: &NeuronVoxelDimensions<Q::Coord>,
                                                   source_neuron_density: NumberNeuronsPerVoxel,
                                                   destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>,
                                                   destination_neuron_indexes: core::ops::Range<NPUNeuronIndex<Q::NeuronIndex>>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   destination_cortical_dimensions: &NeuronVoxelDimensions<Q::Coord>,
                                                   destination_neuron_density: NumberNeuronsPerVoxel,
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q::NeuronIndex, Q::SynapseIndex, Q::Coord, Q::CorticalIndex, Q::BurstDelta, Q::Value>)
        -> Result<SynapseBundleIndex<Q::SynapseBundleIndex>, FeagiStructuresError>{


        let (synapse_iterator, number_synapses) = neuron_mapping_executor.non_plastic_synapse_iterator(
            source_neuron_indexes, source_neuron_flags, source_cortical_dimensions, source_neuron_density,
            source_area_index.dimensional_type, destination_neuron_indexes, destination_neuron_flags,
            destination_cortical_dimensions, destination_neuron_density,
            destination_area_index.dimensional_type)?;

        // TODO check length is ok
        // TODO check that you dont spawn in dead synapses

        // Data
        let number_synapses: usize = number_synapses.to_usize();
        let synapse_writing_region = self.cache_invalid_synapse_blocks.find_space(NPUSynapseIndex::from_usize(number_synapses)); // TODO incorrect type! Shouldnt this take in count?

        let (synapse_bundle_index, synapse_writing_region, extending) = match synapse_writing_region {
            None => {
                let synapse_writing_region = NPUSynapseIndex::from_usize(self.synapses_data.len()) .. NPUSynapseIndex::from_usize(self.synapses_data.len() + number_synapses);
                let synapse_bundle_index = 
                
                // Allocate at the end
                self.synapses_data.reserve(number_synapses);
                self.source_to_synapse.reserve(number_synapses);
                self.destination_to_synapse.reserve(number_synapses);
                
                

                let starting_synapse_index = NPUSynapseIndex::from_usize(self.synapses_data.len());
                for (local_index, synapse) in synapse_iterator.enumerate() {
                    let synapse_index = starting_synapse_index + NPUSynapseIndex::from_usize(local_index);
                    self.source_to_synapse.entry(synapse.source_neuron_index).or_insert_with(vec![synapse_index]);
                    self.destination_to_synapse.entry(synapse.destination_neuron_index).or_insert_with(vec![synapse_index]);
                    self.synapses_data.push(synapse);
                }
                
                
            }
            
            Some(synapse_writing_region) => {
                // We have a region to write within, no need to allocate
            }

        }







        let starting_synapse_index: NPUSynapseIndex<Q::SynapseIndex>;

        // TODO check that you dont spawn in dead synapses
        if false {
            // TODO instead of allocating right to the end, what if we have a way to quickly check through cache_invalid_neuron_indexes (assuming we also group neighboring ranges) and put ourselves there if we fit?
            starting_synapse_index = NPUSynapseIndex::from_usize(self.synapses_data.len()) // TODO replace with first spot
        } else {
            self.synapses_data.reserve(number_synapses);
            self.source_to_synapse.reserve(number_synapses);
            self.destination_to_synapse.reserve(number_synapses);
            starting_synapse_index = NPUSynapseIndex::from_usize(self.synapses_data.len());
            for (local_index, synapse) in synapse_iterator.enumerate() {
                let synapse_index = starting_synapse_index + NPUSynapseIndex::from_usize(local_index);
                self.source_to_synapse.entry(synapse.source_neuron_index).or_insert_with(Vec::new).push(synapse_index);
                self.destination_to_synapse.entry(synapse.destination_neuron_index).or_insert_with(Vec::new).push(synapse_index);
                self.synapses_data.push(synapse);
            }
        }

        // Cache properties
        let synapse_range = starting_synapse_index..(starting_synapse_index + NPUSynapseIndex::from_usize(number_synapses));

        let added = SynapseCount::from_usize(number_synapses);
        self.cache_valid_synapse_count += added;
        self.cache_total_synapse_count += added;
        Ok(self.insert_valid_synapse_block_and_get_index(synapse_range, source_area_index, destination_area_index))

    }
}

impl<Q: NPUQuantization>
NonplasticSynapseBaseStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    //region Get Connections
    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<Q::NeuronIndex>, source_neuron_type: &NPUDimensionalNeuronType) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        self.get_synapse_data_from_source_neuron_index(&source_neuron_index)
    }

    fn get_nonplastic_synapse_data_from_source_neuron_index_mut(&mut self, source_neuron_index: NPUNeuronIndex<Q::NeuronIndex>, source_neuron_type: &NPUDimensionalNeuronType) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        self.get_synapse_data_from_source_neuron_index_mut(&source_neuron_index)
    }

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<Q::NeuronIndex>, destination_neuron_type: &NPUDimensionalNeuronType) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        self.get_synapse_data_from_destination_neuron_index(&destination_neuron_index)
    }

    fn get_nonplastic_synapse_data_from_destination_neuron_index_mut(&mut self, destination_neuron_index: NPUNeuronIndex<Q::NeuronIndex>, destination_neuron_type: &NPUDimensionalNeuronType) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, NeuronCount<Q::NeuronIndex>), FeagiNPUSynapseError> {
        self.get_synapse_data_from_destination_neuron_index_mut(&destination_neuron_index)
    }

    //endregion

}

impl<Q: NPUQuantization>
Dim2DimSynapseAllocStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    fn remove_all_synapses_mappings_to_and_from_cortical_area(&mut self, area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn remove_all_synaptic_mappings_between_cortical_areas(&mut self, source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>, destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndex>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }
}

impl<Q: NPUQuantization>
Dim2DimSynapseBaseStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    //region Get Connections

    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<&[DimensionalTypedNeuronIndex<Q::NeuronIndex>], FeagiNPUSynapseError> {
        todo!()
    }

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<&[DimensionalTypedNeuronIndex<Q::NeuronIndex>], FeagiNPUSynapseError> {
        todo!()
    }

    //endregion

    //region Sparse Synapse Invalidation

    fn invalidate_synapse_by_synapse_index(&mut self, synapse_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_by_synapse_indexes(&mut self, synapse_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_source_neuron_index(&mut self, source_neurons_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_source_neuron_indexes(&mut self, source_neurons_indexes: &[DimensionalTypedNeuronIndex<Q::NeuronIndex>]) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_destination_neuron_index(&mut self, destination_neurons_index: DimensionalTypedNeuronIndex<Q::NeuronIndex>) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_destination_neuron_indexes(&mut self, destination_neurons_indexes: &[DimensionalTypedNeuronIndex<Q::NeuronIndex>]) -> Result<SynapseCount<Q::SynapseIndex>, FeagiNPUSynapseError> {
        todo!()
    }

    //endregion
}

impl<Q: NPUQuantization>
BaseSynapseAllocStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    fn free_unused_synapse_capacity(&mut self, spare_capacity_to_maintain: SynapseCount<Q::SynapseIndex>) -> SynapseCount<Q::SynapseIndex> {
        let target = self.cache_total_synapse_count.to_usize() + spare_capacity_to_maintain.to_usize();
        self.synapses_data.shrink_to(target);
        self.source_to_synapse.shrink_to_fit();
        self.destination_to_synapse.shrink_to_fit();
        // TODO delete empty vec keys?
        self.cache_total_synapse_count
    }
}

impl<Q: NPUQuantization>
BaseSynapseStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    const NUMBER_BYTES_PER_SYNAPSE: usize = 0; // TODO

    fn get_max_possible_synapse_index(&self) -> NPUSynapseIndex<Q::SynapseIndex> {
        NPUSynapseIndex::MAX_VALUE
    }

    fn get_total_number_of_synapses(&self) -> &SynapseCount<Q::SynapseIndex> {
        &self.cache_total_synapse_count
    }

    fn get_total_number_of_valid_synapses(&self) -> &SynapseCount<Q::SynapseIndex> {
        &self.cache_valid_synapse_count
    }

    fn get_total_number_of_invalid_synapses(&self) -> &SynapseCount<Q::SynapseIndex> {
        &self.cache_invalid_synapse_count
    }
}
