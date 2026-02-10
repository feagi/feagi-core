mod coder_shared_functions;
/// Describes encoding / decoding to NeuronVoxelXYZP format
mod coder_traits;

pub(crate) mod decoders;
pub(crate) mod encoders;
pub(crate) use coder_traits::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
