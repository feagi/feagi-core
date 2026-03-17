// TODO Rayon? Could the trait perhaps implement some sort of iterator support for rayon?

// NOTE: it makes more sense space AND compute wise to simply index the cortical area index for
// each neuron instead of having a neuron to cortical map. However, there are times a cortical to
// neuron map are useful in cases of reference speed when we can afford the memory

pub struct InterNeuronDataDynamic<NeuronIndex, CorticalIndex, CoordQuant, NMP, LC, RP, RC, EX, CFC, CFL, SPC, SPL>
where
    NeuronIndex: QuantizableUInt,
    CorticalIndex: QuantizableUInt,
    NMP: NeuralMembranePotential,
    CoordQuant: NeuronVoxelCoordinate,
    LC: LeakCoefficient,
    RP: RefractoryPeriod,
    RC: RefractoryCountdown,
    EX: Excitability,
    CFC: ConsecutiveFireCountdown,
    CFL: ConsecutiveFireLimit,
    SPC: SnoozePeriodCountdown,
    SPL: SnoozePeriodLimit,
{
    cortical_2_neuron_mapping_and_neurons_per_voxel: AHashMap<CorticalIndex, (Vec<NeuronIndex>, NumberNeuronsPerVoxel)>,
    // See note at top about lack of cortical_2_neuron

    // Per Neuron
    neuron_cortical_area_index: Vec<CorticalIndex>,
    neuron_membrane_potential: Vec<NMP>,
    neuron_voxel_coordinate: Vec<NeuronVoxelCoordinate<CoordQuant>>, // TODO Maybe we should use index instead for memory savings?
    neuron_threshold: Vec<NMP>,
    neuron_threshold_limit: Vec<NMP>,
    neuron_leak_coefficient: Vec<LC>,
    neuron_flags: Vec<NeuronFlag>,
    neuron_refractory_countdown: Vec<RC>,
    consecutive_fire_count: Vec<CFC>,
    consecutive_fire_limit: Vec<CFL>,
    snooze_period_countdown: Vec<SPC>,
    snooze_period_limit: Vec<SPL>,

    // Per Cortical Area
    cortical_refractory_period: Vec<RP>,
    cortical_excitability: Vec<EX>,
    neurons_per_voxel: Vec<NeuronIndex>,
}


impl<Neuron, Cortical, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> InterCorticalIndexData<Neuron, Cortical, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> for InterNeuronDataDynamic<Neuron, Cortical, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> {

    fn get_total_number_of_neurons(&self) -> NeuronIndex {
        self.neuron_cortical_area_index.len() as NeuronIndex  // any array works
    }

    fn get_cortical_index(&self, neuron_id: NeuronIndex) -> Result<CorticalIndex, FeagiNPUDataError> {
        self.neuron_cortical_area_index.get(neuron_id as usize).ok_or_else(|| FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_ids(&self, cortical_index: CorticalIndex) -> Result<impl Iterator<Item = &NeuronIndex>, FeagiNPUDataError> {
        Ok(self.cortical_2_neuron_mapping_and_neurons_per_voxel.get(cortical_index).ok_or_else(|| return Err(
            FeagiNPUDataError::CorticalIndexOutOfRange{
                given_cortical_index: cortical_index as usize,
                range: self.neuron_cortical_area_index.len()
            }
        )).0)

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
        Ok(self.cortical_2_neuron_mapping_and_neurons_per_voxel.get(cortical_index).ok_or_else(|| return Err(
            FeagiNPUDataError::CorticalIndexOutOfRange{
                given_cortical_index: cortical_index as usize,
                range: self.neuron_cortical_area_index.len()
            }
        )).1)
    }

    fn get_number_of_neurons_per_voxel_descriptor_slice(&self) -> &[NumberNeuronsPerVoxel] {
        todo!()
    }
    //endregion

    //endregion

}

impl<Neuron, Cortical, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> InterCorticalIndexDataAlloc<Neuron, Cortical, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> for InterNeuronDataDynamic<Neuron, Cortical, NPV, LC, RP, RC, EX, CFC, CFL, SPC, SPL> {
    //region Individual neuron Properties

    //region Neuron Voxel Coordinates
    fn set_neuron_voxel_coordinate(&mut self, neuron_id: NeuronIndex, voxel_coordinate: NeuronVoxelCoordinate<CoordQuant>) -> Result<(), FeagiNeuronError> {
        if let Some(neuron_coordinate) = self.neuron_voxel_coordinate.get_mut(neuron_id as usize) {
            *neuron_coordinate = voxel_coordinate;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_neuron_voxel_coordinates_slice_mut(&mut self) -> &mut [NeuronVoxelCoordinate<CoordQuant>] {
        &mut self.neuron_voxel_coordinate
    }
    //endregion

    //endregion


    //region Individual Cortical Area Properties

    //region Number of Neurons per Voxel

    // Note: This is just a descriptor, does NOT actually update neuron counts
    // get functions in base trait

    fn set_number_of_neurons_per_voxel_descriptor(&mut self, cortical_index: CorticalIndex, number_neurons_per_voxel: NumberNeuronsPerVoxel) -> Result<(), FeagiNeuronError> {
        if let Some(neuron_number_neurons_per_voxel) = self.cortical_2_neuron_mapping_and_neurons_per_voxel.get_mut(neuron_id) {
            *neuron_number_neurons_per_voxel = number_neurons_per_voxel;
            return Ok(());
        }
        Err(FeagiNPUDataError::NeuronIndexOutOfRange{
            given_neuron_index: &neuron_id as usize, range: self.neuron_cortical_area_index.len()
        })
    }

    fn get_number_of_neurons_per_voxel_descriptor_slice_mut(&mut self) -> &mut [NumberNeuronsPerVoxel] {
        todo!()
    }
    //endregion

    //endregion

    // TODO functions for adding / removing neurons
}