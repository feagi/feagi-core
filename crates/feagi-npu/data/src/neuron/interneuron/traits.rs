use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::neuron::data::InterneuronFlag;
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralMembranePotential, NeuronVoxelCoordinate, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
use crate::neuron::FeagiNPUDataError;
// TODO some of these error checks may be excessive, as we may already check for it at runtime!
// TODO maybe this number of generics is a bit excessive....


pub trait InterCorticalIndexData<NeuronIndex, CorticalIndex, CoordQuant, NMP, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
where

    NeuronIndex: QuantizableUInt,
    CorticalIndex: QuantizableUInt,
    NMP: NeuralMembranePotential,
    CoordQuant: QuantizableUInt,
    LC: LeakCoefficient,
    RP: RefractoryPeriod,
    RC: RefractoryCountdown,
    EX: Excitability,
    CFC: ConsecutiveFireCountdown,
    CFL: ConsecutiveFireLimit,
    SPC: SnoozePeriodCountdown,
    SPL: SnoozePeriodLimit,

// % synaptic attractivity

// TODO get_neuron_ids should return an iterator
// TODO parallel iterators? perhaps as an optional trait

{
    fn get_total_number_of_neurons(&self) -> NeuronIndex;

    fn get_cortical_index(&self, neuron_id: NeuronIndex) -> Result<CorticalIndex, FeagiNPUDataError>;

    fn get_neuron_ids(&self, cortical_index: CorticalIndex) -> Result<impl Iterator<Item = &NeuronIndex>, FeagiNPUDataError>;

    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    //region Individual neuron Properties

    //region Membrane Potential
    fn get_neuron_membrane_potentials_slice(&self) -> &[NMP];

    fn get_neuron_membrane_potentials_slice_mut(&mut self) -> &mut [NMP];
    //endregion

    //region Neuron Voxel Coordinates

    // No setting functions since no neurons can move voxels in static environments

    fn get_neuron_voxel_coordinates_slice(&self) -> &[NeuronVoxelCoordinate<CoordQuant>];
    //endregion

    //region Threshold
    fn get_neuron_thresholds_slice(&self) -> &[NMP];

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [NMP];
    //endregion

    //region Threshold Limit
    fn get_neuron_threshold_limits_slice(&self) -> &[NMP];

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [NMP];
    //endregion

    //region Leak Coefficient
    fn get_neuron_leak_coefficients_slice(&self) -> &[LC];

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [LC];
    //endregion

    //region Neuron Flags
    fn get_neuron_flags_slice(&self) -> &[InterneuronFlag];

    fn get_neuron_flags_slice_mut(&mut self) -> &mut [InterneuronFlag];
    //endregion

    //region Refractory Countdown
    fn get_neuron_refractory_countdowns_slice(&self) -> &[RC];

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [RC];
    //endregion

    //region Fire Count
    fn get_consecutive_fire_count_slice(&self) -> &[CFC];

    fn get_consecutive_fire_count_slice_mut(&mut self) -> &mut [CFC];
    //endregion

    //region Fire Limit
    fn get_consecutive_fire_limit_slice(&self) -> &[u16];

    fn get_consecutive_fire_limit_slice_mut(&mut self) -> &mut [u16];
    //endregion

    //region Snooze Period Countdown
    fn get_snooze_period_countdown_slice(&self) -> &[SPC];

    fn get_snooze_period_countdown_slice_mut(&mut self) -> &mut [SPC];
    //endregion

    //region Snooze Period Limit
    fn get_snooze_period_limit_slice(&self) -> &[SPL];

    fn get_snooze_period_limit_slice_mut(&mut self) -> &mut [SPL];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // since this affects neuron count, the setting functions are only in the alloc extension

    fn get_number_of_neurons_per_voxel_descriptor(&self, cortical_index: CorticalIndex) -> Result<u8, FeagiNPUDataError>;

    fn get_number_of_neurons_per_voxel_descriptor_slice(&self) -> &[u8];
    //endregion

    //endregion
}

#[cfg(feature = "alloc")]
pub trait InterCorticalIndexDataAlloc<NeuronIndex, CorticalIndex, CoordQuant, NMP, LC, RP, RC, EX, CFC, CFL, SPC, SPL>: InterCorticalIndexData<NeuronIndex, CorticalIndex, CoordQuant, NMP, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
where
    NeuronIndex: QuantizableUInt,
    CorticalIndex: QuantizableUInt,
    NMP: NeuralMembranePotential,
    CoordQuant: QuantizableUInt,
    LC: LeakCoefficient,
    RP: RefractoryPeriod,
    RC: RefractoryCountdown,
    EX: Excitability,
    CFC: ConsecutiveFireCountdown,
    CFL: ConsecutiveFireLimit,
    SPC: SnoozePeriodCountdown,
    SPL: SnoozePeriodLimit,
    // % synaptic attractivity
{
    //region Individual neuron Properties

    //region Neuron Voxel Coordinates
    fn set_neuron_voxel_coordinate(&mut self, neuron_id: NeuronIndex, coordinate: NeuronVoxelCoordinate<CoordQuant>) -> Result<(), FeagiNPUDataError>;

    fn get_neuron_voxel_coordinates_slice_mut(&mut self) -> &mut [NeuronVoxelCoordinate<CoordQuant>];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // get functions in base trait

    fn set_number_of_neurons_per_voxel_descriptor(&mut self, cortical_index: CorticalIndex, number_neurons_per_voxel: NumberNeuronsPerVoxel) -> Result<(), FeagiNPUDataError>;

    fn get_number_of_neurons_per_voxel_descriptor_slice_mut(&mut self) -> &mut [NumberNeuronsPerVoxel];
    //endregion

    //endregion

    // TODO functions for adding / removing neurons
}

// TODO par iterator trait