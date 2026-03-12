use crate::cortical_area::descriptors::{CorticalAreaCount, CorticalAreaIndex};
use crate::neuron::{FeagiNeuronError};
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralPotentialValue, NeuronCount, NeuronId, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
// TODO some of these error checks may be excessive, as we may already check for it at runtime!
// TODO Shouldnt we have a Percentage Base type?
// TODO maybe this number of generics is a bit excessive....

pub trait InterNeuronData<NC, NID, CAC, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
where
    NC: NeuronCount,
    NID: NeuronId,
    CAC: CorticalAreaCount,
    CAI: CorticalAreaIndex,
    NPV: NeuralPotentialValue,
    LC: LeakCoefficient,
    RP: RefractoryPeriod,
    RC: RefractoryCountdown,
    EX: Excitability, // %
    CFC: ConsecutiveFireCountdown,
    CFL: ConsecutiveFireLimit,
    SPC: SnoozePeriodCountdown,
    SPL: SnoozePeriodLimit,

// % synaptic attractivity

{
    fn get_total_number_of_neurons(&self) -> NC;

    fn get_cortical_index(&self, neuron_id: NID) -> Result<CAI, FeagiNeuronError>;

    fn get_neuron_ids(&self, cortical_index: CAI) -> Result<&[NID], FeagiNeuronError>;

    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    //region Individual neuron Properties

    //region Neuron Membrane Potential
    fn get_neuron_membrane_potential(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError>;

    fn set_neuron_membrane_potential(&mut self, neuron_id: NID, potential: NPV) -> Result<(), FeagiNeuronError>;

    fn get_neuron_membrane_potentials_slice(&self) -> &[NPV];

    fn get_neuron_membrane_potentials_slice_mut(&mut self) -> &mut [NPV];
    //endregion

    //region Neuron Threshold
    fn get_neuron_threshold(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError>;

    fn set_neuron_threshold(&mut self, neuron_id: NID, threshold: NPV) -> Result<(), FeagiNeuronError>;

    fn get_neuron_thresholds_slice(&self) -> &[NPV];

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [NPV];
    //endregion

    //region Neuron Threshold Limit
    fn get_neuron_threshold_limit(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError>; // 0 is no limit

    fn set_neuron_threshold_limit(&mut self, neuron_id: NID, threshold_limit: NPV) -> Result<(), FeagiNeuronError>;

    fn get_neuron_threshold_limits_slice(&self) -> &[NPV];

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [NPV];
    //endregion

    //region Neuron Leak Coefficient
    fn get_neuron_leak_coefficient(&self, neuron_id: NID) -> Result<LC, FeagiNeuronError>;

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NID, leak_coefficient: LC) -> Result<(), FeagiNeuronError>;

    fn get_neuron_leak_coefficients_slice(&self) -> &[LC];

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [LC];
    //endregion

    //region Neuron Validity
    fn get_neuron_validity(&self, neuron_id: NID) -> Result<bool, FeagiNeuronError>;

    fn set_neuron_validity(&self, neuron_id: NID, is_valid: bool) -> Result<() , FeagiNeuronError>;

    fn get_neuron_validity_slice(&self) -> &[bool];

    fn get_neuron_validity_slice_mut(&mut self) -> &mut [bool];
    //endregion

    //region Neuron Refractory Countdown
    fn get_neuron_refractory_countdown(&self, neuron_id: NID) -> Result<RC, FeagiNeuronError>;

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NID, countdown: RC) -> Result<(), FeagiNeuronError>;
    
    fn get_neuron_refractory_countdowns_slice(&self) -> &[RC];

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [RC];
    //endregion

    //region Consecutive Fire Count
    fn get_consecutive_fire_count(&self, neuron_id: NID) -> Result<CFC, FeagiNeuronError>;

    fn set_consecutive_fire_count(&mut self, neuron_id: NID, countdown: CFC) -> Result<(), FeagiNeuronError>;

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
    fn get_snooze_period_countdown(&self, neuron_id: NID) -> Result<SPC, FeagiNeuronError>;

    fn set_snooze_period_countdown(&mut self, neuron_id: NID, countdown: SPC) -> Result<(), FeagiNeuronError>;

    fn get_snooze_period_countdown_slice(&self) -> &[SPC];

    fn get_snooze_period_countdown_slice_mut(&mut self) -> &mut [SPC];
    //endregion

    //region Snooze Period Limit
    fn get_snooze_period_limit(&self, neuron_id: NID) -> Result<SPL, FeagiNeuronError>;

    fn set_snooze_period_limit(&mut self, neuron_id: NID, countdown: SPL) -> Result<(), FeagiNeuronError>;
    
    fn get_snooze_period_limit_slice(&self) -> &[SPL];

    fn get_snooze_period_limit_slice_mut(&mut self) -> &mut [SPL];
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Cortical Refractory Period
    fn get_cortical_refractory_period(&self, cortical_index: CAI) -> Result<RP, FeagiNeuronError>;

    fn set_cortical_refractory_period(&mut self, cortical_index: CAI, refractory_period: RP) -> Result<(), FeagiNeuronError>;

    fn get_cortical_refractory_periods_slice(&self) -> &[RP];

    fn get_cortical_refractory_periods_slice_mut(&mut self) -> &mut [RP];
    //endregion

    //region Cortical Excitability
    fn get_cortical_excitability(&self, cortical_index: CAI) -> Result<EX, FeagiNeuronError>;

    fn set_cortical_excitability(&mut self, cortical_index: CAI, excitability: EX) -> Result<(), FeagiNeuronError>;

    fn get_cortical_excitability_slice(&self) -> &[EX];

    fn get_cortical_excitability_slice_mut(&mut self) -> &mut [EX];
    //endregion

    //endregion













}

