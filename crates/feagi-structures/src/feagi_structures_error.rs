// Top level error enum for this crate, holds errors from individual models

use crate::genomic::FeagiStructuresGenomicError;
use crate::neuron_old::neuron_collections::neuron_voxel_collections::FeagiStructuresNeuronVoxelError;
use crate::neuron_old::neuron_collections::individual_neuron_collections::FeagiIndividualNeuronCollectionError;

#[derive(Debug)]
pub enum FeagiStructuresError {
    NeuronVoxelError { neuron_voxel_error: FeagiStructuresNeuronVoxelError },
    NeuronError { neuron_error: FeagiIndividualNeuronCollectionError },
    GenomicError { genomic_error: FeagiStructuresGenomicError},
    JSONError { context: &'static str},
    InvalidValue {context: &'static str}
}

// TODO automatic impls

// TODO error stuff