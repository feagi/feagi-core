/*!
Morphology rules for synaptogenesis.

Implements various connectivity patterns:
- Projector: Map source neurons to destination maintaining topology
- Block connections: Scaled block mapping
- Expander: Scale coordinates from source to destination
- Reducer: Binary encoding/decoding
*/

mod block_connection;
mod expander;
pub mod patterns;
mod projector;
mod reducer;
mod trivial;
mod vectors;

pub use block_connection::syn_block_connection;
pub use expander::{syn_expander, syn_expander_batch};
pub use patterns::{
    find_destination_coordinates, find_source_coordinates, match_patterns_batch, Pattern3D,
    PatternElement,
};
pub use projector::{syn_projector, syn_projector_batch, ProjectorParams};
pub use reducer::syn_reducer_x;
pub use trivial::{syn_last_to_first, syn_lateral_pairs_x, syn_randomizer};
pub use vectors::{apply_vector_offset, match_vectors_batch};
