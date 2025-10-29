/*!
Service trait definitions.

These traits define the stable application boundary between
transport adapters and domain logic.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod analytics_service;
pub mod connectome_service;
pub mod genome_service;
pub mod neuron_service;

// Re-export for convenience
pub use analytics_service::AnalyticsService;
pub use connectome_service::ConnectomeService;
pub use genome_service::GenomeService;
pub use neuron_service::NeuronService;

