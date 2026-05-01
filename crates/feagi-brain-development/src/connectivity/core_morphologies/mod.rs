// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Core morphology implementations for synaptogenesis.

Each core morphology is implemented in its own module, making them
independent and easier to maintain.
*/

mod bitmask;
mod block_connection;
mod common;
mod expander;
mod first_to_last;
mod last_to_first;
mod patterns;
mod projector;
mod rotator_z;
mod sweeper;
mod tile;
mod vectors;

pub use bitmask::apply_bitmask_morphology_with_dimensions;
pub use bitmask::{BitmaskAxis, BitmaskMode};
pub use block_connection::apply_block_connection_morphology;
pub use block_connection::apply_block_connection_morphology_batched;
pub use expander::apply_expander_morphology;
pub use first_to_last::apply_first_to_last_morphology_with_dimensions;
pub use last_to_first::apply_last_to_first_morphology_with_dimensions;
pub use patterns::apply_patterns_morphology;
pub use projector::apply_projector_morphology;
pub use projector::apply_projector_morphology_with_dimensions;
pub use rotator_z::apply_rotator_z_morphology_with_dimensions;
pub use sweeper::apply_sweeper_morphology_with_dimensions;
pub use tile::apply_tile_morphology_with_dimensions;
pub use vectors::{apply_vectors_morphology, apply_vectors_morphology_with_dimensions};
