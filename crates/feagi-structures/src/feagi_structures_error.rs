// Top level error enum for this crate, holds errors from individual models

use crate::FeagiStructuresGenomicError;
use crate::neuron_voxels::FeagiStructuresNeuronVoxelError;
use crate::neurons::FeagiStructuresNeuronError;

pub enum FeagiStructuresError {
    NeuronVoxelError { neuron_voxel_error: FeagiStructuresNeuronVoxelError },
    NeuronError { neuron_error: FeagiStructuresNeuronError },
    GenomicError { genomic_error: FeagiStructuresGenomicError},
    JSONError { context: &'static str},
    InvalidValue {context: &'static str}
}

// TODO automatic impls

// TODO error stuff