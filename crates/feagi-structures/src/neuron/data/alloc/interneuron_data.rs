use ahash::AHashMap;
use crate::cortical_area::descriptors::CorticalAreaIndex;
use crate::neuron::data::InterNeuronData;
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralPotentialValue, NeuronCount, NeuronId, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
use crate::neuron::FeagiNeuronError;

// TODO Rayon? Could the trait perhaps implement some sort of iterator support for rayon?
// TODO return mut refs to indexes?
// TODO slice structure with lifetimes to this?

pub struct InterNeuronDataDynamic<NC, NID, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
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
    total_number_of_neurons: NC,
    neuron_2_cortical: AHashMap<NID, CAI>,
    cortical_2_neuron: AHashMap<CAI, Vec<NID>>,

    // Per Neuron
    neuron_membrane_potential: Vec<NPV>,
    neuron_threshold: Vec<NPV>,
    neuron_threshold_limit: Vec<NPV>,
    neuron_leak_coefficient: Vec<LC>,
    neuron_validity: Vec<bool>, // TODO bit this!
    neuron_refractory_countdown: Vec<RC>,
    consecutive_fire_count: Vec<CFC>,
    consecutive_fire_limit: Vec<CFL>,
    snooze_period_countdown: Vec<SPC>,
    snooze_period_limit: Vec<SPL>,

    // Per Cortical Area
    cortical_refractory_period: Vec<RP>,
    cortical_excitability: Vec<EX>,
    mp_charge_accumulation_enabled: Vec<bool>,
}

impl<NC, NID, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> InterNeuronData<NC, NID, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> for InterNeuronDataDynamic<NC, NID, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> {
    fn get_total_number_of_neurons(&self) -> NC {
        todo!()
    }

    fn get_cortical_index(&self, neuron_id: NID) -> Result<CAI, FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_ids(&self, cortical_index: CAI) -> Result<&[NID], FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_membrane_potential(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_membrane_potential(&mut self, neuron_id: NID, potential: NPV) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_threshold(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_threshold(&mut self, neuron_id: NID, threshold: NPV) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_threshold_limit(&self, neuron_id: NID) -> Result<NPV, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_threshold_limit(&mut self, neuron_id: NID, threshold_limit: NPV) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_leak_coefficient(&self, neuron_id: NID) -> Result<LC, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NID, leak_coefficient: LC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_validity(&self, neuron_id: NID) -> Result<bool, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_validity(&self, neuron_id: NID, is_valid: bool) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_refractory_countdown(&self, neuron_id: NID) -> Result<RC, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NID, countdown: RC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_consecutive_fire_count(&self, neuron_id: NID) -> Result<CFC, FeagiNeuronError> {
        todo!()
    }

    fn set_consecutive_fire_count(&mut self, neuron_id: NID, countdown: CFC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_consecutive_fire_limit(&self, neuron_id: CFL) -> Result<u16, FeagiNeuronError> {
        todo!()
    }

    fn set_consecutive_fire_limit(&mut self, neuron_id: CFL, countdown: u16) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_snooze_period_countdown(&self, neuron_id: NID) -> Result<SPC, FeagiNeuronError> {
        todo!()
    }

    fn set_snooze_period_countdown(&mut self, neuron_id: NID, countdown: SPC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_snooze_period_limit(&self, neuron_id: NID) -> Result<SPL, FeagiNeuronError> {
        todo!()
    }

    fn set_snooze_period_limit(&mut self, neuron_id: NID, countdown: SPL) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_cortical_refractory_period(&self, cortical_index: CAI) -> Result<RP, FeagiNeuronError> {
        todo!()
    }

    fn set_cortical_refractory_period(&mut self, cortical_index: CAI, refractory_period: RP) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_cortical_excitability(&self, cortical_index: CAI) -> Result<EX, FeagiNeuronError> {
        todo!()
    }

    fn set_cortical_excitability(&mut self, cortical_index: CAI, excitability: EX) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_mp_charge_accumulation_enabled(&self, cortical_index: CAI) -> Result<bool, FeagiNeuronError> {
        todo!()
    }

    fn set_mp_charge_accumulation_enabled(&mut self, cortical_index: CAI, is_mp_charge_accumulation_enabled: bool) -> Result<(), FeagiNeuronError> {
        todo!()
    }
}


// TODO separate dynamic implementations