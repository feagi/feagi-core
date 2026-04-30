use core::marker::PhantomData;
use feagi_structures::base_quantizable::QuantizableUIntType;
use feagi_structures::FeagiStructuresError;
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use crate::neuron::dimensional_neurons::neuron_models::DimensionalNeuronModelDataResizableTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

// TODO we should be smarter. We should reuse allocated data of the neuron data ITSELF

pub(crate) struct NeuronCollectionRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> {
    cortical_area_data: Vec<NeuronModel>,
    skipped_cortical_areas: Vec<CorticalAreaIndex<Q::CorticalIndexQuant>>,
    index_length: CorticalAreaIndex<Q::CorticalIndexQuant>,
    _phantom: PhantomData<DNQ>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> NeuronCollectionRam<Q, DNQ, NeuronModel>
{
    /// Creates a new empty neuron collection for ram
    pub fn new() -> Self {
        Self {
            cortical_area_data: Vec::new(),
            skipped_cortical_areas: Vec::new(),
            index_length: CorticalAreaIndex::ZERO,
            _phantom: PhantomData,
        }
    }

    /// Returns number of indexes skipped internally
    pub fn get_number_skipped_indexes(&self) -> usize {
        self.skipped_cortical_areas.len()
    }

    /// Inserts an element
    /// Returns the index where the element was inserted
    pub fn insert_existing_cortical_area_data_and_get_index(&mut self, value: NeuronModel) -> CorticalAreaIndex<Q::CorticalIndexQuant> {

        // TODO this is using a generic implementation, we should actually check for memory usage

        let latest_skipped = self.skipped_cortical_areas.pop();
        match latest_skipped {
            Some(index) => {
                self.cortical_area_data[index.to_usize()] = Some(value);
                index
            }
            None => {
                self.index_length += CorticalAreaIndex::ONE;
                self.cortical_area_data.push(Some(value));
                self.index_length - CorticalAreaIndex::ONE
            }
        }
    }

    /// Return an index to be able to be used later, invalidating data at that index as well
    pub fn return_index_and_invalidate_data(&mut self, returning_index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Result<(), FeagiStructuresError> {
        if returning_index >= self.index_length {
            // Not possible
            return Err(FeagiStructuresError::InvalidValue { context: "returned unique Index is larger than largest possible checked out!" })
        }

        // User tried checking in index 0 when nothing was checked out
        if self.index_length == CorticalAreaIndex::ZERO {
            return Err(FeagiStructuresError::InvalidValue { context: "Tried returning an index when no indexes were checked out!" })
        }

        if returning_index == self.index_length - CorticalAreaIndex::ONE {
            self.index_length -= CorticalAreaIndex::ONE;
        }
        else {
            // TODO DEBUG ONLY CHECK: Make sure we arent returning the same index multiple times!
            self.skipped_cortical_areas.push(returning_index);
        }

        self.cortical_area_data[returning_index.to_usize()] = None;
        Ok(())
    }

    /// Gets a reference to the element at the specified index (if valid)
    pub fn get(&self, index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Option<&NeuronModel> {
        self.cortical_area_data.get(index.to_usize()).and_then(|opt| *opt.as_ref())
    }

    /// Gets a mutable reference to the element at the specified index (if valid)
    pub fn get_mut(&mut self, index: CorticalAreaIndex<Q::CorticalIndexQuant>) -> Option<&mut NeuronModel> {
        self.cortical_area_data.get_mut(index.to_usize()).and_then(|opt| *opt.as_mut())
    }

}

