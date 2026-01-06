// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Core morphology implementations for synaptogenesis.

Each core morphology is implemented in its own module, making them
independent and easier to maintain.
*/

mod block_connection;
mod common;
mod expander;
mod patterns;
mod projector;
mod vectors;

pub use block_connection::apply_block_connection_morphology;
pub use block_connection::apply_block_connection_morphology_batched;
pub use expander::apply_expander_morphology;
pub use patterns::apply_patterns_morphology;
pub use projector::apply_projector_morphology;
pub use projector::apply_projector_morphology_with_dimensions;
pub use vectors::{apply_vectors_morphology, apply_vectors_morphology_with_dimensions};

