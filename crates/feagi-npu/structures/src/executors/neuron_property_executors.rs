use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::FeagiNPUStructureError;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{FireThreshold};



//region Neuron Fire Threshold

/// Runs on dimensional_neuron cortical areas, to set the neuron fire threshold across the area
pub trait NeuronFireThresholdExecutor<PotentialQuant, CoordQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
{

    // Neuron order to be incrementing x->y->z
    fn set_new_fire_thresholds(&self, thresholds: &mut [PotentialQuant],
                          neuron_flags: &[NeuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                          number_neurons_per_voxel: NumberNeuronsPerVoxel)
        -> Result<(), FeagiNPUNeuronError>;
}

//region Set Uniform
pub struct SetUniformNeuronFireThreshold<PotentialQuant> where
    PotentialQuant: QuantizableValueType,
{
    fire_threshold: FireThreshold<PotentialQuant>,
}

impl<PotentialQuant> SetUniformNeuronFireThreshold<PotentialQuant> where
    PotentialQuant: QuantizableValueType
{
    pub fn new(fire_threshold: FireThreshold<PotentialQuant>) -> Self
    { Self { fire_threshold } }
}

impl<PotentialQuant, CoordQuant> NeuronFireThresholdExecutor<PotentialQuant, CoordQuant> for SetUniformNeuronFireThreshold<PotentialQuant> where
    PotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType
{
    // Neuron order to be incrementing x->y->z
    fn set_new_fire_thresholds(&self,
                               thresholds: &mut [FireThreshold<PotentialQuant>],
                               _neuron_flags: &[NeuronFlag],
                               _cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                               _number_neurons_per_voxel: NumberNeuronsPerVoxel)
                               -> Result<(), FeagiNPUNeuronError>
    {
        // TODO enable rayon support if feature enabled
        thresholds.fill(self.fire_threshold);
        Ok(())
    }
}
//endregion


//endregion

//region Neuron Leak Coefficient

/// Runs on dimensional_neuron cortical areas, to set leak coefficient across neurons
pub trait NeuronLeakCoefficientExecutor<PercentageQuant, CoordQuant> {

    // Neuron order to be incrementing x->y->z
    fn set_new_leak_coefficients(thresholds: &mut Vec<PercentageQuant>,
                          neuron_flags: &[NeuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                          neuron_density: NumberNeuronsPerVoxel)
        -> Result<(), FeagiNPUStructureError>;
}


//endregion