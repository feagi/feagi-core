
// NOTE: In alloc contexts, dont be too overspecific with quantization

use std::ops::Range;
use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaIndex;
use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::{NeuronCount, NeuronMembranePotential, NumberNeuronsPerVoxel};
use crate::connectome::{ConnectomeAllocTrait, ConnectomeBaseTrait};
use crate::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
use crate::executors::neuron_property_executors::NeuronFireThresholdExecutor;
use crate::FeagiNPUStructureError;
use crate::neuron::base_dimension_traits::DimensionalAllocStorageTrait;
use crate::neuron::dimensional_neurons::DimensionalNeuronAllocRAMStorage;
use crate::neuron::dimensional_neurons::shared_funcs_and_structs::DimensionalNeuronDataFromCorticalArea;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{BurstDelta, BurstGlobalIndex, FireThreshold, FireThresholdLimit, LeakCoefficient, NPUNeuronIndex, NeuronExcitability, SynapseBundleIndex, SynapseCount};
use crate::synapse::non_plastic_dimensional::NonplasticDimensionalSynapseAllocRAMStorage;
use crate::synapse::non_plastic_dimensional::traits::NonplasticSynapseAllocStorageTrait;

pub struct ConnectomeAllocRam<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
    // Neurons
    neurons_dimensional: DimensionalNeuronAllocRAMStorage<NeuronIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>,
    
    // Synapses
    synapse_nonplastic: NonplasticDimensionalSynapseAllocRAMStorage<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
}


impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
ConnectomeAllocRam<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
    pub fn new(
        preallocated_dimensional_neuron_count: NeuronCount<NeuronIndexQuant>,
        preallocated_nonplastic_dimensional_synapse_count: SynapseCount<SynapseIndexQuant>
    ) -> Self {
        Self {
            neurons_dimensional: DimensionalNeuronAllocRAMStorage::new(preallocated_dimensional_neuron_count),
            synapse_nonplastic: NonplasticDimensionalSynapseAllocRAMStorage::new(preallocated_nonplastic_dimensional_synapse_count),
        }
    }
}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
ConnectomeAllocTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
ConnectomeAllocRam<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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

    //region Cortical Areas

    //region Dimensional Neuron Cortical Areas

    fn create_dimensional_neuron_cortical_area_with_default_neurons(&mut self, 
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>, 
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel) 
        -> Result<(CorticalAreaIndex<CorticalIndexQuant>, Range<NPUNeuronIndex<NeuronIndexQuant>>), FeagiNPUStructureError> {
        self.neurons_dimensional.create_cortical_area_with_default_neurons(
            cortical_area_dimensions,
            neurons_per_voxel
        ).map_err(|err| err.into())
    }

    fn create_dimensional_neuron_cortical_area_with_uniform_neurons(&mut self, 
                                                                    cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>, 
                                                                    neurons_per_voxel: NumberNeuronsPerVoxel, 
                                                                    neuron_global_burst_index_of_last_firing: BurstGlobalIndex<BurstIndexQuant>,
                                                                    neuron_membrane_potential: NeuronMembranePotential<ValueQuant>, 
                                                                    neuron_fire_threshold: FireThreshold<ValueQuant>, 
                                                                    neuron_leak_coefficient: LeakCoefficient<PercentageQuant>, 
                                                                    neuron_refractory_countdown: BurstDelta<BurstDeltaQuant>, 
                                                                    neuron_consecutive_fire_count: BurstDelta<BurstDeltaQuant>, 
                                                                    cortical_excitability: NeuronExcitability<PercentageQuant>, 
                                                                    cortical_refractory_period_limit: BurstDelta<BurstDeltaQuant>, 
                                                                    cortical_fire_threshold_limit: FireThresholdLimit<ValueQuant>, 
                                                                    cortical_consecutive_fire_limit: BurstDelta<BurstDeltaQuant>, 
                                                                    cortical_is_mp_charge_accumulation_enabled: bool, 
                                                                    cortical_is_mp_driven_psp_enabled: bool)
        -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError> {
        todo!()
    }

    fn create_dimensional_neuron_cortical_area_with_individualized_neurons(&mut self, cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>, neurons_per_voxel: NumberNeuronsPerVoxel, neuron_data: DimensionalNeuronDataFromCorticalArea<NeuronIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>) -> Result<CorticalAreaIndex<CorticalIndexQuant>, FeagiNPUStructureError> {
        todo!()
    }

    fn resize_dimensional_neuron_cortical_area<'a>(&mut self, cortical_area_dimensions: NeuronVoxelDimensions<CoordQuant>, neurons_per_voxel: NumberNeuronsPerVoxel, cortical_index: CorticalAreaIndex<CorticalIndexQuant>, presynaptic_dimensional_mappings: &Vec<(CorticalAreaIndex<CorticalIndexQuant>, DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>)>, postsynaptic_dimensional_mappings: &Vec<(CorticalAreaIndex<CorticalIndexQuant>, DimensionCorticalAreaType, &'a impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>)>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn delete_dimensional_neuron_cortical_area(&mut self, cortical_index: CorticalAreaIndex<CorticalIndexQuant>) -> Result<Range<NPUNeuronIndex<NeuronIndexQuant>>, FeagiNPUStructureError> {
        todo!()
    }

    //endregion

    //endregion

    //region Synapses

    //region dimensional area to dimensional area

    fn disconnect_all_synapses_from_dimensional_area_to_dimensional_area(&mut self, source_index: CorticalAreaIndex<CorticalIndexQuant>, source_dimensional_type: DimensionCorticalAreaType, destination_index: CorticalAreaIndex<CorticalIndexQuant>, destination_dimensional_type: DimensionCorticalAreaType) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn add_nonplastic_connection_from_dimensional_area_to_dimensional_area(&mut self, source_area_index: CorticalAreaIndex<CorticalIndexQuant>, destination_area_index: CorticalAreaIndex<CorticalIndexQuant>, neuron_mapping_executor: &impl NonPlasticCorticalMappingDefinitionExecutor<NeuronIndexQuant, SynapseIndexQuant, CoordQuant, CorticalIndexQuant, BurstDeltaQuant, ValueQuant>) -> Result<SynapseBundleIndex<SynapseBundleIndexQuant>, FeagiNPUStructureError> {
        
        
        
        let synapse_bundle_index = self.synapse_nonplastic.add_synapses_mapping_between_cortical_areas(
            source_area_index,
            source_neuron_indexes,
            source_neuron_flags,
            source_cortical_dimensions,
            source_neuron_density,
            destination_area_index,
            destination_neuron_indexes,
            destination_neuron_flags: &[NeuronFlag],
            destination_cortical_dimensions,
            destination_neuron_density,
            neuron_mapping_executor).map_err(|err| err.into())?;
    }

    //endregion

    //endregion

    //region Housekeeping

    fn free_unused_neuron_capacity(&mut self) {
        todo!()
    }

    fn free_unused_synapse_capacity(&mut self) {
        todo!()
    }

    fn free_unused_cortical_area_capacity(&mut self) {
        todo!()
    }
    //endregion
}

impl<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
ConnectomeBaseTrait<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant> for
ConnectomeAllocRam<NeuronIndexQuant, SynapseIndexQuant, SynapseBundleIndexQuant, CorticalIndexQuant, CoordQuant, BurstDeltaQuant, BurstIndexQuant, ValueQuant, PercentageQuant>
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
    fn process_burst(&mut self, burst_index: BurstGlobalIndex<BurstIndexQuant>) {
        todo!()
    }

    //region Set Neuron Properties
    fn set_dimensional_neuron_fire_thresholds(&mut self, cortical_area_index: CorticalAreaIndex<CorticalIndexQuant>, executor: &impl NeuronFireThresholdExecutor<ValueQuant, CoordQuant>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn set_dimensional_neuron_leak_coefficients(&mut self, cortical_area_index: CorticalAreaIndex<CorticalIndexQuant>, executor: &impl NeuronFireThresholdExecutor<ValueQuant, CoordQuant>) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }
    //endregion

    //region Utility and HouseKeeping
    fn prune_dead_synapses_and_neurons(&mut self) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }

    fn defragment_connectome(&mut self) -> Result<(), FeagiNPUStructureError> {
        todo!()
    }
    //endregion
}






