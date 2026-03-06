use crate::neuron::descriptors::CorticalAreaDimensions;
use crate::neuron::NeuralPotentialValue;

pub struct NeuronVoxelPArray<Potential>
where Potential: NeuralPotentialValue {
    area_dimensions: CorticalAreaDimensions,
    data: [Potential]
}

impl<Potential: NeuralPotentialValue> NeuronVoxelPArray<Potential> {
    pub fn new(area_dimensions: CorticalAreaDimensions) -> Self {
        let size = area_dimensions.number_elements();
        NeuronVoxelPArray {
            area_dimensions: area_dimensions,
            data: [Potential::default(); size]
        }
    }
}

