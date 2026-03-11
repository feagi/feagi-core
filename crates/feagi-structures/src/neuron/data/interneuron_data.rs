use crate::cortical_area::descriptors::CorticalAreaIndex;
use crate::neuron::{FeagiNeuronError};
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralPotentialValue, NeuronCount, NeuronId, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
// TODO some of these error checks may be excessive, as we may already check for it at runtime!
// TODO Shouldnt we have a Percentage Base type?

pub trait InterNeuronData<NC, NID, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
where
    NC: NeuronCount,
    NID: NeuronId,
    CAI: CorticalAreaIndex,
    NPV: NeuralPotentialValue,
    LC: LeakCoefficient,
    RP: RefractoryPeriod,
    RC: RefractoryCountdown,
    EX: Excitability,
    CFC: ConsecutiveFireCountdown,
    CFL: ConsecutiveFireLimit,
    SPC: SnoozePeriodCountdown,
    SPL: SnoozePeriodLimit,
{
    fn get_total_number_of_neurons(&self) -> NC;

    fn get_cortical_index(&self, neuron_id: NID) -> Result<CAI, FeagiNeuronError>;

    fn get_neuron_ids(&self, cortical_index: CAI) -> Result<&[NID], FeagiNeuronError>;

    //region Individual neuron Properties

    fn get_neuron_membrane_potential(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError>;

    fn set_neuron_membrane_potential(&mut self, neuron_id: NID, potential: NPV) -> Result<(), FeagiNeuronError>;


    fn get_neuron_threshold(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError>;

    fn set_neuron_threshold(&mut self, neuron_id: NID, threshold: NPV) -> Result<(), FeagiNeuronError>;


    fn get_neuron_threshold_limit(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError>; // 0 is no limit

    fn set_neuron_threshold_limit(&mut self, neuron_id: NID, threshold_limit: NPV) -> Result<(), FeagiNeuronError>;


    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    fn get_neuron_leak_coefficient(&self, neuron_id: NID) -> Result<LC, FeagiNeuronError>;

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NID, leak_coefficient: LC) -> Result<(), FeagiNeuronError>;


    fn get_neuron_validity(&self, neuron_id: NID) -> Result<bool, FeagiNeuronError>;

    fn set_neuron_validity(&self, neuron_id: NID, is_valid: bool) -> Result<() , FeagiNeuronError>;


    fn get_neuron_refractory_countdown(&self, neuron_id: NID) -> Result<RC, FeagiNeuronError>;

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NID, countdown: RC) -> Result<(), FeagiNeuronError>;



    fn get_consecutive_fire_count(&self, neuron_id: NID) -> Result<CFC, FeagiNeuronError>;

    fn set_consecutive_fire_count(&mut self, neuron_id: NID, countdown: CFC) -> Result<(), FeagiNeuronError>;

    fn get_consecutive_fire_limit(&self, neuron_id: CFL) -> Result<u16, FeagiNeuronError>;

    fn set_consecutive_fire_limit(&mut self, neuron_id: CFL, countdown: u16) -> Result<(), FeagiNeuronError>;


    fn get_snooze_period_countdown(&self, neuron_id: NID) -> Result<SPC, FeagiNeuronError>;

    fn set_snooze_period_countdown(&mut self, neuron_id: NID, countdown: SPC) -> Result<(), FeagiNeuronError>;

    fn get_snooze_period_limit(&self, neuron_id: NID) -> Result<SPL, FeagiNeuronError>;

    fn set_snooze_period_limit(&mut self, neuron_id: NID, countdown: SPL) -> Result<(), FeagiNeuronError>;

    //endregion


    //region Individual Cortical Area Properties

    fn get_cortical_refractory_period(&self, cortical_index: CAI) -> Result<RP, FeagiNeuronError>;

    fn set_cortical_refractory_period(&mut self, cortical_index: CAI, refractory_period: RP) -> Result<(), FeagiNeuronError>;


    fn get_cortical_excitability(&self, cortical_index: CAI) -> Result<EX, FeagiNeuronError>;

    fn set_cortical_excitability(&mut self, cortical_index: CAI, excitability: EX) -> Result<(), FeagiNeuronError>;


    fn get_mp_charge_accumulation_enabled(&self, cortical_index: CAI) -> Result<bool, FeagiNeuronError>;

    fn set_mp_charge_accumulation_enabled(&mut self, cortical_index: CAI, is_mp_charge_accumulation_enabled: bool) -> Result<(), FeagiNeuronError>;

    //endregion













}

