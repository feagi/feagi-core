use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralMembranePotential, NeuronVoxelCoordinate, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
use crate::neuron::FeagiNeuronError;
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

{
    fn get_total_number_of_neurons(&self) -> NeuronIndex;

    fn get_cortical_index(&self, neuron_id: NeuronIndex) -> Result<CorticalIndex, FeagiNeuronError>;

    fn get_neuron_ids(&self, cortical_index: CorticalIndex) -> Result<&[NeuronIndex], FeagiNeuronError>;

    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    //region Individual neuron Properties

    //region Membrane Potential
    fn get_neuron_membrane_potential(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNeuronError>;

    fn set_neuron_membrane_potential(&mut self, neuron_id: NeuronIndex, potential: NMP) -> Result<(), FeagiNeuronError>;

    fn get_neuron_membrane_potentials_slice(&self) -> &[NMP];

    fn get_neuron_membrane_potentials_slice_mut(&mut self) -> &mut [NMP];
    //endregion

    //region Neuron Voxel Coordinates

    // No setting functions since no neurons can move voxels in static environments

    fn get_neuron_voxel_coordinate(&self, neuron_id: NeuronIndex) -> Result<NeuronVoxelCoordinate<CoordQuant>, FeagiNeuronError>;

    fn get_neuron_voxel_coordinates_slice(&self) -> &[NeuronVoxelCoordinate<CoordQuant>];
    //endregion

    //region Threshold
    fn get_neuron_threshold(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNeuronError>;

    fn set_neuron_threshold(&mut self, neuron_id: NeuronIndex, threshold: NMP) -> Result<(), FeagiNeuronError>;

    fn get_neuron_thresholds_slice(&self) -> &[NMP];

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [NMP];
    //endregion

    //region Threshold Limit
    fn get_neuron_threshold_limit(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNeuronError>; // 0 is no limit

    fn set_neuron_threshold_limit(&mut self, neuron_id: NeuronIndex, threshold_limit: NMP) -> Result<(), FeagiNeuronError>;

    fn get_neuron_threshold_limits_slice(&self) -> &[NMP];

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [NMP];
    //endregion

    //region Leak Coefficient
    fn get_neuron_leak_coefficient(&self, neuron_id: NeuronIndex) -> Result<LC, FeagiNeuronError>;

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NeuronIndex, leak_coefficient: LC) -> Result<(), FeagiNeuronError>;

    fn get_neuron_leak_coefficients_slice(&self) -> &[LC];

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [LC];
    //endregion

    //region Validity
    fn get_neuron_validity(&self, neuron_id: NeuronIndex) -> Result<bool, FeagiNeuronError>;

    fn set_neuron_validity(&self, neuron_id: NeuronIndex, is_valid: bool) -> Result<() , FeagiNeuronError>;

    fn get_neuron_validity_slice(&self) -> &[bool];

    fn get_neuron_validity_slice_mut(&mut self) -> &mut [bool];
    //endregion

    //region Refractory Countdown
    fn get_neuron_refractory_countdown(&self, neuron_id: NeuronIndex) -> Result<RC, FeagiNeuronError>;

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NeuronIndex, countdown: RC) -> Result<(), FeagiNeuronError>;
    
    fn get_neuron_refractory_countdowns_slice(&self) -> &[RC];

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [RC];
    //endregion

    //region Fire Count
    fn get_consecutive_fire_count(&self, neuron_id: NeuronIndex) -> Result<CFC, FeagiNeuronError>;

    fn set_consecutive_fire_count(&mut self, neuron_id: NeuronIndex, countdown: CFC) -> Result<(), FeagiNeuronError>;

    fn get_consecutive_fire_count_slice(&self) -> &[CFC];

    fn get_consecutive_fire_count_slice_mut(&mut self) -> &mut [CFC];
    //endregion

    //region Fire Limit
    fn get_consecutive_fire_limit(&self, neuron_id: CFL) -> Result<u16, FeagiNeuronError>;

    fn set_consecutive_fire_limit(&mut self, neuron_id: CFL, countdown: u16) -> Result<(), FeagiNeuronError>;

    fn get_consecutive_fire_limit_slice(&self) -> &[u16];

    fn get_consecutive_fire_limit_slice_mut(&mut self) -> &mut [u16];
    //endregion

    //region Snooze Period Countdown
    fn get_snooze_period_countdown(&self, neuron_id: NeuronIndex) -> Result<SPC, FeagiNeuronError>;

    fn set_snooze_period_countdown(&mut self, neuron_id: NeuronIndex, countdown: SPC) -> Result<(), FeagiNeuronError>;

    fn get_snooze_period_countdown_slice(&self) -> &[SPC];

    fn get_snooze_period_countdown_slice_mut(&mut self) -> &mut [SPC];
    //endregion

    //region Snooze Period Limit
    fn get_snooze_period_limit(&self, neuron_id: NeuronIndex) -> Result<SPL, FeagiNeuronError>;

    fn set_snooze_period_limit(&mut self, neuron_id: NeuronIndex, countdown: SPL) -> Result<(), FeagiNeuronError>;
    
    fn get_snooze_period_limit_slice(&self) -> &[SPL];

    fn get_snooze_period_limit_slice_mut(&mut self) -> &mut [SPL];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // since this affects neuron count, the setting functions are only in the alloc extension

    fn get_number_of_neurons_per_voxel_descriptor(&self, cortical_index: CorticalIndex) -> Result<u8, FeagiNeuronError>;

    fn get_number_of_neurons_per_voxel_descriptor_slice(&self) -> &[u8];
    //endregion

    //endregion
}





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
    fn set_neuron_voxel_coordinate(&mut self, neuron_id: NeuronIndex, coordinate: NeuronVoxelCoordinate<CoordQuant>) -> Result<(), FeagiNeuronError>;

    fn get_neuron_voxel_coordinates_slice_mut(&mut self) -> &mut [NeuronVoxelCoordinate<CoordQuant>];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // get functions in base trait

    fn set_number_of_neurons_per_voxel_descriptor(&mut self, cortical_index: CorticalIndex, number_neurons_per_voxel: u8) -> Result<(), FeagiNeuronError>; // cannot be embeded

    fn get_number_of_neurons_per_voxel_descriptor_slice_mut(&mut self) -> &mut [u8]; // cannot be embeded
    //endregion
    
    //endregion
    
    // TODO functions for adding / removing neurons
}
