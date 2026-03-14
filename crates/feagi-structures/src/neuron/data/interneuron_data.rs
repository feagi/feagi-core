use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralPotentialValue, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
use crate::neuron::FeagiNeuronError;
// TODO some of these error checks may be excessive, as we may already check for it at runtime!
// TODO Shouldnt we have a Percentage Base type?
// TODO maybe this number of generics is a bit excessive....

pub trait InterCorticalIndexData<NeuronIndex, CorticalIndex, Coord, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
where

    NeuronIndex: QuantizableUInt,
    CorticalIndex: QuantizableUInt,
    NPV: NeuralPotentialValue,
    Coord: CorticalIndexVoxelCoordinate,
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

    //region CorticalIndex Membrane Potential
    fn get_neuron_membrane_potential(&self, neuron_id: NeuronIndex) -> Result<NPV, FeagiNeuronError>;

    fn set_neuron_membrane_potential(&mut self, neuron_id: NeuronIndex, potential: NPV) -> Result<(), FeagiNeuronError>;

    fn get_neuron_membrane_potentials_slice(&self) -> &[NPV];

    fn get_neuron_membrane_potentials_slice_mut(&mut self) -> &mut [NPV];
    //endregion

    //region CorticalIndex Threshold
    fn get_neuron_threshold(&self, neuron_id: NeuronIndex) -> Result<NPV, FeagiNeuronError>;

    fn set_neuron_threshold(&mut self, neuron_id: NeuronIndex, threshold: NPV) -> Result<(), FeagiNeuronError>;

    fn get_neuron_thresholds_slice(&self) -> &[NPV];

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [NPV];
    //endregion

    //region CorticalIndex Threshold Limit
    fn get_neuron_threshold_limit(&self, neuron_id: NeuronIndex) -> Result<NPV, FeagiNeuronError>; // 0 is no limit

    fn set_neuron_threshold_limit(&mut self, neuron_id: NeuronIndex, threshold_limit: NPV) -> Result<(), FeagiNeuronError>;

    fn get_neuron_threshold_limits_slice(&self) -> &[NPV];

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [NPV];
    //endregion

    //region CorticalIndex Leak Coefficient
    fn get_neuron_leak_coefficient(&self, neuron_id: NeuronIndex) -> Result<LC, FeagiNeuronError>;

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NeuronIndex, leak_coefficient: LC) -> Result<(), FeagiNeuronError>;

    fn get_neuron_leak_coefficients_slice(&self) -> &[LC];

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [LC];
    //endregion

    //region CorticalIndex Validity
    fn get_neuron_validity(&self, neuron_id: NeuronIndex) -> Result<bool, FeagiNeuronError>;

    fn set_neuron_validity(&self, neuron_id: NeuronIndex, is_valid: bool) -> Result<() , FeagiNeuronError>;

    fn get_neuron_validity_slice(&self) -> &[bool];

    fn get_neuron_validity_slice_mut(&mut self) -> &mut [bool];
    //endregion

    //region CorticalIndex Refractory Countdown
    fn get_neuron_refractory_countdown(&self, neuron_id: NeuronIndex) -> Result<RC, FeagiNeuronError>;

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NeuronIndex, countdown: RC) -> Result<(), FeagiNeuronError>;
    
    fn get_neuron_refractory_countdowns_slice(&self) -> &[RC];

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [RC];
    //endregion

    //region Consecutive Fire Count
    fn get_consecutive_fire_count(&self, neuron_id: NeuronIndex) -> Result<CFC, FeagiNeuronError>;

    fn set_consecutive_fire_count(&mut self, neuron_id: NeuronIndex, countdown: CFC) -> Result<(), FeagiNeuronError>;

    fn get_consecutive_fire_count_slice(&self) -> &[CFC];

    fn get_consecutive_fire_count_slice_mut(&mut self) -> &mut [CFC];
    //endregion

    //region Consecutive Fire Limit
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

    //region Cortical Refractory Period
    fn get_cortical_refractory_period(&self, cortical_index: CorticalIndex) -> Result<RP, FeagiNeuronError>;

    fn set_cortical_refractory_period(&mut self, cortical_index: CorticalIndex, refractory_period: RP) -> Result<(), FeagiNeuronError>;

    fn get_cortical_refractory_periods_slice(&self) -> &[RP];

    fn get_cortical_refractory_periods_slice_mut(&mut self) -> &mut [RP];
    //endregion

    //region Cortical Excitability
    fn get_cortical_excitability(&self, cortical_index: CorticalIndex) -> Result<EX, FeagiNeuronError>;

    fn set_cortical_excitability(&mut self, cortical_index: CorticalIndex, excitability: EX) -> Result<(), FeagiNeuronError>;

    fn get_cortical_excitability_slice(&self) -> &[EX];

    fn get_cortical_excitability_slice_mut(&mut self) -> &mut [EX];
    //endregion

    //endregion













}

