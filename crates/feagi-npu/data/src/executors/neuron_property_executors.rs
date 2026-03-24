

/// Runs on interneuron cortical areas, to set the neuron fire threshold across the area
pub trait NeuronFireThresholdExecutor<PotentialQuant, CoordQuant> {

    // Neuron order to be incrementing x->y->z
    fn set_new_fire_thresholds(thresholds: &mut [PotentialQuant],
                          neuron_flags: &[InterneuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>)
        -> Result<(), FeagiNPUError>;
}


/// Runs on interneuron cortical areas, to set leak coefficient across neurons
pub trait NeuronLeakCoefficientExecutor<PercentageQuant, CoordQuant> {

    // Neuron order to be incrementing x->y->z
    fn set_new_leak_coefficients(thresholds: &mut Vec<PercentageQuant>,
                          neuron_flags: &[InterneuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>)
        -> Result<(), FeagiNPUError>;
}


