use feagi_data::neurons::neuron_voxels::wrapped_values::NeuronVoxelCoordinate;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization, FeagiIndexQuantizationStandard};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DoubletIteratorDimensionalTypeGenomic {
    OneToAll {
        source: NeuronVoxelCoordinate<<FeagiIndexQuantizationStandard as FeagiIndexQuantization>::NeuronIndexQuant>,
    },
    AllToOne {
        destination: NeuronVoxelCoordinate<<FeagiIndexQuantizationStandard as FeagiIndexQuantization>::NeuronIndexQuant>,
    },
    OneToOne {
        source: NeuronVoxelCoordinate<<FeagiIndexQuantizationStandard as FeagiIndexQuantization>::NeuronIndexQuant>,
        destination: NeuronVoxelCoordinate<<FeagiIndexQuantizationStandard as FeagiIndexQuantization>::NeuronIndexQuant>,
    },
}

// TODO this should be macro generated

pub enum DoubletIteratorDimensionalType<FIQ: FeagiIndexQuantization> {
    OneToAll {
        source: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
    },
    AllToOne {
        destination: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
    },
    OneToOne {
        source: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
        destination: NeuronVoxelCoordinate<FIQ::NeuronIndexQuant>,
    },
}
