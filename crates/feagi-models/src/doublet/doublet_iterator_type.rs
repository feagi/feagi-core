use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub enum DoubletIteratorDimensionalType<FIQ: FeagiIndexQuantization> {
    OneToAll{source: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>},
    AllToOne{destination: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>},
    OneToOne{source: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>, destination: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>},
}