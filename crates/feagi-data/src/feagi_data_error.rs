use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};
use crate::neuron_voxels::neuron_voxel_error::FeagiVoxelError;


generate_feagi_error! {
    FeagiDataError,
    keys: {
        
    },
    sub_errors: {
        VoxelError: FeagiVoxelError,
    },
}