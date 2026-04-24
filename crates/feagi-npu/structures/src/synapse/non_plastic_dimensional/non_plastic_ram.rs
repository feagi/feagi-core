use core::marker::PhantomData;
use core::ops::Range;
use ahash::AHashMap;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::{IndexTracker, IndexedDataTracker, RangeUintVector};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::dimensional_neurons::shared_structs::{DimensionalNeuronCorticalData, DimensionalTypedCorticalIndex, DimensionalTypedNeuronIndex};
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUQuantization, NPUNeuronIndex, SynapseIndex, SynapseBundleIndex, SynapseCount};
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
    synapses_data: Vec<NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>,
    source_to_synapse: AHashMap<DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>, Vec<SynapseIndex<Q::SynapseIndexQuant>>>,
    destination_to_synapse: AHashMap<DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>, Vec<SynapseIndex<Q::SynapseIndexQuant>>>,

    // Cached Data
    cache_valid_synapse_count: SynapseCount<Q::SynapseIndexQuant>,
    cache_invalid_synapse_count: SynapseCount<Q::SynapseIndexQuant>,

    /// Includes ranges of entire valid synapse blocks mapped to their cortical mapping. MAY INCLUDE individual dead synapses
    cache_valid_synapse_blocks: AHashMap<
        (DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>, DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>),
        IndexedDataTracker<Range<SynapseIndex<Q::SynapseIndexQuant>>, SynapseBundleIndex<Q::SynapseBundleIndexQuant>>
    >,

    /// Includes ranges of entire invalid synapse blocks. Does NOT notate singular dead synapses
    cache_invalid_synapse_blocks: RangeUintVector<SynapseIndex<Q::SynapseIndexQuant>, SynapseCount<Q::SynapseIndexQuant>>,

    _phantom: PhantomData<Q>,
}

impl<Q: NPUQuantization>
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    pub fn new(number_synapses_to_preallocate: SynapseCount<Q::SynapseIndexQuant>) -> Self {
        let count = number_synapses_to_preallocate.to_usize();
        Self {
            synapses_data: Vec::with_capacity(count),
            source_to_synapse: AHashMap::with_capacity(count),
            destination_to_synapse: AHashMap::with_capacity(count),
            cache_valid_synapse_count: SynapseCount::ZERO,
            cache_invalid_synapse_count: SynapseCount::ZERO,
            cache_valid_synapse_blocks: AHashMap::new(),
            cache_invalid_synapse_blocks: RangeUintVector::new(),
            _phantom: PhantomData,
        }
    }

    fn insert_valid_synapse_block_and_get_index(&mut self,
                                                synapse_block: Range<SynapseIndex<Q::SynapseIndexQuant>>,
                                                source_area: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>,
                                                destination_area: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>)
                                                -> SynapseBundleIndex<Q::SynapseBundleIndexQuant>
    {
        let key = (source_area, destination_area);
        let indexed_data_tracker = self.cache_valid_synapse_blocks.entry(key).or_insert(IndexedDataTracker::new());
        let index = indexed_data_tracker.insert_data_and_get_unique_index(synapse_block);
        index
    }

    //region Get Synapse Data

    fn get_total_synapse_count(&self) -> SynapseCount<Q::SynapseIndexQuant> {
        self.cache_valid_synapse_count + self.cache_invalid_synapse_count
    }

    /// Tries to get synapse at given index. Errors if index is invalid. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_data_at_synapse_index(&self, synapse_index: SynapseIndex<Q::SynapseIndexQuant>)
        -> Result<&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: (self.cache_valid_synapse_count.to_usize() + self.cache_invalid_synapse_count.to_usize()) as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if index and synapse valid
    fn get_valid_synapse_data_at_synapse_index(&self, synapse_index: SynapseIndex<Q::SynapseIndexQuant>)
        -> Result<&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>, FeagiNPUSynapseError>
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
    fn get_synapse_data_at_synapse_index_mut(&mut self, synapse_index: SynapseIndex<Q::SynapseIndexQuant>)
        -> Result<&mut NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get_mut(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: (self.cache_valid_synapse_count.to_usize() + self.cache_invalid_synapse_count.to_usize()) as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if index and synapse valid
    fn get_valid_synapse_data_at_synapse_index_mut(&mut self, synapse_index: SynapseIndex<Q::SynapseIndexQuant>)
        -> Result<&mut NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>, FeagiNPUSynapseError>
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
    fn get_synapse_indexes_from_source_neuron_index(&self, source_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        -> &[SynapseIndex<Q::SynapseIndexQuant>] {
        if let Some(val) = self.source_to_synapse.get(source_neuron_index) {
            return val.as_slice()
        }
        &[]
    }

    /// Gets all synapse indexes that have the following source_neuron_index. Returns empty if nothing found
    fn get_synapse_indexes_from_destination_neuron_index(&self, destination_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        ->  &[SynapseIndex<Q::SynapseIndexQuant>] {
        if let Some(val) = self.destination_to_synapse.get(destination_neuron_index) {
            return val.as_slice()
        }
        &[]
    }

    /// Gets synapse data from a source neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_source_neuron_index(&self, source_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>, NeuronCount<Q::NeuronIndexQuant>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_source_neuron_index(source_neuron_index);

        // TODO debug check only!
        for synapse_index in synapse_indexes {
            if synapse_index.to_usize() >= self.synapses_data.len() {
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
    fn get_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: &DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>, NeuronCount<Q::NeuronIndexQuant>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_destination_neuron_index(destination_neuron_index);

        // TODO debug check only!
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

    //endregion


}

impl<Q: NPUQuantization>
NonplasticSynapseAllocStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>,
                                                   source_cortical_data: &DimensionalNeuronCorticalData<Q>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>,
                                                   destination_cortical_data: &DimensionalNeuronCorticalData<Q>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<Q>)
                                                   -> Result<SynapseBundleIndex<Q::SynapseBundleIndexQuant>, FeagiStructuresError>{


        let (synapse_iterator, number_synapses) = neuron_mapping_executor.non_plastic_synapse_iterator(
            source_area_index.dimensional_type, source_cortical_data, source_neuron_flags,
            destination_area_index.dimensional_type, destination_cortical_data, destination_neuron_flags
        )?;

        // TODO debug check length is ok
        // TODO debug check that you dont spawn in dead synapses

        // Data
        let synapse_writing_region = self.cache_invalid_synapse_blocks.find_space(number_synapses); // TODO incorrect type! Shouldnt this take in count?

        let (synapse_bundle_index, extending) = match synapse_writing_region {
            None => {
                let synapse_writing_region = SynapseIndex::from_usize(self.synapses_data.len()) .. SynapseIndex::from_usize(self.synapses_data.len() + number_synapses.to_usize());
                let synapse_bundle_index = self.insert_valid_synapse_block_and_get_index(synapse_writing_region, source_area_index, destination_area_index);



                // Allocate at the end
                self.synapses_data.reserve(number_synapses.to_usize());
                self.source_to_synapse.reserve(number_synapses.to_usize());
                self.destination_to_synapse.reserve(number_synapses.to_usize());
                let starting_synapse_index = SynapseIndex::from_usize(self.synapses_data.len());
                for (local_index, synapse) in synapse_iterator.enumerate() {
                    let synapse_index = starting_synapse_index + SynapseIndex::from_usize(local_index);
                    self.source_to_synapse.entry(synapse.source_neuron_index).or_insert_with(|| vec![synapse_index]);
                    self.destination_to_synapse.entry(synapse.destination_neuron_index).or_insert_with(|| vec![synapse_index]);
                    self.synapses_data.push(synapse);
                }
                (synapse_bundle_index, true)
            }

            Some(synapse_writing_region) => {
                let starting_synapse_index = synapse_writing_region.start;
                let synapse_bundle_index = self.insert_valid_synapse_block_and_get_index(synapse_writing_region, source_area_index, destination_area_index);

                for (local_index, synapse) in synapse_iterator.enumerate() {
                    let synapse_index = starting_synapse_index + SynapseIndex::from_usize(local_index);
                    self.source_to_synapse.entry(synapse.source_neuron_index).or_insert_with(|| vec![synapse_index]);
                    self.destination_to_synapse.entry(synapse.destination_neuron_index).or_insert_with(|| vec![synapse_index]);
                    self.synapses_data[synapse_index.to_usize()] = synapse
                }

                (synapse_bundle_index, false)
            }
        };

        // Cache properties

        self.cache_valid_synapse_count += number_synapses;
        Ok(synapse_bundle_index)


    }
}

impl<Q: NPUQuantization>
NonplasticSynapseBaseStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    //region Get Connections
    fn get_nonplastic_synapse_data_from_source_neuron_index(&self,
                                                            source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>, NeuronCount<Q::NeuronIndexQuant>), FeagiNPUSynapseError> {
        self.get_synapse_data_from_source_neuron_index(&source_neuron_index)
    }


    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self,
                                                                 destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>)
        -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<Q::NeuronIndexQuant, Q::BurstDeltaQuant, Q::ValueQuant>>, NeuronCount<Q::NeuronIndexQuant>), FeagiNPUSynapseError> {
        self.get_synapse_data_from_destination_neuron_index(&destination_neuron_index)
    }

    //endregion

}

impl<Q: NPUQuantization>
Dim2DimSynapseAllocStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    fn remove_all_synapses_mappings_to_and_from_cortical_area(&mut self, area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn remove_all_synaptic_mappings_between_cortical_areas(&mut self, source_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>, destination_area_index: DimensionalTypedCorticalIndex<Q::CorticalIndexQuant>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }
}

impl<Q: NPUQuantization>
Dim2DimSynapseBaseStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    //region Get Connections

    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>) -> Result<&[DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>], FeagiNPUSynapseError> {
        todo!()
    }

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>) -> Result<&[DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>], FeagiNPUSynapseError> {
        todo!()
    }

    //endregion

    //region Sparse Synapse Invalidation

    fn invalidate_synapse_by_synapse_index(&mut self, synapse_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_by_synapse_indexes(&mut self, synapse_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>) -> Result<(), FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_source_neuron_index(&mut self, source_neurons_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>) -> Result<SynapseCount<Q::SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_source_neuron_indexes(&mut self, source_neurons_indexes: &[DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>]) -> Result<SynapseCount<Q::SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_destination_neuron_index(&mut self, destination_neurons_index: DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>) -> Result<SynapseCount<Q::SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    fn invalidate_synapses_with_destination_neuron_indexes(&mut self, destination_neurons_indexes: &[DimensionalTypedNeuronIndex<Q::NeuronIndexQuant>]) -> Result<SynapseCount<Q::SynapseIndexQuant>, FeagiNPUSynapseError> {
        todo!()
    }

    //endregion
}

impl<Q: NPUQuantization>
BaseSynapseAllocStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    fn free_unused_synapse_capacity(&mut self, spare_capacity_to_maintain: SynapseCount<Q::SynapseIndexQuant>) {
        let target = self.get_total_synapse_count().to_usize() + spare_capacity_to_maintain.to_usize();
        self.synapses_data.shrink_to(target);
        self.source_to_synapse.shrink_to_fit();
        self.destination_to_synapse.shrink_to_fit();
        // TODO delete empty vec keys?
    }
}

impl<Q: NPUQuantization>
BaseSynapseStorageTrait<Q> for
NonplasticDimensionalSynapseAllocRAMStorage<Q>
{
    const NUMBER_BYTES_PER_SYNAPSE: usize = 0; // TODO

    fn get_max_possible_synapse_index(&self) -> SynapseIndex<Q::SynapseIndexQuant> {
        SynapseIndex::MAX_VALUE
    }

    fn get_total_number_of_synapses(&self) -> &SynapseCount<Q::SynapseIndexQuant> {
        todo!()
    }

    fn get_total_number_of_valid_synapses(&self) -> &SynapseCount<Q::SynapseIndexQuant> {
        &self.cache_valid_synapse_count
    }

    fn get_total_number_of_invalid_synapses(&self) -> &SynapseCount<Q::SynapseIndexQuant> {
        &self.cache_invalid_synapse_count
    }
}
