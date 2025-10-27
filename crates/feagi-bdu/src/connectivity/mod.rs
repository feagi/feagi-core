/*!
Connectivity and synaptogenesis operations.

This module implements high-performance synapse creation based on morphology rules.
*/

pub mod rules;
pub mod synaptogenesis; // NPU-native synaptogenesis (zero-copy)
pub mod synaptogenesis_legacy; // Legacy Python-driven synaptogenesis (deprecated)

// Export NPU-native synaptogenesis functions
pub use synaptogenesis::{
    apply_block_connection_morphology, apply_expander_morphology, apply_patterns_morphology,
    apply_projector_morphology, apply_vectors_morphology,
};

// Export legacy functions for backward compatibility (will be removed)
pub use synaptogenesis_legacy::{
    find_candidate_neurons, CandidateNeuron, MorphologyParams, SynaptogenesisRequest,
    SynaptogenesisResult,
};

pub use rules::{syn_projector, ProjectorParams};
