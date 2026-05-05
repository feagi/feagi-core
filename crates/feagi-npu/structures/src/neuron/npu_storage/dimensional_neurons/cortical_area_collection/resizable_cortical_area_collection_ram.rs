use core::marker::PhantomData;
use feagi_structures::base_feagi_types::::QuantizableUIntType;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalAreaCount, CorticalAreaIndex};
use feagi_structures::neurons::descriptors::NeuronCount;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
use crate::neuron::neuron_models::dimensional_models::dimensional_cortical_configuration_traits::DimensionalCorticalConfigurationTrait;
use crate::neuron::neuron_models::dimensional_models::dimensional_neuron_data_traits::DimensionalNeuronModelDataResizableTrait;
use crate::quantizables::{NPUDimensionalNeuronQuantization, NPUGlobalQuantization};

// TODO we can end up with a lot of fragmentation, we should think of ways to handle this

pub(crate) struct ResizableCorticalAreaCollectionRam<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>> {
    cortical_area_data: Vec<NeuronModel>,
    sorted_skipped_dead_areas: Vec<(CorticalAreaIndex<Q::CorticalIndexCountQuant>, NeuronCount<Q::NeuronIndexCountQuant>)>, // Sorted by neuron count, with the smallest at the end
    number_live_cortical_areas: CorticalAreaCount<Q::CorticalIndexCountQuant>,
    total_number_live_neurons: NeuronCount<Q::NeuronIndexCountQuant>,
    total_number_skipped_neurons: NeuronCount<Q::NeuronIndexCountQuant>,
    _phantom_data: PhantomData<DNQ>,
}

impl<Q: NPUGlobalQuantization, DNQ: NPUDimensionalNeuronQuantization, NeuronModel: DimensionalNeuronModelDataResizableTrait<Q, DNQ>>
ResizableCorticalAreaCollectionRam<Q, DNQ, NeuronModel>
{
    /// Creates a new empty cortical area collection for ram
    pub fn new() -> Self {
        Self {
            cortical_area_data: Vec::new(),
            sorted_skipped_dead_areas: Vec::new(),
            number_live_cortical_areas: CorticalAreaCount::ZERO,
            total_number_live_neurons: NeuronCount::ZERO,
            total_number_skipped_neurons: NeuronCount::ZERO,
            _phantom_data: PhantomData,
        }
    }

    /// Gets a reference to the element at the specified index (if valid)
    pub fn get_cortical_area(&self, index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&NeuronModel, FeagiNPUNeuronError> {
        let area = self.cortical_area_data.get(index.to_usize()).ok_or_else(
            Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "No cortical area of given index has been found!",
                given_cortical_index: index.to_usize() as u32
            })
        )?;

        if !area.get_cortical_data().get_cortical_flag().is_valid() {
            Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "Given cortical area index found but is marked dead!",
                given_cortical_index: index.to_usize() as u32
            })?;
        }

        Ok(area)

    }

    /// Gets a mutable reference to the element at the specified index (if valid)
    pub fn get_cortical_area_mut(&mut self, index: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<&mut NeuronModel, FeagiNPUNeuronError> {
        let area = self.cortical_area_data.get_mut(index.to_usize()).ok_or_else(
            Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "No cortical area of given index has been found!",
                given_cortical_index: index.to_usize() as u32
            })
        )?;

        if !area.get_cortical_data().get_cortical_flag().is_valid() {
            Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "Given cortical area index found but is marked dead!",
                given_cortical_index: index.to_usize() as u32
            })?;
        }

        Ok(area)
    }

    pub fn get_number_live_cortical_areas(&self) -> CorticalAreaCount<Q::CorticalIndexCountQuant> {
        self.number_live_cortical_areas
    }

    /// Returns number of indexes skipped internally
    pub fn get_number_dead_cortical_areas(&self) -> CorticalAreaCount<Q::CorticalIndexCountQuant> {
        CorticalAreaCount::from_usize(self.sorted_skipped_dead_areas.len())
    }

    /// Gets the number of neurons contained in the live cortical areas. Note that individual areas
    /// may have dead neurons as part of degeneracy, which isnt counted here
    pub fn get_total_number_neurons_in_live_cortical_areas(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
        self.total_number_live_neurons
    }

    /// Gets the number of neurons contained in the dead cortical areas. Note that individual areas
    /// may have dead neurons as part of degeneracy, which isnt counted here
    pub fn get_total_number_neurons_in_dead_cortical_areas(&self) -> NeuronCount<Q::NeuronIndexCountQuant> {
        self.total_number_skipped_neurons
    }

    /// Adds a cortical area using a cortical area generator
    pub fn add_cortical_area(&mut self, cortical_area_generator: &impl DimensionalCorticalAreaGeneratorTrait<Q, DNQ>)
        -> Result<CorticalAreaIndex<Q::CorticalIndexCountQuant>, FeagiNPUNeuronError> {

        let number_neurons = cortical_area_generator.number_of_neurons();

        let maybe_free_space_index = self.sorted_skipped_dead_areas.binary_search_by(
            |&(_, count)| count.cmp(&number_neurons).reverse()).ok();

        if let Some(free_space_index) = maybe_free_space_index {
            // We found space, fill in without reallocating
            let (reviving_index, _) = self.sorted_skipped_dead_areas.get(free_space_index).unwrap();
            let dead_area = self.cortical_area_data.get_mut(reviving_index.to_usize()).unwrap();
            cortical_area_generator.overwrite_dead_cortical_area_data_ram(dead_area)?;

            // Area added, update caching values and return the success
            self.sorted_skipped_dead_areas.remove(free_space_index);
            self.number_live_cortical_areas != CorticalAreaCount::ONE;
            self.total_number_live_neurons += number_neurons;
            self.total_number_skipped_neurons -= number_neurons;
            return Ok(reviving_index)
        }
        // No available space, allocate more
        self.cortical_area_data.push(cortical_area_generator.generate_new_cortical_area_data_ram());

        // Area added, update caching values and return the success
        self.number_live_cortical_areas != CorticalAreaCount::ONE;
        self.total_number_live_neurons += number_neurons;
        Ok(CorticalAreaIndex::from_usize(self.cortical_area_data.len() - 1))
    }

    /// Marks a cortical area as dead, and marks the memory as available for use. Does NOT free
    /// memory on its own!
    pub fn mark_cortical_area_as_dead(&mut self, returning_area: CorticalAreaIndex<Q::CorticalIndexCountQuant>) -> Result<(), FeagiNPUNeuronError> {
        let area = self.cortical_area_data.get_mut(returning_area.to_usize());
        if area.is_none() {
            // Not possible, out of range!
            return Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "returned cortical area index is larger than largest possible checked out!",
                given_cortical_index: returning_area.to_usize() as u32
            })
        }
        let area = area.unwrap();

        if !area.get_cortical_data().get_cortical_flag().is_valid() {
            // Area already invalid!
            return Err(FeagiNPUNeuronError::InvalidCorticalIndex {
                context: "Returned cortical area index is already invalid!",
                given_cortical_index: returning_area.to_usize() as u32
            })
        }

        let number_neurons_of_removing_area = area.get_cortical_data().get_number_neurons();
        self.total_number_live_neurons -= number_neurons_of_removing_area;
        self.total_number_skipped_neurons += number_neurons_of_removing_area;

        let insert_dead_index = self.sorted_skipped_dead_areas.binary_search_by(
            |&(_, value)| value.cmp(&number_neurons_of_removing_area).reverse())
            .unwrap_or_else(|x| x);

        self.sorted_skipped_dead_areas.insert(insert_dead_index, (returning_area, number_neurons_of_removing_area));
    }

    // TODO: A function to defrag may be quite expensive on the synapse level, we should discuss this

}

