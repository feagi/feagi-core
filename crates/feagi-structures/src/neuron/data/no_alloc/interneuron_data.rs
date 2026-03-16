use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::neuron::data::interneuron_data::InterCorticalIndexData;
use crate::neuron::data::NeuronFlag;
use crate::neuron::descriptors::{ConsecutiveFireCountdown, ConsecutiveFireLimit, Excitability, LeakCoefficient, NeuralMembranePotential, NeuronVoxelCoordinate, RefractoryCountdown, RefractoryPeriod, SnoozePeriodCountdown, SnoozePeriodLimit};
use crate::neuron::FeagiNeuronError;

// NOTE: neuron_count_per_cortical_index: As interneuron count and cortical area count cannot change,
// we init this array with neurons grouped in xyz (incrementing in order) by cortical area,
// those groupings being ordered by cortical are size biggest to smallest (in terms of neuron count)
// such that the most common areas to hit (by chance) are at the start

pub struct InterNeuronDataStatic<
    NeuronIndex,
    CorticalIndex,
    CoordQuant,
    NMP,
    LC,
    RP,
    RC,
    EX,
    CFC,
    CFL,
    SPC,
    SPL,
    const NEURON_COUNT: NeuronIndex,
    const CORTICAL_AREA_COUNT: CorticalIndex,
>
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
{
    neuron_count_per_cortical_index: [NeuronIndex; CORTICAL_AREA_COUNT], // See note at top

    // Per Neuron
    neuron_cortical_area_index: [CorticalIndex; NEURON_COUNT],
    neuron_membrane_potential: [NMP; NEURON_COUNT],
    neuron_voxel_coordinate: [NeuronVoxelCoordinate<CoordQuant>; NEURON_COUNT],
    neuron_threshold: [NMP; NEURON_COUNT],
    neuron_threshold_limit: [NMP; NEURON_COUNT],
    neuron_leak_coefficient: [LC; NEURON_COUNT],
    neuron_flags: [NeuronFlag; NEURON_COUNT],
    neuron_refractory_countdown: [RC; NEURON_COUNT],
    consecutive_fire_count: [CFC; NEURON_COUNT],
    consecutive_fire_limit: [CFL; NEURON_COUNT],
    snooze_period_countdown: [SPC; NEURON_COUNT],
    snooze_period_limit: [SPL; NEURON_COUNT],

    // Per Cortical Area
    cortical_refractory_period: [RP; CORTICAL_AREA_COUNT],
    cortical_excitability: [EX; CORTICAL_AREA_COUNT],
    neurons_per_voxel: [NeuronIndex; CORTICAL_AREA_COUNT],
}

impl<
    NeuronIndex,
    CorticalIndex,
    CoordQuant,
    NMP,
    LC,
    RP,
    RC,
    EX,
    CFC,
    CFL,
    SPC,
    SPL,
    const NEURON_COUNT: NeuronIndex,
    const CORTICAL_AREA_COUNT: CorticalIndex,
> InterCorticalIndexData<
    NeuronIndex,
    CorticalIndex,
    CoordQuant,
    NMP,
    LC,
    RP,
    RC,
    EX,
    CFC,
    CFL,
    SPC,
    SPL,
> for InterNeuronDataStatic<
    NeuronIndex,
    CorticalIndex,
    CoordQuant,
    NMP,
    LC,
    RP,
    RC,
    EX,
    CFC,
    CFL,
    SPC,
    SPL,
    NEURON_COUNT,
    CORTICAL_AREA_COUNT,
>
{
    fn get_total_number_of_neurons(&self) -> NeuronIndex {
        self.neuron_cortical_area_index.len() as NeuronIndex  // any array works
    }

    fn get_cortical_index(&self, neuron_id: NeuronIndex) -> Result<CorticalIndex, FeagiNeuronError> {
        self.neuron_cortical_area_index.get(neuron_id as usize).ok_or_else(|| FeagiNeuronError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_ids(&self, cortical_index: CorticalIndex) -> Result<&[NeuronIndex], FeagiNeuronError> {
        todo!()
    }

    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    //region Individual neuron Properties

    //region Membrane Potential
    fn get_neuron_membrane_potential(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNeuronError> {
        self.neuron_membrane_potential.get(neuron_id as usize).ok_or_else(|| FeagiNeuronError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_membrane_potential(&mut self, neuron_id: NeuronIndex, potential: NMP) -> Result<(), FeagiNeuronError> {
        if let Some(neuron_membrane_potential) = self.neuron_cortical_area_index.get(neuron_id as usize) {
            *neuron_membrane_potential = potential;
            return Ok(());
        }
        Err(FeagiNeuronError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_membrane_potentials_slice(&self) -> &[NMP] {
        &self.neuron_membrane_potential
    }

    fn get_neuron_membrane_potentials_slice_mut(&mut self) -> &mut [NMP] {
        &mut self.neuron_membrane_potential
    }
    //endregion

    //region Neuron Voxel Coordinates

    // No setting functions since no neurons can move voxels in static environments
    fn get_neuron_voxel_coordinate(&self, neuron_id: NeuronIndex) -> Result<NeuronVoxelCoordinate<CoordQuant>, FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_voxel_coordinates_slice(&self) -> &[NeuronVoxelCoordinate<CoordQuant>] {
        todo!()
    }
    //endregion

    //region Threshold
    fn get_neuron_threshold(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_threshold(&mut self, neuron_id: NeuronIndex, threshold: NMP) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_thresholds_slice(&self) -> &[NMP] {
        todo!()
    }

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [NMP] {
        todo!()
    }
    //endregion

    //region Threshold Limit
    fn get_neuron_threshold_limit(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_threshold_limit(&mut self, neuron_id: NeuronIndex, threshold_limit: NMP) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_threshold_limits_slice(&self) -> &[NMP] {
        todo!()
    }

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [NMP] {
        todo!()
    }
    //endregion

    //region Leak Coefficient
    fn get_neuron_leak_coefficient(&self, neuron_id: NeuronIndex) -> Result<LC, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NeuronIndex, leak_coefficient: LC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_leak_coefficients_slice(&self) -> &[LC] {
        todo!()
    }

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [LC] {
        todo!()
    }
    //endregion

    //region Validity
    fn get_neuron_validity(&self, neuron_id: NeuronIndex) -> Result<bool, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_validity(&self, neuron_id: NeuronIndex, is_valid: bool) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_validity_slice(&self) -> &[bool] {
        todo!()
    }

    fn get_neuron_validity_slice_mut(&mut self) -> &mut [bool] {
        todo!()
    }
    //endregion

    //region Refractory Countdown
    fn get_neuron_refractory_countdown(&self, neuron_id: NeuronIndex) -> Result<RC, FeagiNeuronError> {
        todo!()
    }

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NeuronIndex, countdown: RC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_neuron_refractory_countdowns_slice(&self) -> &[RC] {
        todo!()
    }

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [RC] {
        todo!()
    }
    //endregion

    //region Fire Count
    fn get_consecutive_fire_count(&self, neuron_id: NeuronIndex) -> Result<CFC, FeagiNeuronError> {
        todo!()
    }

    fn set_consecutive_fire_count(&mut self, neuron_id: NeuronIndex, countdown: CFC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_consecutive_fire_count_slice(&self) -> &[CFC] {
        todo!()
    }

    fn get_consecutive_fire_count_slice_mut(&mut self) -> &mut [CFC] {
        todo!()
    }
    //endregion

    //region Fire Limit
    fn get_consecutive_fire_limit(&self, neuron_id: CFL) -> Result<u16, FeagiNeuronError> {
        todo!()
    }

    fn set_consecutive_fire_limit(&mut self, neuron_id: CFL, countdown: u16) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_consecutive_fire_limit_slice(&self) -> &[u16] {
        todo!()
    }

    fn get_consecutive_fire_limit_slice_mut(&mut self) -> &mut [u16] {
        todo!()
    }
    //endregion

    //region Snooze Period Countdown
    fn get_snooze_period_countdown(&self, neuron_id: NeuronIndex) -> Result<SPC, FeagiNeuronError> {
        todo!()
    }

    fn set_snooze_period_countdown(&mut self, neuron_id: NeuronIndex, countdown: SPC) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_snooze_period_countdown_slice(&self) -> &[SPC] {
        todo!()
    }

    fn get_snooze_period_countdown_slice_mut(&mut self) -> &mut [SPC] {
        todo!()
    }
    //endregion

    //region Snooze Period Limit
    fn get_snooze_period_limit(&self, neuron_id: NeuronIndex) -> Result<SPL, FeagiNeuronError> {
        todo!()
    }

    fn set_snooze_period_limit(&mut self, neuron_id: NeuronIndex, countdown: SPL) -> Result<(), FeagiNeuronError> {
        todo!()
    }

    fn get_snooze_period_limit_slice(&self) -> &[SPL] {
        todo!()
    }

    fn get_snooze_period_limit_slice_mut(&mut self) -> &mut [SPL] {
        todo!()
    }
    //endregion

    //endregion

    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // since this affects neuron count, the setting functions are only in the alloc extension
    fn get_number_of_neurons_per_voxel_descriptor(&self, cortical_index: CorticalIndex) -> Result<u8, FeagiNeuronError> {
        todo!()
    }

    fn get_number_of_neurons_per_voxel_descriptor_slice(&self) -> &[u8] {
        todo!()
    }
    //endregion

    //endregion
}