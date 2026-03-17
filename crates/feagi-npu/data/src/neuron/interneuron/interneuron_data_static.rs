
// NOTE: neuron_count_per_cortical_index: As interneuron count and cortical area count cannot change,
// we init this array with neurons grouped in xyz (incrementing in order) by cortical area,
// those groupings being ordered by cortical are size biggest to smallest (in terms of neuron count)
// such that the most common areas to hit (by chance) are at the start

pub struct InterneuronDataStatic<
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
    neuron_end_index_and_neurons_per_voxel_per_cortical_index: [(NeuronIndex, NumberNeuronsPerVoxel); CORTICAL_AREA_COUNT], // See note at top

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
> for InterneuronDataStatic<
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

    fn get_cortical_index(&self, neuron_id: NeuronIndex) -> Result<CorticalIndex, FeagiNPUDataError> {
        self.neuron_cortical_area_index.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_ids(&self, cortical_index: CorticalIndex) -> Result<impl Iterator<Item = &NeuronIndex>, FeagiNPUDataError> {
        let ending_neuron_index: usize =  self.neuron_cortical_area_index
            .get(cortical_index as usize).ok_or_else(|| return Err(
                FeagiNPUDataError::CorticalIndexOutOfRange{
                    given_cortical_index: cortical_index as usize,
                    range: self.neuron_cortical_area_index.len()
                }
        )) as usize;

        let starting_neuron_index: usize = if cortical_index == CorticalIndex::zero() {
            0
        } else {
            self.neuron_end_index_and_neurons_per_voxel_per_cortical_index.get((cortical_index - CorticalIndex::one()) as usize).0 as usize
        };
        Ok(starting_neuron_index..ending_neuron_index)
    }

    //TODO Set Fire threshold increment -> we need a function to be able to set thresholds in a gradiant across a cortical area

    //region Individual neuron Properties

    //region Membrane Potential
    fn get_neuron_membrane_potential(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNPUDataError> {
        self.neuron_membrane_potential.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_membrane_potential(&mut self, neuron_id: NeuronIndex, potential: NMP) -> Result<(), FeagiNPUDataError> {
        if let Some(neuron_membrane_potential) = self.neuron_membrane_potential.get_mut(neuron_id as usize) {
            *neuron_membrane_potential = potential;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
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
    fn get_neuron_voxel_coordinate(&self, neuron_id: NeuronIndex) -> Result<NeuronVoxelCoordinate<CoordQuant>, FeagiNPUDataError> {
        self.neuron_voxel_coordinate.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_voxel_coordinates_slice(&self) -> &[NeuronVoxelCoordinate<CoordQuant>] {
        &self.neuron_voxel_coordinate
    }
    //endregion

    //region Threshold
    fn get_neuron_threshold(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNPUDataError> {
        self.neuron_threshold.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_threshold(&mut self, neuron_id: NeuronIndex, threshold: NMP) -> Result<(), FeagiNPUDataError> {
        if let Some(neuron_threshold) = self.neuron_threshold.get_mut(neuron_id as usize) {
            *neuron_threshold = threshold;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_thresholds_slice(&self) -> &[NMP] {
        &self.neuron_threshold
    }

    fn get_neuron_thresholds_slice_mut(&mut self) -> &mut [NMP] {
        &mut self.neuron_threshold
    }
    //endregion

    //region Threshold Limit
    fn get_neuron_threshold_limit(&self, neuron_id: NeuronIndex) -> Result<NMP, FeagiNPUDataError> {
        self.neuron_threshold_limit.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_threshold_limit(&mut self, neuron_id: NeuronIndex, threshold_limit: NMP) -> Result<(), FeagiNPUDataError> {
        if let Some(neuron_threshold_limit) = self.neuron_threshold_limit.get_mut(neuron_id as usize) {
            *neuron_threshold_limit = threshold_limit;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_threshold_limits_slice(&self) -> &[NMP] {
        &self.neuron_threshold_limit
    }

    fn get_neuron_threshold_limits_slice_mut(&mut self) -> &mut [NMP] {
        &mut self.neuron_threshold_limit
    }
    //endregion

    //region Leak Coefficient
    fn get_neuron_leak_coefficient(&self, neuron_id: NeuronIndex) -> Result<LC, FeagiNPUDataError> {
        self.neuron_leak_coefficient.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_leak_coefficient(&mut self, neuron_id: NeuronIndex, leak_coefficient: LC) -> Result<(), FeagiNPUDataError> {
        if let Some(neuron_leak_coefficient) = self.neuron_leak_coefficient.get_mut(neuron_id as usize) {
            *neuron_leak_coefficient = leak_coefficient;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_leak_coefficients_slice(&self) -> &[LC] {
        &self.neuron_leak_coefficient
    }

    fn get_neuron_leak_coefficients_slice_mut(&mut self) -> &mut [LC] {
        &mut self.neuron_leak_coefficient
    }
    //endregion

    //region Neuron Flags

    fn get_neuron_flags(&self, neuron_id: NeuronIndex) -> Result<NeuronFlag, FeagiNPUDataError> {
        self.neuron_flags.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_flags(&self, neuron_id: NeuronIndex, neuron_flag: NeuronFlag) -> Result<() , FeagiNPUDataError> {
        if let Some(flag) = self.neuron_flags.get_mut(neuron_id as usize) {
            *flag = neuron_flag;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_flags_slice(&self) -> &[NeuronFlag] {
        &self.neuron_flags
    }

    fn get_neuron_flags_slice_mut(&mut self) -> &mut [NeuronFlag] {
        &mut self.neuron_flags
    }

    //endregion

    //region Refractory Countdown
    fn get_neuron_refractory_countdown(&self, neuron_id: NeuronIndex) -> Result<RC, FeagiNPUDataError> {
        self.neuron_refractory_countdown.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_neuron_refractory_countdown(&mut self, neuron_id: NeuronIndex, countdown: RC) -> Result<(), FeagiNPUDataError> {
        if let Some(neuron_refractory_countdown) = self.neuron_refractory_countdown.get_mut(neuron_id as usize) {
            *neuron_refractory_countdown = countdown;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_refractory_countdowns_slice(&self) -> &[RC] {
        &self.neuron_refractory_countdown
    }

    fn get_neuron_refractory_countdowns_slice_mut(&mut self) -> &mut [RC] {
        &mut self.neuron_refractory_countdown
    }
    //endregion

    //region Fire Count
    fn get_consecutive_fire_count(&self, neuron_id: NeuronIndex) -> Result<CFC, FeagiNPUDataError> {
        self.consecutive_fire_count.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_consecutive_fire_count(&mut self, neuron_id: NeuronIndex, countdown: CFC) -> Result<(), FeagiNPUDataError> {
        if let Some(consecutive_fire_count) = self.consecutive_fire_count.get_mut(neuron_id as usize) {
            *consecutive_fire_count = countdown;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_consecutive_fire_count_slice(&self) -> &[CFC] {
        &self.consecutive_fire_count
    }

    fn get_consecutive_fire_count_slice_mut(&mut self) -> &mut [CFC] {
        &mut self.consecutive_fire_count
    }
    //endregion

    //region Fire Limit
    fn get_consecutive_fire_limit(&self, neuron_id: CFL) -> Result<u16, FeagiNPUDataError> {
        self.consecutive_fire_limit.get(neuron_id.to_usize()).copied().ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_consecutive_fire_limit(&mut self, neuron_id: CFL, countdown: u16) -> Result<(), FeagiNPUDataError> {
        if let Some(consecutive_fire_limit) = self.consecutive_fire_limit.get_mut(neuron_id.to_usize()) {
            *consecutive_fire_limit = countdown;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_consecutive_fire_limit_slice(&self) -> &[u16] {
        &self.consecutive_fire_limit
    }

    fn get_consecutive_fire_limit_slice_mut(&mut self) -> &mut [u16] {
        &mut self.consecutive_fire_limit
    }
    //endregion

    //region Snooze Period Countdown
    fn get_snooze_period_countdown(&self, neuron_id: NeuronIndex) -> Result<SPC, FeagiNPUDataError> {
        self.snooze_period_countdown.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_snooze_period_countdown(&mut self, neuron_id: NeuronIndex, countdown: SPC) -> Result<(), FeagiNPUDataError> {
        if let Some(snooze_period_countdown) = self.snooze_period_countdown.get_mut(neuron_id as usize) {
            *snooze_period_countdown = countdown;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_snooze_period_countdown_slice(&self) -> &[SPC] {
        &self.snooze_period_countdown
    }

    fn get_snooze_period_countdown_slice_mut(&mut self) -> &mut [SPC] {
        &mut self.snooze_period_countdown
    }
    //endregion

    //region Snooze Period Limit
    fn get_snooze_period_limit(&self, neuron_id: NeuronIndex) -> Result<SPL, FeagiNPUDataError> {
        self.snooze_period_limit.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn set_snooze_period_limit(&mut self, neuron_id: NeuronIndex, countdown: SPL) -> Result<(), FeagiNPUDataError> {
        if let Some(snooze_period_limit) = self.snooze_period_limit.get_mut(neuron_id as usize) {
            *snooze_period_limit = countdown;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_snooze_period_limit_slice(&self) -> &[SPL] {
        &self.snooze_period_limit
    }

    fn get_snooze_period_limit_slice_mut(&mut self) -> &mut [SPL] {
        &mut self.snooze_period_limit
    }
    //endregion

    //endregion

    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // since this affects neuron count, the setting functions are only in the alloc extension
    fn get_number_of_neurons_per_voxel_descriptor(&self, cortical_index: CorticalIndex) -> Result<NumberNeuronsPerVoxel, FeagiNPUDataError> {
        Ok(
            self.neuron_cortical_area_index
                .get(cortical_index as usize).ok_or_else(|| return Err(
                    FeagiNPUDataError::CorticalIndexOutOfRange{
                        given_cortical_index: cortical_index as usize,
                        range: self.neuron_cortical_area_index.len()
                    }
            )) as usize
        )
    }

    fn get_number_of_neurons_per_voxel_descriptor_slice(&self) -> &[NumberNeuronsPerVoxel] {
        todo!()
    }
    //endregion

    //endregion
}