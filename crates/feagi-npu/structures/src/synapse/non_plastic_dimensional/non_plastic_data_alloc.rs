use ahash::AHashMap;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NumberNeuronsPerVoxel};
use feagi_structures::useful_structs::InvalidatableVector;
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{NPUNeuronIndex, NPUSynapseIndex, SynapseBundleIndex, SynapseCount};
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


pub struct NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    // Data
    synapses_data: Vec<NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>,
    source_to_synapse: AHashMap<NPUNeuronIndex<NeuronIndexQuant>, Vec<NPUSynapseIndex<SynapseIndexQuant>>>,
    destination_to_synapse: AHashMap<NPUNeuronIndex<NeuronIndexQuant>, Vec<NPUSynapseIndex<SynapseIndexQuant>>>,

    // Cached Data
    cache_valid_synapse_count: SynapseCount<SynapseIndexQuant>,
    cache_invalid_synapse_count: SynapseCount<SynapseIndexQuant>,
    /// Includes ranges of entire valid synapse blocks mapped to their cortical mapping. MAY INCLUDE individual dead synapses
    cache_valid_synapse_blocks: AHashMap<
        (CorticalAreaIndex<CorticalIndexQuant>, CorticalAreaIndex<CorticalIndexQuant>),
        InvalidatableVector<core::ops::Range<SynapseIndexQuant>>
    >,
    /// Includes ranges of entire invalid synapse blocks. Does NOT include singular dead synapses
    cache_invalid_synapse_blocks: Vec<core::ops::Range<SynapseIndexQuant>>,
}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
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
            source_to_synapse: AHashMap::with_capacity(count),
            destination_to_synapse: AHashMap::with_capacity(count),
            cache_valid_synapse_count: SynapseCount(0),
            cache_invalid_synapse_count: SynapseCount(0),
            cache_valid_synapse_blocks: AHashMap::new(),
            cache_invalid_synapse_blocks: Vec::new(),
        }
    }

    fn insert_valid_synapse_block_and_get_index(&mut self,
                                                synapse_block: core::ops::Range<NPUSynapseIndex<SynapseIndexQuant>>,
                                                source_area: CorticalAreaIndex<SynapseIndexQuant>,
                                                destination_area: CorticalAreaIndex<SynapseIndexQuant>)
        -> SynapseBundleIndex<SynapseBundleIndexQuant>
    {
        let key = (source_area, destination_area);
        if !self.cache_valid_synapse_blocks.contains_key(&key) {
            self.cache_valid_synapse_blocks.insert(key, InvalidatableVector::new());
        }
        let block_vec = self.cache_valid_synapse_blocks.get_mut(&key);
        let index = block_vec.insert(synapse_block);
        SynapseBundleIndex::from_usize(index)
    }

    //region Get Synapse Data

    /// Tries to get synapse at given index. Errors if index is invalid. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_data_at_synapse_index(&self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: self.get_total_number_of_synapses().to_usize() as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if index and synapse valid
    fn get_valid_synapse_data_at_synapse_index(&self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let possible_valid = self.get_synapse_at_synapse_index(synapse_index)?;
        if !possible_valid.is_valid() {
            return Err(FeagiNPUSynapseError::SynapseIndexIsInvalid {
                context: "Expected valid synapse at index!",
                given_synapse_index: synapse_index.to_usize() as u32 })
        }
        Ok(possible_valid)
    }

    /// Tries to get synapse at given index. Errors if index is invalid. DOES NOT CHECK IF SYNAPSE IS VALID
    fn get_synapse_data_at_synapse_index_mut(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let index_synapse: usize = synapse_index.to_usize();
        self.synapses_data.get_mut(index_synapse).ok_or_else(
            || FeagiNPUSynapseError::SynapseIndexOutOfRange {
                context: "Given synapse index is not in range of non-plastic dimensional synapse ram storage!",
                given_synapse_index: synapse_index.to_usize() as u32,
                range: self.get_total_number_of_synapses().to_usize() as u32}
        )
    }

    /// Tries to get a valid synapse at given index. Checks if index and synapse valid
    fn get_valid_synapse_data_at_synapse_index_mut(&mut self, synapse_index: NPUSynapseIndex<SynapseIndexQuant>) -> Result<&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>, FeagiNPUSynapseError>
    {
        let possible_valid = self.get_synapse_at_synapse_index_mut(synapse_index)?;
        if !possible_valid.is_valid() {
            return Err(FeagiNPUSynapseError::SynapseIndexIsInvalid {
                context: "Expected valid synapse at index!",
                given_synapse_index: synapse_index.to_usize() as u32 })
        }
        Ok(possible_valid)
    }

    /// Gets all synapse indexes that have the following source_neuron_index. Returns empty if nothing found
    fn get_synapse_indexes_from_source_neuron_index(&self, source_neuron_index: &NPUNeuronIndex<NeuronIndexQuant>) -> &[NPUSynapseIndex<SynapseIndexQuant>] {
        if let Some(val) = self.source_to_synapse.get(source_neuron_index) {
            return val.to_slice()
        }
        &[]
    }

    /// Gets all synapse indexes that have the following source_neuron_index. Returns empty if nothing found
    fn get_synapse_indexes_from_destination_neuron_index(&self, destination_neuron_index: &NPUNeuronIndex<NeuronIndexQuant>) ->  &[NPUSynapseIndex<SynapseIndexQuant>] {
        if let Some(val) = self.destination_to_synapse.get(destination_neuron_index) {
            return val.to_slice()
        }
        &[]
    }

    /// Gets synapse data from a source neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_source_neuron_index(&self, source_neuron_index: &NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_source_neuron_index(source_neuron_index);
        let iterator = synapse_indexes.iter().map(|synapse_index| self.get_synapse_data_at_synapse_index(*synapse_index).map_err(
            || FeagiNPUSynapseError::InternalError {context: "Source neuron index pointed to invalid synapse index! Something went wrong!"}
        )?);
        Ok((iterator, NeuronCount::from_usize(synapse_indexes.len())))
    }

    /// Gets synapse data from a destination neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: &NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_destination_neuron_index(destination_neuron_index);
        let iterator = synapse_indexes.iter().map(|synapse_index| self.get_synapse_data_at_synapse_index(*synapse_index).map_err(
            || FeagiNPUSynapseError::InternalError {context: "Destination neuron index pointed to invalid synapse index! Something went wrong!"}
        )?);
        Ok((iterator, NeuronCount::from_usize(synapse_indexes.len())))
    }

    /// Gets mut synapse data from a source neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_source_neuron_index_mut(&self, source_neuron_index: &NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_source_neuron_index(source_neuron_index);
        let iterator = synapse_indexes.iter_mut().map(|synapse_index| self.get_synapse_data_at_synapse_index(*synapse_index).map_err(
            || FeagiNPUSynapseError::InternalError {context: "Source neuron index pointed to invalid synapse index! Something went wrong!"}
        )?);
        Ok((iterator, NeuronCount::from_usize(synapse_indexes.len())))
    }

    /// Gets mut synapse data from a destination neuron index as an iterator, and the number of them. DOES NOT FILTER OUT INVALID SYNAPSES
    fn get_synapse_data_from_destination_neuron_index_mut(&self, destination_neuron_index: &NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {
        let synapse_indexes = self.get_synapse_indexes_from_destination_neuron_index(destination_neuron_index);
        let iterator = synapse_indexes.iter_mut().map(|synapse_index| self.get_synapse_data_at_synapse_index(*synapse_index).map_err(
            || FeagiNPUSynapseError::InternalError {context: "Destination neuron index pointed to invalid synapse index! Something went wrong!"}
        )?);
        Ok((iterator, NeuronCount::from_usize(synapse_indexes.len())))
    }

    //endregion


}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
NonplasticSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    fn add_synapses_mapping_between_cortical_areas(&mut self,
                                                   source_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   source_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                                   source_neuron_flags: &[NeuronFlag],
                                                   source_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                                                   source_neuron_density: NumberNeuronsPerVoxel,
                                                   destination_area_index: CorticalAreaIndex<CorticalIndexQuant>,
                                                   destination_neuron_indexes: core::ops::Range<NPUNeuronIndex<NeuronIndexQuant>>,
                                                   destination_neuron_flags: &[NeuronFlag],
                                                   destination_cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                                                   destination_neuron_density: NumberNeuronsPerVoxel,
                                                   neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>)
        -> Result<SynapseBundleIndex<SynapseBundleIndexQuant>, FeagiNPUSynapseError>{


        let (synapse_iterator, number_synapses) = neuron_mapping_executor.non_plastic_synapse_iterator(
            source_neuron_indexes, source_neuron_flags, source_cortical_dimensions, source_neuron_density,
            destination_neuron_indexes, destination_neuron_flags, destination_cortical_dimensions, destination_neuron_density
        )?;

        // TODO check length is ok


        // Data
        let number_synapses: usize = number_synapses.to_usize();
        let starting_synapse_index: NPUSynapseIndex<SynapseIndexQuant>;

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
                self.source_to_synapse.insert(synapse.source_neuron_index.clone(), synapse_index);
                self.destination_to_synapse.insert(synapse.destination_neuron_index.clone(), synapse_index);
                self.synapses_data.push(synapse)
            }
        }

        // Cache properties
        let synapse_range = starting_synapse_index..(starting_synapse_index + NPUSynapseIndex::from_usize(number_synapses));

        self.cache_valid_synapse_count += number_synapses;
        Ok(self.insert_valid_synapse_block_and_get_index(synapse_range, source_area_index, destination_area_index))

    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
NonplasticSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{

    //region Get Connections
    fn get_nonplastic_synapse_data_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError>
    {
        self.get_synapse_data_from_source_neuron_index(&source_neuron_index)
    }

    fn get_nonplastic_synapse_data_from_source_neuron_index_mut(&mut self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {
        todo!()
    }

    fn get_nonplastic_synapse_data_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {

    }

    fn get_nonplastic_synapse_data_from_destination_neuron_index_mut(&mut self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<(impl Iterator<Item=&mut NonPlasticSynapseFull<NeuronIndexQuant, BurstDeltaQuant, ValueQuant>>, NeuronCount<NeuronIndexQuant>), FeagiNPUSynapseError> {
        todo!()
    }


    //endregion

}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
Dim2DimSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
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

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
Dim2DimSynapseBaseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    //region Get Connections

    fn get_destination_neuron_indexes_from_source_neuron_index(&self, source_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&[NPUNeuronIndex<NeuronIndexQuant>], FeagiNPUSynapseError> {
        todo!()
    }

    fn get_source_neuron_indexes_from_destination_neuron_index(&self, destination_neuron_index: NPUNeuronIndex<NeuronIndexQuant>) -> Result<&[NPUNeuronIndex<NeuronIndexQuant>], FeagiNPUSynapseError> {
        todo!()
    }

    //endregion

    //region Sparse Synapse Invalidation

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
    //endregion
}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
BaseSynapseAllocStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
    CorticalIndexQuant: QuantizableUIntType,
    CoordQuant: QuantizableUIntType,
    BurstDeltaQuant: QuantizableUIntType,
    BurstIndexQuant: QuantizableUIntType,
    ValueQuant: QuantizableValueType,
    PercentageQuant: QuantizablePercentType,
{
    fn free_unused_synapse_capacity(&mut self, spare_capacity_to_maintain: SynapseCount<SynapseIndexQuant>) -> SynapseCount<SynapseIndexQuant> {
        self.synapses_data.shrink_to(self.get_total_number_of_synapses() + spare_capacity_to_maintain);
        self.source_to_synapse.shrink_to_fit();
        self.destination_to_synapse.shrink_to_fit();
        // TODO delete empty vec keys?
        *self.get_total_number_of_synapses()
    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
BaseSynapseStorageTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
where
    NeuronIndexQuant: QuantizableUIntType,
    SynapseIndexQuant: QuantizableUIntType,
    SynapseBundleIndexQuant: QuantizableUIntType,
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

