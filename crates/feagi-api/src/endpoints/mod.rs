// Unified, transport-agnostic endpoint implementations
// These endpoints are called by both HTTP and ZMQ adapters

pub mod health;
pub mod cortical_areas;
pub mod brain_regions;
pub mod genome;
pub mod neurons;
pub mod runtime;
pub mod analytics;

// TODO: Add more endpoint modules
// pub mod mappings;

