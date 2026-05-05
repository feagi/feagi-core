use feagi_structures::quantization::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
use crate::FeagiNPUStructureError;
use crate::neuron::FeagiNPUNeuronError;
use crate::neuron::flags::NeuronFlag;
use crate::quantizables::{FireThreshold};



//region Neuron Fire Threshold

/// Runs on dimensional_neuron cortical areas, to set the neuron fire threshold across the area
pub trait NeuronFireThresholdExecutor<ValueQuant, CoordQuant> where
    ValueQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
{

    // Neuron order to be incrementing x->y->z
    fn set_new_fire_thresholds(&self, thresholds: &mut [FireThreshold<ValueQuant>],
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

impl<ValueQuant, CoordQuant> NeuronFireThresholdExecutor<ValueQuant, CoordQuant> for SetUniformNeuronFireThreshold<ValueQuant> where
    ValueQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType
{
    // Neuron order to be incrementing x->y->z
    fn set_new_fire_thresholds(&self,
                               thresholds: &mut [FireThreshold<ValueQuant>],
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
pub trait NeuronLeakCoefficientExecutor<PercentageQuant, CoordQuant> where
    PercentageQuant: QuantizablePercentType,
    CoordQuant: QuantizableUIntType
{

    // Neuron order to be incrementing x->y->z
    fn set_new_leak_coefficients(thresholds: &mut Vec<PercentageQuant>,
                          neuron_flags: &[NeuronFlag],
                          cortical_dimensions: &NeuronVoxelDimensions<CoordQuant>,
                          neuron_density: NumberNeuronsPerVoxel)
        -> Result<(), FeagiNPUStructureError>;
}


//endregion