use crate::cortical_area::descriptors::{CorticalAreaCount, CorticalAreaIndex};
use crate::neuron::data::{InterNeuronData, NeuronFlag};
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralPotentialValue, NeuronCount, NeuronId, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
use crate::neuron::FeagiNeuronError;

// NOTE: neuron_count_per_cortical_index: As interneuron count and cortical area count cannot change,
// we init this array with neurons grouped in xyz (incrementing in order) by cortical area,
// those groupings being ordered by cortical are size biggest to smallest (in terms of neuron count)
// such that the most common areas to hit (by chance) are at the start

pub struct InterNeuronDataStatic<NC, NID, CAC, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL, const NEURON_COUNT: NC, const CORTICAL_AREA_COUNT: CAC>
where
    NC: NeuronCount,
    NID: NeuronId,
    CAC: CorticalAreaCount,
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
    neuron_count_per_cortical_index: [NC; CORTICAL_AREA_COUNT], // See note at top

    // Per Neuron
    neuron_cortical_area_index: [CAI; NEURON_COUNT],
    neuron_membrane_potential: [NPV; NEURON_COUNT],
    neuron_threshold: [NPV; NEURON_COUNT],
    neuron_threshold_limit: [NPV; NEURON_COUNT],
    neuron_leak_coefficient: [LC; NEURON_COUNT],
    neuron_flags: [NeuronFlag; NEURON_COUNT],
    neuron_refractory_countdown:[RC; NEURON_COUNT],
    consecutive_fire_count: [CFC; NEURON_COUNT],
    consecutive_fire_limit: [CFL; NEURON_COUNT],
    snooze_period_countdown: [SPC; NEURON_COUNT],
    snooze_period_limit: [SPL; NEURON_COUNT],

    // Per Cortical Area
    cortical_refractory_period: [RP; CORTICAL_AREA_COUNT],
    cortical_excitability: [EX; CORTICAL_AREA_COUNT],
    neurons_per_voxel: [NC; CORTICAL_AREA_COUNT],

}

impl<NC, NID, CAC, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL, const NEURON_COUNT: NC, const CORTICAL_AREA_COUNT: CAC> InterNeuronData<NC, NID, CAC, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> for InterNeuronDataStatic<NC, NID, CAC, CAI, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL, const NEURON_COUNT: NC, const CORTICAL_AREA_COUNT: CAC> {
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